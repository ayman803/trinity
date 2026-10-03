//! Trains Trinity's NNUE with the bullet library.
//!
//! Network: (768 -> 512)x2 -> 1 with SCReLU and 8 output buckets chosen by
//! piece count. The sizes and quantisation constants below MUST match
//! `src/nnue.rs` in the engine.
//!
//! Usage:
//!   trinity-trainer <shuffled data file> [superbatches] [name]
//!       Train from scratch on our own self-play data (bulletformat).
//!   trinity-trainer finetune <checkpoint dir> <superbatches> <name> <file.binpack>...
//!       Continue training a finished network on Stockfish-format binpacks
//!       (such as the Lc0-derived datasets), as the Stockfish team advises:
//!       first learn from our own data, then refine on the stronger data.
//! Output: checkpoints/<name>-<N>/quantised.bin  (copy it to nets/default.nnue)

use bullet_lib::{
    game::{inputs::Chess768, outputs::MaterialCount},
    nn::optimiser::AdamW,
    trainer::{
        save::SavedFormat,
        schedule::{TrainingSchedule, TrainingSteps, lr, wdl},
        settings::LocalSettings,
    },
    value::{
        ValueTrainerBuilder, loader,
        loader::sfbinpack::{MoveType, PieceType, SfBinpackLoader, TrainingDataEntry},
    },
};

const HIDDEN_SIZE: usize = 512;
const OUTPUT_BUCKETS: usize = 8;
const SCALE: i32 = 400;
const QA: i16 = 255;
const QB: i16 = 64;

fn usage() -> ! {
    eprintln!("usage: trinity-trainer <shuffled data file> [superbatches] [name]");
    eprintln!("       trinity-trainer finetune <checkpoint dir> <superbatches> <name> <file.binpack>...");
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let finetune = args.get(1).is_some_and(|a| a == "finetune");
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
        let files: Vec<&str> = args[5..].iter().map(String::as_str).collect();
        trainer.load_from_checkpoint(checkpoint);

        let schedule = TrainingSchedule {
            net_id: name,
            eval_scale: SCALE as f32,
            steps: steps(superbatches),
            // Lean more on game results: binpack scores use Stockfish's
            // scale, not ours.
            wdl_scheduler: wdl::ConstantWDL { value: 0.5 },
            // Start lower than from scratch: the network is already trained.
            lr_scheduler: lr::StepLR { start: 0.0005, gamma: 0.1, step: (superbatches * 9 / 20).max(1) },
            save_rate: 10,
        };
        let data_loader = SfBinpackLoader::new_concat_multiple(&files, 1024, 8, binpack_filter);
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
