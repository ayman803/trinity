//! Trains Trinity's NNUE with the bullet library.
//!
//! Network: (768 -> 512)x2 -> 1 with SCReLU and 8 output buckets chosen by
//! piece count. The sizes and quantisation constants below MUST match
//! `src/nnue.rs` in the engine.
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
//!       data, then refine on the stronger data.
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
    game::{inputs::Chess768, outputs::MaterialCount},
    nn::optimiser::AdamW,
    trainer::{
        save::SavedFormat,
        schedule::{TrainingSchedule, TrainingSteps, lr, wdl},
        settings::LocalSettings,
    },
    value::{ValueTrainerBuilder, loader},
};

const HIDDEN_SIZE: usize = 512;
const OUTPUT_BUCKETS: usize = 8;
const SCALE: i32 = 400;
const QA: i16 = 255;
const QB: i16 = 64;

fn usage() -> ! {
    eprintln!("usage: trinity-trainer <shuffled data file> [superbatches] [name]");
    eprintln!("       trinity-trainer convert <file.binpack> <out.data> [max positions] [skip]");
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
    let file = File::open(input).unwrap_or_else(|e| panic!("cannot open {input}: {e}"));
    let total = file.metadata().map(|m| m.len()).unwrap_or(0).max(1);
    let mut reader = CompressedTrainingDataEntryReader::new(BufReader::with_capacity(1 << 20, file))
        .unwrap_or_else(|e| panic!("{input} is not a valid binpack: {e:?}"));
    let mut writer = BufWriter::with_capacity(1 << 22, File::create(output).expect("cannot create output"));
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
    while reader.has_next() && kept < max {
        let entry = reader.next();
        seen += 1;
        if binpack_filter(&entry) {
            if let Some(board) = convert_entry(&entry) {
                buffer.push(board);
                kept += 1;
            }
        }
        if buffer.len() == buffer.capacity() {
            ChessBoard::write_to_bin(&mut writer, &buffer).expect("write failed");
            buffer.clear();
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
    let finetune = mode == Some("finetune");
    if args.len() < 2 || (finetune && args.len() < 6) {
        usage();
    }

    let mut trainer = ValueTrainerBuilder::default()
        .dual_perspective()
        .optimiser(AdamW)
        .inputs(Chess768)
        .output_buckets(MaterialCount::<OUTPUT_BUCKETS>)
        .save_format(&[
            SavedFormat::id("l0w").round().quantise::<i16>(QA),
            SavedFormat::id("l0b").round().quantise::<i16>(QA),
            // Transposed: all weights of one bucket are stored together.
            SavedFormat::id("l1w").round().quantise::<i16>(QB).transpose(),
            SavedFormat::id("l1b").round().quantise::<i16>(QA * QB),
        ])
        .loss_fn(|output, target| output.sigmoid().squared_error(target))
        .build(|builder, stm_inputs, ntm_inputs, output_buckets| {
            let l0 = builder.new_affine("l0", 768, HIDDEN_SIZE);
            let l1 = builder.new_affine("l1", 2 * HIDDEN_SIZE, OUTPUT_BUCKETS);
            let stm_hidden = l0.forward(stm_inputs).screlu();
            let ntm_hidden = l0.forward(ntm_inputs).screlu();
            l1.forward(stm_hidden.concat(ntm_hidden)).select(output_buckets)
        });

    let settings = LocalSettings { threads: 4, test_set: None, output_directory: "checkpoints", batch_queue_size: 64 };
    let steps = |superbatches: usize| TrainingSteps {
        batch_size: 16_384,
        batches_per_superbatch: 6104,
        start_superbatch: 1,
        end_superbatch: superbatches,
    };

    if finetune {
        let checkpoint = &args[2];
        let superbatches: usize = args[3].parse().unwrap_or_else(|_| usage());
        let name = args[4].clone();
        let data_path = &args[5];
        let start_lr: f32 = args.get(6).map_or(0.0005, |l| l.parse().unwrap_or_else(|_| usage()));
        trainer.load_from_checkpoint(checkpoint);

        let schedule = TrainingSchedule {
            net_id: name,
            eval_scale: SCALE as f32,
            steps: steps(superbatches),
            // Lean a little more on game results: Lc0-derived scores are
            // only approximately on our scale.
            wdl_scheduler: wdl::ConstantWDL { value: 0.5 },
            // Start lower than from scratch: the network is already trained.
            lr_scheduler: lr::StepLR { start: start_lr, gamma: 0.1, step: (superbatches * 9 / 20).max(1) },
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
