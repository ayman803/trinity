//! Trains Trinity's NNUE with the bullet library.
//!
//! Network: (768x8 -> 512)x2 -> 1 with SCReLU, 8 king input buckets
//! (horizontally mirrored) and 8 output buckets chosen by piece count. The
//! sizes, bucket layout and quantisation constants below MUST match
//! `src/nnue.rs` in the engine.
//!
//! The input layer is trained as a shared part (`l0f`, the "factoriser",
//! the same for every king bucket) plus a per-bucket part (`l0w`); the saved
//! network contains their sum.
//!
//! Usage:
//!   trinity-trainer <shuffled data file> [superbatches] [name]
//!       Train from scratch on our own self-play data (bulletformat).
//!   trinity-trainer convert <file.binpack> <out.data> [max positions] [skip]
//!       Convert a Stockfish-format binpack (such as the Lc0-derived
//!       datasets) to bulletformat, keeping only useful positions.
//!   trinity-trainer finetune <checkpoint dir> <superbatches> <name> <shuffled data file> [start lr]
//!       Continue training a finished network on other data (the converted
//!       Lc0 data), as the Stockfish team advises: first learn from our own
//!       data, then refine on the stronger data. A checkpoint of the older
//!       network without king buckets is accepted too: its input weights
//!       become the shared part and the per-bucket parts start at zero, so
//!       training starts from (almost exactly) the old network.
//! Output: checkpoints/<name>-<N>/quantised.bin  (copy it to nets/default.nnue)

use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
};

use bulletformat::{BulletFormat, ChessBoard};
use sfbinpack::{
    CompressedTrainingDataEntryReader, TrainingDataEntry,
    chess::{color::Color, r#move::MoveType, piecetype::PieceType},
};

use bullet_lib::{
    game::{
        inputs::{ChessBucketsMirrored, get_num_buckets},
        outputs::MaterialCount,
    },
    nn::{
        InitSettings, Shape,
        optimiser::{AdamW, AdamWParams},
    },
    trainer::{
        save::SavedFormat,
        schedule::{TrainingSchedule, TrainingSteps, lr, wdl},
        settings::LocalSettings,
    },
    value::{ValueTrainerBuilder, loader},
};

/// Hidden layer size: 512 unless the environment variable TRINITY_HIDDEN
/// says otherwise (1024 is the other size the engine supports).
fn hidden_size() -> usize {
    std::env::var("TRINITY_HIDDEN").ok().and_then(|v| v.parse().ok()).unwrap_or(512)
}
const OUTPUT_BUCKETS: usize = 8;
const SCALE: i32 = 400;
const QA: i16 = 255;
const QB: i16 = 64;

/// King bucket per own-king square (seen from its side, mirrored onto files
/// a-d; index = rank * 4 + file). MUST match `KING_BUCKET_LAYOUT` in
/// `src/nnue.rs`.
#[rustfmt::skip]
const KING_BUCKET_LAYOUT: [usize; 32] = [
    0, 1, 2, 3,
    4, 4, 5, 5,
    6, 6, 6, 6,
    6, 6, 6, 6,
    7, 7, 7, 7,
    7, 7, 7, 7,
    7, 7, 7, 7,
    7, 7, 7, 7,
];
const KING_BUCKETS: usize = get_num_buckets(&KING_BUCKET_LAYOUT);

/// Weights file format used in bullet checkpoints: per tensor, its name and
/// a newline, its length as a little-endian u64, then f32 values.
fn read_weights(path: &str) -> Vec<(String, Vec<f32>)> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let nl = bytes[i..].iter().position(|&c| c == b'\n').expect("bad weights file") + i;
        let name = String::from_utf8_lossy(&bytes[i..nl]).into_owned();
        let len = u64::from_le_bytes(bytes[nl + 1..nl + 9].try_into().unwrap()) as usize;
        let start = nl + 9;
        let vals =
            bytes[start..start + 4 * len].chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
        out.push((name, vals));
        i = start + 4 * len;
    }
    out
}

fn write_weights(path: &str, weights: &[(String, Vec<f32>)]) {
    let mut buf = Vec::new();
    for (name, vals) in weights {
        buf.extend_from_slice(name.as_bytes());
        buf.push(b'\n');
        buf.extend_from_slice(&(vals.len() as u64).to_le_bytes());
        for v in vals {
            buf.extend_from_slice(&v.to_le_bytes());
        }
    }
    std::fs::write(path, buf).unwrap_or_else(|e| panic!("cannot write {path}: {e}"));
}

/// If `checkpoint` holds a network without king buckets, write a weights
/// file for the bucketed network that starts from it (old input weights as
/// the shared part, per-bucket parts zero) and return its path.
fn warm_start_weights(checkpoint: &str) -> Option<String> {
    let old = read_weights(&format!("{checkpoint}/optimiser_state/weights.bin"));
    if old.iter().any(|(name, _)| name == "l0f") {
        return None;
    }
    let mut new = Vec::new();
    for (name, vals) in old {
        if name == "l0w" {
            assert_eq!(vals.len(), 768 * hidden_size(), "unexpected input layer size in {checkpoint}");
            new.push(("l0w".to_string(), vec![0.0; vals.len() * KING_BUCKETS]));
            new.push(("l0f".to_string(), vals));
        } else {
            new.push((name, vals));
        }
    }
    let path = format!("{checkpoint}/king-bucket-start.bin");
    write_weights(&path, &new);
    Some(path)
}

fn usage() -> ! {
    eprintln!("usage: trinity-trainer <shuffled data file> [superbatches] [name]");
    eprintln!("       trinity-trainer convert <file.binpack> <out.data> [max positions] [skip]");
    eprintln!("       trinity-trainer convert-chunks <file.binpack> <out prefix> <chunks> <positions per chunk>");
    eprintln!("       trinity-trainer scratch <superbatches> <name> <start lr> <wdl> <data files...>");
    eprintln!("       trinity-trainer finetune <checkpoint dir> <superbatches> <name> <shuffled data file> [start lr]");
    std::process::exit(1);
}

/// Positions worth learning from in binpack data: past the opening, not in
/// check, a quiet best move (captures make the score unstable) and a score
/// that is not a mate.
fn binpack_filter(entry: &TrainingDataEntry) -> bool {
    entry.ply >= 16
        && !entry.pos.is_checked(entry.pos.side_to_move())
        && entry.score.unsigned_abs() <= 10_000
        && entry.mv.mtype() == MoveType::Normal
        && entry.pos.piece_at(entry.mv.to()).piece_type() == PieceType::None
}

/// Stockfish's internal score units per pawn in these datasets; scores are
/// rescaled to centipawns so they match our own data.
const SF_PAWN: i32 = 208;

/// Convert one binpack entry to bulletformat (white-relative score and
/// result), or None if it can't be represented.
fn convert_entry(entry: &TrainingDataEntry) -> Option<ChessBoard> {
    let pos = &entry.pos;
    let both = |pt| pos.pieces_bb_color(Color::White, pt).bits() | pos.pieces_bb_color(Color::Black, pt).bits();
    let bbs = [
        pos.pieces_bb(Color::White).bits(),
        pos.pieces_bb(Color::Black).bits(),
        both(PieceType::Pawn),
        both(PieceType::Knight),
        both(PieceType::Bishop),
        both(PieceType::Rook),
        both(PieceType::Queen),
        both(PieceType::King),
    ];
    let stm = usize::from(pos.side_to_move().ordinal());
    let mut score = i32::from(entry.score) * 100 / SF_PAWN;
    let mut result = f32::from(1 + entry.result) / 2.0;
    if stm == 1 {
        score = -score;
        result = 1.0 - result;
    }
    ChessBoard::from_raw(bbs, stm, score as i16, result).ok()
}

/// `skip`: number of entries at the start of the file to pass over, so a
/// later conversion can continue where an earlier one stopped.
fn convert(input: &str, output: &str, max: u64, skip: u64) {
    convert_into(input, &[output.to_string()], max, skip);
}

/// Convert into several files of `per_file` positions each (the last one may
/// hold fewer if the binpack ends), in one pass over the binpack.
fn convert_into(input: &str, outputs: &[String], per_file: u64, skip: u64) {
    let file = File::open(input).unwrap_or_else(|e| panic!("cannot open {input}: {e}"));
    let total = file.metadata().map(|m| m.len()).unwrap_or(0).max(1);
    let mut reader = CompressedTrainingDataEntryReader::new(BufReader::with_capacity(1 << 20, file))
        .unwrap_or_else(|e| panic!("{input} is not a valid binpack: {e:?}"));
    let create = |path: &str| BufWriter::with_capacity(1 << 22, File::create(path).expect("cannot create output"));
    let mut index = 0;
    let mut writer = create(&outputs[0]);
    let mut skipped = 0u64;
    while skipped < skip && reader.has_next() {
        reader.next();
        skipped += 1;
        if skipped % 100_000_000 == 0 {
            println!("skipped {skipped} of {skip} already used positions");
        }
    }
    let (mut seen, mut kept) = (0u64, 0u64);
    let mut buffer = Vec::with_capacity(1 << 16);
    let max = per_file.saturating_mul(outputs.len() as u64);
    while reader.has_next() && kept < max {
        let entry = reader.next();
        seen += 1;
        if binpack_filter(&entry) {
            if let Some(board) = convert_entry(&entry) {
                buffer.push(board);
                kept += 1;
            }
        }
        if buffer.len() == buffer.capacity() || (kept > 0 && kept % per_file == 0 && !buffer.is_empty()) {
            ChessBoard::write_to_bin(&mut writer, &buffer).expect("write failed");
            buffer.clear();
            if kept % per_file == 0 && kept < max && index + 1 < outputs.len() {
                writer.flush().expect("write failed");
                index += 1;
                writer = create(&outputs[index]);
                println!("file {} of {} written", index, outputs.len());
            }
        }
        if seen % 50_000_000 == 0 {
            println!(
                "read {seen} positions ({:.1}% of the file), kept {kept}",
                100.0 * reader.read_bytes() as f64 / total as f64
            );
        }
    }
    ChessBoard::write_to_bin(&mut writer, &buffer).expect("write failed");
    writer.flush().expect("write failed");
    println!("done: read {seen} positions, kept {kept} (stopped at file position {})", skipped + seen);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str);
    if mode == Some("convert") {
        if args.len() < 4 {
            usage();
        }
        let max = args.get(4).map_or(u64::MAX, |m| m.parse().unwrap_or_else(|_| usage()));
        let skip = args.get(5).map_or(0, |m| m.parse().unwrap_or_else(|_| usage()));
        convert(&args[2], &args[3], max, skip);
        return;
    }
    if mode == Some("convert-chunks") {
        if args.len() < 6 {
            usage();
        }
        let chunks: usize = args[4].parse().unwrap_or_else(|_| usage());
        let per_chunk: u64 = args[5].parse().unwrap_or_else(|_| usage());
        let outputs: Vec<String> = (1..=chunks).map(|i| format!("{}-{i}.raw", args[3])).collect();
        convert_into(&args[2], &outputs, per_chunk, 0);
        return;
    }
    let scratch = mode == Some("scratch");
    if scratch && args.len() < 7 {
        usage();
    }
    let hidden = hidden_size();
    println!("Hidden layer size: {hidden}");
    let finetune = mode == Some("finetune");
    // Hidden check: `kbcheck <old checkpoint> <out file>` saves the warm-started
    // king-bucket network without training (to verify the layout).
    let kbcheck = mode == Some("kbcheck");
    if args.len() < 2 || (finetune && args.len() < 6) {
        usage();
    }

    let mut trainer = ValueTrainerBuilder::default()
        .dual_perspective()
        .optimiser(AdamW)
        .inputs(ChessBucketsMirrored::new(KING_BUCKET_LAYOUT))
        .output_buckets(MaterialCount::<OUTPUT_BUCKETS>)
        .save_format(&[
            // Each bucket's weights = its own part + the shared part.
            SavedFormat::id("l0w")
                .transform(|store, weights| {
                    let shared = store.get("l0f").values.f32().repeat(KING_BUCKETS);
                    weights.into_iter().zip(shared).map(|(a, b)| a + b).collect()
                })
                .round()
                .quantise::<i16>(QA),
            SavedFormat::id("l0b").round().quantise::<i16>(QA),
            // Transposed: all weights of one bucket are stored together.
            SavedFormat::id("l1w").round().quantise::<i16>(QB).transpose(),
            SavedFormat::id("l1b").round().quantise::<i16>(QA * QB),
        ])
        .loss_fn(|output, target| output.sigmoid().squared_error(target))
        .build(move |builder, stm_inputs, ntm_inputs, output_buckets| {
            let l0f = builder.new_weights("l0f", Shape::new(hidden, 768), InitSettings::Zeroed);
            let mut l0 = builder.new_affine("l0", 768 * KING_BUCKETS, hidden);
            l0.weights = l0.weights + l0f.repeat(KING_BUCKETS);
            let l1 = builder.new_affine("l1", 2 * hidden, OUTPUT_BUCKETS);
            let stm_hidden = l0.forward(stm_inputs).screlu();
            let ntm_hidden = l0.forward(ntm_inputs).screlu();
            l1.forward(stm_hidden.concat(ntm_hidden)).select(output_buckets)
        });

    // Keep the sum of the shared and per-bucket parts within what the
    // quantised network can hold (the shared part may use the full range,
    // as the input weights of the older networks it starts from do).
    trainer
        .optimiser
        .set_params_for_weight("l0w", AdamWParams { max_weight: 0.99, min_weight: -0.99, ..Default::default() });

    let settings = LocalSettings { threads: 4, test_set: None, output_directory: "checkpoints", batch_queue_size: 64 };
    let steps = |superbatches: usize| TrainingSteps {
        batch_size: 16_384,
        batches_per_superbatch: 6104,
        start_superbatch: 1,
        end_superbatch: superbatches,
    };

    if kbcheck {
        let path = warm_start_weights(&args[2]).expect("checkpoint already has king buckets");
        if let Err(e) = trainer.optimiser.load_weights_from_file(&path) {
            panic!("cannot load {path}: {e:?}");
        }
        trainer.save_quantised(&args[3]).expect("cannot save");
        return;
    }
    if scratch {
        // scratch <superbatches> <name> <start lr> <wdl> <data files...>:
        // train from random weights; the LR slides smoothly to 1% of its start.
        let superbatches: usize = args[2].parse().unwrap_or_else(|_| usage());
        let name = args[3].clone();
        let start_lr: f32 = args[4].parse().unwrap_or_else(|_| usage());
        let wdl: f32 = args[5].parse().unwrap_or_else(|_| usage());
        let files: Vec<&str> = args[6..].iter().map(String::as_str).collect();
        let schedule = TrainingSchedule {
            net_id: name,
            eval_scale: SCALE as f32,
            steps: steps(superbatches),
            wdl_scheduler: wdl::ConstantWDL { value: wdl },
            lr_scheduler: lr::CosineDecayLR {
                initial_lr: start_lr,
                final_lr: start_lr * 0.01,
                final_superbatch: superbatches,
            },
            save_rate: 50,
        };
        let data_loader = loader::DirectSequentialDataLoader::new(&files);
        trainer.run(&schedule, &settings, &data_loader);
        return;
    }
    if finetune {
        let checkpoint = &args[2];
        let superbatches: usize = args[3].parse().unwrap_or_else(|_| usage());
        let name = args[4].clone();
        let data_path = &args[5];
        let start_lr: f32 = args.get(6).map_or(0.0005, |l| l.parse().unwrap_or_else(|_| usage()));
        match warm_start_weights(checkpoint) {
            Some(path) => {
                println!("Starting the king-bucket network from {checkpoint} (no king buckets)");
                if let Err(e) = trainer.optimiser.load_weights_from_file(&path) {
                    panic!("cannot load {path}: {e:?}");
                }
            }
            None => trainer.load_from_checkpoint(checkpoint),
        }

        let schedule = TrainingSchedule {
            net_id: name,
            eval_scale: SCALE as f32,
            steps: steps(superbatches),
            // Lean a little more on game results: Lc0-derived scores are
            // only approximately on our scale.
            wdl_scheduler: wdl::ConstantWDL { value: 0.5 },
            // Start lower than from scratch (the network is already trained)
            // and slide smoothly down to 1% of the start.
            lr_scheduler: lr::CosineDecayLR {
                initial_lr: start_lr,
                final_lr: start_lr * 0.01,
                final_superbatch: superbatches,
            },
            save_rate: 10,
        };
        let data_loader = loader::DirectSequentialDataLoader::new(&[data_path.as_str()]);
        trainer.run(&schedule, &settings, &data_loader);
    } else {
        let data_path = &args[1];
        let superbatches: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(40);
        let name = args.get(3).cloned().unwrap_or_else(|| "trinity".to_string());

        let schedule = TrainingSchedule {
            net_id: name,
            eval_scale: SCALE as f32,
            steps: steps(superbatches),
            // Blend of game result (WDL) and search score in the training target.
            wdl_scheduler: wdl::ConstantWDL { value: 0.4 },
            // Drop the learning rate 10x at 45% and 90% of training.
            lr_scheduler: lr::StepLR { start: 0.001, gamma: 0.1, step: (superbatches * 9 / 20).max(1) },
            save_rate: 10,
        };
        let data_loader = loader::DirectSequentialDataLoader::new(&[data_path.as_str()]);
        trainer.run(&schedule, &settings, &data_loader);
    }
}
