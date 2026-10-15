//! `trinity netloss`: the training loss of one or more float checkpoints on
//! the same sample of training positions, computed the trainer's way.
//!
//! Usage: trinity netloss <data file> <positions> <checkpoint dirs...>
//!
//! The data file is shuffled training data in bullet's 32-byte format (the
//! parts the training scripts make), so its first `positions` records are a
//! random sample. Each checkpoint folder holds optimiser_state\weights.bin
//! (the trainer's float weights; 1024, 1536 or 2048 neurons, 8 king buckets,
//! 8 output buckets). The loss is the one the trainer minimises: the
//! squared difference between sigmoid(output) and the target
//! wdl x result + (1 - wdl) x sigmoid(score / 400), with wdl = 0.5 as in
//! the training runs; the pure-result (wdl 1) and pure-score (wdl 0) losses
//! and the mean |output - score| in cp are reported as well.
//!
//! Why: the 1024 run's loss log was not kept, so "compare the loss at
//! superbatch 300" means scoring its checkpoint and the new run's checkpoint
//! on the same positions. These are training positions (both runs trained
//! on the whole file), so this measures fit, not generalisation.

use bulletformat::ChessBoard;

use crate::nnue::{self, INPUTS, KING_BUCKETS, OUTPUT_BUCKETS, SCALE};
use crate::quantcheck::read_weights;
use crate::types::*;

/// Float weights of any supported width, in the engine's layout.
struct FloatNet {
    hidden: usize,
    /// `[feature][neuron]`, feature = king bucket * 768 + piece feature.
    l0w: Vec<f32>,
    l0b: Vec<f32>,
    /// `[bucket][2 * hidden]`: side to move first, then the other side.
    l1w: Vec<f32>,
    l1b: Vec<f32>,
}

fn load(dir: &str) -> Result<FloatNet, String> {
    let path = format!("{dir}/optimiser_state/weights.bin");
    let tensors = read_weights(&path)?;
    let get = |name: &str| -> Result<&Vec<f32>, String> {
        tensors.iter().find(|(n, _)| n == name).map(|(_, v)| v).ok_or(format!("{path} has no tensor '{name}'"))
    };
    let l0b = get("l0b")?.clone();
    let hidden = l0b.len();
    if ![512, 1024, 1536, 2048].contains(&hidden) {
        return Err(format!("{path}: {hidden} neurons is not a size Trinity uses"));
    }
    let per_bucket = get("l0w")?;
    if per_bucket.len() != KING_BUCKETS * INPUTS * hidden {
        return Err(format!(
            "{path}: l0w has {} values, expected {}",
            per_bucket.len(),
            KING_BUCKETS * INPUTS * hidden
        ));
    }
    let shared: Vec<f32> = match get("l0f") {
        Ok(f) if f.len() == INPUTS * hidden => f.clone(),
        Ok(f) => return Err(format!("{path}: l0f has {} values, expected {}", f.len(), INPUTS * hidden)),
        Err(_) => vec![0.0; INPUTS * hidden],
    };
    let l0w: Vec<f32> = per_bucket.iter().enumerate().map(|(i, w)| w + shared[i % (INPUTS * hidden)]).collect();
    let l1_raw = get("l1w")?;
    if l1_raw.len() != 2 * hidden * OUTPUT_BUCKETS {
        return Err(format!("{path}: l1w has {} values, expected {}", l1_raw.len(), 2 * hidden * OUTPUT_BUCKETS));
    }
    let mut l1w = vec![0f32; 2 * hidden * OUTPUT_BUCKETS];
    for input in 0..2 * hidden {
        for bucket in 0..OUTPUT_BUCKETS {
            l1w[bucket * 2 * hidden + input] = l1_raw[input * OUTPUT_BUCKETS + bucket];
        }
    }
    let l1b = get("l1b")?.clone();
    if l1b.len() != OUTPUT_BUCKETS {
        return Err(format!("{path}: l1b has {} values, expected {OUTPUT_BUCKETS}", l1b.len()));
    }
    Ok(FloatNet { hidden, l0w, l0b, l1w, l1b })
}

/// The network's raw output for a record (side to move's view; x SCALE = cp).
/// Records are stored from the side to move's view: its pieces are "white",
/// the board is flipped when Black is to move.
fn output(f: &FloatNet, rec: &ChessBoard) -> f64 {
    let h = f.hidden;
    // `opp_ksq` is stored already flipped to the opponent's own view.
    let views = [nnue::king_bucket(usize::from(rec.ksq)), nnue::king_bucket(usize::from(rec.opp_ksq))];
    let mut hidden = [f.l0b.iter().map(|&x| f64::from(x)).collect::<Vec<f64>>(), Vec::new()];
    hidden[1] = hidden[0].clone();
    for (pc, sq) in *rec {
        let piece = make_piece(usize::from(pc >> 3), usize::from(pc & 7));
        for (persp, v) in views.iter().enumerate() {
            let feat = nnue::feature(persp, *v, piece, usize::from(sq));
            let col = &f.l0w[feat * h..(feat + 1) * h];
            for (a, w) in hidden[persp].iter_mut().zip(col) {
                *a += f64::from(*w);
            }
        }
    }
    let bucket = (rec.occ.count_ones() as usize - 2) / 4;
    let w = &f.l1w[bucket * 2 * h..(bucket + 1) * 2 * h];
    let mut out = f64::from(f.l1b[bucket]);
    for (k, side) in hidden.iter().enumerate() {
        for (i, &x) in side.iter().enumerate() {
            let v = x.clamp(0.0, 1.0);
            out += v * v * f64::from(w[k * h + i]);
        }
    }
    out
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Losses of one record: [wdl 0.5, wdl 1, wdl 0, |cp error|].
fn losses(f: &FloatNet, rec: &ChessBoard) -> [f64; 4] {
    let out = output(f, rec);
    let pred = sigmoid(out);
    let result = f64::from(rec.result) / 2.0;
    let score = f64::from(rec.score);
    let score_p = sigmoid(score / f64::from(SCALE));
    let loss = |wdl: f64| {
        let t = wdl * result + (1.0 - wdl) * score_p;
        (pred - t) * (pred - t)
    };
    [loss(0.5), loss(1.0), loss(0.0), (out * f64::from(SCALE) - score).abs()]
}

/// The first `count` records of a data file.
fn read_sample(path: &str, count: usize) -> Result<Vec<ChessBoard>, String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).map_err(|e| format!("{path}: {e}"))?;
    let mut bytes = vec![0u8; 32 * count];
    let mut read = 0;
    while read < bytes.len() {
        let n = file.read(&mut bytes[read..]).map_err(|e| format!("{path}: {e}"))?;
        if n == 0 {
            break;
        }
        read += n;
    }
    bytes.truncate(read - read % 32);
    let all = bytes.len() / 32;
    let records: Vec<ChessBoard> = bytes.chunks_exact(32).map(parse).filter(valid).collect();
    if records.len() < all {
        println!("{} malformed records skipped", all - records.len());
    }
    Ok(records)
}

/// A record the network can evaluate: 2 to 32 pieces, piece codes 0-5 of
/// either colour, both kings on the board, a result of 0, 1 or 2.
fn valid(rec: &ChessBoard) -> bool {
    let n = rec.occ.count_ones();
    (2..=32).contains(&n)
        && rec.result <= 2
        && rec.ksq < 64
        && rec.opp_ksq < 64
        && rec.occ & (1u64 << rec.ksq) != 0
        && rec.occ & (1u64 << (rec.opp_ksq ^ 56)) != 0
        && (*rec).into_iter().all(|(pc, _)| pc & 7 <= 5)
}

fn parse(c: &[u8]) -> ChessBoard {
    ChessBoard {
        occ: u64::from_le_bytes(c[0..8].try_into().unwrap()),
        pcs: c[8..24].try_into().unwrap(),
        score: i16::from_le_bytes([c[24], c[25]]),
        result: c[26],
        ksq: c[27],
        opp_ksq: c[28],
        extra: [c[29], c[30], c[31]],
    }
}

/// Mean losses over the sample, computed on all cores.
fn mean_losses(f: &FloatNet, sample: &[ChessBoard]) -> [f64; 4] {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 64);
    let chunk = sample.len().div_ceil(threads).max(1);
    let sums: Vec<[f64; 4]> = std::thread::scope(|s| {
        let handles: Vec<_> = sample
            .chunks(chunk)
            .map(|part| {
                s.spawn(move || {
                    let mut acc = [0f64; 4];
                    for rec in part {
                        for (a, l) in acc.iter_mut().zip(losses(f, rec)) {
                            *a += l;
                        }
                    }
                    acc
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let n = sample.len().max(1) as f64;
    let mut total = [0f64; 4];
    for s in sums {
        for (t, v) in total.iter_mut().zip(s) {
            *t += v;
        }
    }
    total.map(|v| v / n)
}

pub fn run(data: &str, count: usize, checkpoints: &[String]) -> Result<(), String> {
    let sample = read_sample(data, count)?;
    if sample.is_empty() {
        return Err(format!("{data} holds no positions"));
    }
    println!(
        "NETWORK LOSS on the first {} positions of {data} (training positions; wdl 0.5 is the loss the trainer minimised)",
        sample.len()
    );
    println!(
        "{:60} {:>7} {:>10} {:>10} {:>10} {:>9}",
        "checkpoint", "neurons", "loss(0.5)", "loss(wdl)", "loss(cp)", "|cp err|"
    );
    for dir in checkpoints {
        let f = match load(dir) {
            Ok(f) => f,
            Err(e) => {
                println!("{dir:60} skipped: {e}");
                continue;
            }
        };
        let [l, lw, lc, cp] = mean_losses(&f, &sample);
        let name = dir.trim_end_matches(['/', '\\']).rsplit(['/', '\\']).next().unwrap_or(dir);
        println!("{name:60} {:>7} {l:>10.6} {lw:>10.6} {lc:>10.6} {cp:>9.1}", f.hidden);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::datagen::to_bullet;
    use bulletformat::BulletFormat;

    /// Records written by the engine's own data writer come back unchanged,
    /// and the float output computed from a record equals the one computed
    /// from the board by the quantisation check (same weights).
    #[test]
    fn record_output_matches_board_output() {
        let Some(net) = nnue::network() else { return };
        // The embedded network, de-quantised, as a float checkpoint.
        let dir = std::env::temp_dir().join("trinity-netloss-test");
        std::fs::create_dir_all(dir.join("optimiser_state")).unwrap();
        let h = nnue::HIDDEN;
        let mut tensors: Vec<(String, Vec<f32>)> = Vec::new();
        let l0w: Vec<f32> = (0..KING_BUCKETS * INPUTS)
            .flat_map(|f| net.feature_weights(f).iter().map(|&w| f32::from(w) / nnue::QA as f32).collect::<Vec<_>>())
            .collect();
        tensors.push(("l0w".into(), l0w));
        tensors.push(("l0b".into(), net.feature_bias().iter().map(|&w| f32::from(w) / nnue::QA as f32).collect()));
        let mut l1w = vec![0f32; 2 * h * OUTPUT_BUCKETS];
        for b in 0..OUTPUT_BUCKETS {
            for i in 0..2 * h {
                l1w[i * OUTPUT_BUCKETS + b] = f32::from(net.output_weights(b)[i]) / nnue::QB as f32;
            }
        }
        tensors.push(("l1w".into(), l1w));
        tensors.push((
            "l1b".into(),
            (0..OUTPUT_BUCKETS).map(|b| f32::from(net.output_bias(b)) / (nnue::QA * nnue::QB) as f32).collect(),
        ));
        let mut buf = Vec::new();
        for (name, vals) in &tensors {
            buf.extend_from_slice(name.as_bytes());
            buf.push(b'\n');
            buf.extend_from_slice(&(vals.len() as u64).to_le_bytes());
            for v in vals {
                buf.extend_from_slice(&v.to_le_bytes());
            }
        }
        let weights = dir.join("optimiser_state").join("weights.bin");
        std::fs::write(&weights, buf).unwrap();

        let fens = [
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
            "rnbqkb1r/pp1p1ppp/2p2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 1",
            "8/8/3k4/8/8/4K3/8/8 w - - 0 1",
            "6k1/5ppp/8/8/8/8/5PPP/3R2K1 b - - 0 1",
        ];
        let boards: Vec<Board> = fens.iter().map(|f| Board::from_fen(f).unwrap()).collect();
        let records: Vec<ChessBoard> =
            boards.iter().enumerate().map(|(i, b)| to_bullet(b, 37 * i as i32 - 70, 0.5)).collect();
        let data = dir.join("sample.data");
        let mut w = std::io::BufWriter::new(std::fs::File::create(&data).unwrap());
        ChessBoard::write_to_bin(&mut w, &records).unwrap();
        drop(w);
        let back = read_sample(data.to_str().unwrap(), 100).unwrap();
        assert_eq!(back, records);

        let f = load(dir.to_str().unwrap()).unwrap();
        assert_eq!(f.hidden, h);
        for (b, rec) in boards.iter().zip(&back) {
            let from_record = output(&f, rec) * f64::from(SCALE);
            let from_board = crate::quantcheck::float_eval_for_test(weights.to_str().unwrap(), b);
            assert!(
                (from_record - from_board).abs() < 1e-6 * (1.0 + from_board.abs()),
                "{from_record} vs {from_board}"
            );
        }
        // The loss of a perfect prediction is zero; of a sure win called a sure loss, one.
        let [l, lw, lc, _] = mean_losses(&f, &back);
        assert!(l > 0.0 && l < 1.0 && lw < 1.0 && lc < 1.0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
