//! `trinity netloss`: the loss of one or more float checkpoints on the same
//! positions, computed the trainer's way, so two training runs can be
//! compared at the same superbatch.
//!
//! Usage: trinity netloss <positions> <checkpoint dirs...>
//!            [--games=<pgn file>]... [--train=<data file>]
//!
//! Two kinds of positions:
//! - Held-out: Trinity's own games (fastchess PGN files, `--games`), which
//!   no network was trained on; positions from ply 16 on, side to move not
//!   in check, spread evenly over all games, up to `positions`. The only
//!   label is the game's result, so the loss reported is the pure-result
//!   loss (sigmoid(output) vs 0, 0.5 or 1). A bigger network cannot win
//!   here by memorising its training data.
//! - Training: a shuffled training file in bullet's 32-byte format
//!   (`--train`), of which the first `positions` records are a random
//!   sample. The loss is the one the trainer minimised: sigmoid(output) vs
//!   wdl x result + (1 - wdl) x sigmoid(score / 400) with wdl 0.5; the
//!   pure-result and pure-score losses and the mean |output - score| in cp
//!   are shown too. This measures fit, not generalisation; the gap between
//!   the two kinds is the memorisation.
//!
//! Each checkpoint folder holds optimiser_state\weights.bin (the trainer's
//! float weights; 1024, 1536 or 2048 neurons, 8 king buckets, 8 output
//! buckets). Why: the 1024 run's loss log was not kept.

use bulletformat::ChessBoard;

use crate::board::Board;
use crate::datagen::to_bullet;
use crate::forensics::{Game, read_games};
use crate::nnue::{self, INPUTS, KING_BUCKETS, OUTPUT_BUCKETS, SCALE};
use crate::quantcheck::read_weights;
use crate::types::*;

/// Float weights of any supported width, in the engine's layout.
struct FloatNet {
    hidden: usize,
    /// False for the older networks without king buckets: one set of input
    /// weights, no mirroring.
    bucketed: bool,
    /// `[feature][neuron]`, feature = king bucket * 768 + piece feature.
    l0w: Vec<f32>,
    l0b: Vec<f32>,
    /// `[bucket][2 * hidden]`: side to move first, then the other side.
    l1w: Vec<f32>,
    l1b: Vec<f32>,
}

/// A checkpoint folder (float weights) or an integer network file
/// (`.nnue`, de-quantised: the engine's own numbers divided by QA and QB).
fn load(path: &str) -> Result<FloatNet, String> {
    if path.ends_with(".nnue") { load_nnue(path) } else { load_checkpoint(path) }
}

/// The integer network layouts the engine accepts (src/nnue.rs): 256
/// neurons with one output (the first networks), 512 without king buckets
/// (networks 4 to 6 and "fresh"), and 512 to 2048 with 8 king buckets.
fn load_nnue(path: &str) -> Result<FloatNet, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let layouts: Vec<(usize, usize, usize)> = [(256, 1, 1), (512, 1, 8), (1024, 1, 8)]
        .into_iter()
        .chain([512, 1024, 1536, 2048].into_iter().map(|h| (h, KING_BUCKETS, OUTPUT_BUCKETS)))
        .collect();
    let (hidden, king_buckets, out_buckets) = layouts
        .into_iter()
        .find(|&(h, kb, ob)| {
            let size = 2 * (kb * INPUTS * h + h + ob * (2 * h + 1));
            (size..size + 64).contains(&bytes.len())
        })
        .ok_or(format!("{path} is {} bytes: not a network layout Trinity knows", bytes.len()))?;
    let mut vals = bytes.chunks_exact(2).map(|c| f32::from(i16::from_le_bytes([c[0], c[1]])));
    let mut take = |n: usize, q: f32| -> Vec<f32> { (0..n).map(|_| vals.next().unwrap_or(0.0) / q).collect() };
    let (qa, qb) = (nnue::QA as f32, nnue::QB as f32);
    let mut l0w = take(king_buckets * INPUTS * hidden, qa);
    if king_buckets == 1 {
        // The same weights for every king bucket (the file has one set).
        l0w = l0w.repeat(KING_BUCKETS);
    }
    let l0b = take(hidden, qa);
    let mut l1w = take(out_buckets * 2 * hidden, qb);
    let mut l1b = take(out_buckets, qa * qb);
    if out_buckets == 1 {
        l1w = l1w.repeat(OUTPUT_BUCKETS);
        l1b = l1b.repeat(OUTPUT_BUCKETS);
    }
    Ok(FloatNet { hidden, bucketed: king_buckets > 1, l0w, l0b, l1w, l1b })
}

fn load_checkpoint(dir: &str) -> Result<FloatNet, String> {
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
    Ok(FloatNet { hidden, bucketed: true, l0w, l0b, l1w, l1b })
}

/// The network's raw output for a record (side to move's view; x SCALE = cp).
/// Records are stored from the side to move's view: its pieces are "white",
/// the board is flipped when Black is to move.
fn output(f: &FloatNet, rec: &ChessBoard) -> f64 {
    let h = f.hidden;
    // `opp_ksq` is stored already flipped to the opponent's own view.
    let views = if f.bucketed {
        [nnue::king_bucket(usize::from(rec.ksq)), nnue::king_bucket(usize::from(rec.opp_ksq))]
    } else {
        [(0, 0), (0, 0)]
    };
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

/// Positions from Trinity's games (from ply 16 on, side to move not in
/// check, at most `count`, spread evenly), labelled with the game's result
/// and a score of 0 (unknown).
fn read_games_sample(files: &[String], count: usize) -> Result<(Vec<ChessBoard>, usize, usize), String> {
    let mut games: Vec<Game> = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(f).map_err(|e| format!("{f}: {e}"))?;
        read_games(&text, &mut games, false);
    }
    let all: Vec<(Board, f32)> = games
        .iter()
        .flat_map(|g| {
            let white_result = if g.trinity == WHITE { g.score } else { 1.0 - g.score } as f32;
            g.plies.iter().skip(16).map(move |p| (p.board, white_result))
        })
        .filter(|(b, _)| !b.in_check())
        .collect();
    let total = all.len();
    let chosen: Vec<ChessBoard> = if total > count && count > 0 {
        (0..count).map(|k| &all[k * total / count]).map(|(b, r)| to_bullet(b, 0, *r)).collect()
    } else {
        all.iter().map(|(b, r)| to_bullet(b, 0, *r)).collect()
    };
    Ok((chosen, games.len(), total))
}

fn score_all(checkpoints: &[String], sample: &[ChessBoard], held_out: bool) {
    if held_out {
        println!("{:60} {:>7} {:>12}", "checkpoint", "neurons", "loss(result)");
    } else {
        println!(
            "{:60} {:>7} {:>10} {:>10} {:>10} {:>9}",
            "checkpoint", "neurons", "loss(0.5)", "loss(wdl)", "loss(cp)", "|cp err|"
        );
    }
    for dir in checkpoints {
        let f = match load(dir) {
            Ok(f) => f,
            Err(e) => {
                println!("{dir:60} skipped: {e}");
                continue;
            }
        };
        let [l, lw, lc, cp] = mean_losses(&f, sample);
        let name =
            dir.trim_end_matches(['/', '\\']).rsplit(['/', '\\']).next().unwrap_or(dir).trim_end_matches(".nnue");
        if held_out {
            println!("{name:60} {:>7} {lw:>12.6}", f.hidden);
        } else {
            println!("{name:60} {:>7} {l:>10.6} {lw:>10.6} {lc:>10.6} {cp:>9.1}", f.hidden);
        }
    }
}

pub fn run(count: usize, args: &[String]) -> Result<(), String> {
    let games: Vec<String> = args.iter().filter_map(|a| a.strip_prefix("--games=")).map(str::to_string).collect();
    let train: Option<&str> = args.iter().find_map(|a| a.strip_prefix("--train="));
    let checkpoints: Vec<String> = args.iter().filter(|a| !a.starts_with("--")).cloned().collect();
    if checkpoints.is_empty() {
        return Err("no checkpoint folders given".into());
    }
    if games.is_empty() && train.is_none() {
        return Err("give at least one --games=<pgn file> or --train=<data file>".into());
    }
    if !games.is_empty() {
        let (sample, n_games, total) = read_games_sample(&games, count)?;
        if sample.is_empty() {
            return Err("no positions found in the PGN files".into());
        }
        println!(
            "HELD-OUT LOSS on {} positions from {n_games} of Trinity's games ({total} candidates, ply 16 on, not in check; \
             label = game result; no network trained on these)",
            sample.len()
        );
        score_all(&checkpoints, &sample, true);
    }
    if let Some(data) = train {
        let sample = read_sample(data, count)?;
        if sample.is_empty() {
            return Err(format!("{data} holds no positions"));
        }
        println!(
            "TRAINING LOSS on the first {} positions of {data} (training positions: fit, not generalisation; \
             wdl 0.5 is the loss the trainer minimised)",
            sample.len()
        );
        score_all(&checkpoints, &sample, false);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        // The same network read as an integer file gives the same output.
        let as_file = load("nets/default.nnue").or_else(|_| load("../nets/default.nnue"));
        if let Ok(g) = as_file {
            for rec in &back {
                assert!((output(&g, rec) - output(&f, rec)).abs() < 1e-9);
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Integer files of the older layouts (no king buckets; 256 neurons
    /// with one output) evaluate as the engine itself evaluates them.
    #[test]
    fn older_layouts_match_the_engine() {
        use crate::nnue::{AccPair, Network, QA, QB};
        let dir = std::env::temp_dir().join("trinity-netloss-layouts");
        std::fs::create_dir_all(&dir).unwrap();
        let fens = [
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
            "6k1/5ppp/8/8/8/8/5PPP/3R2K1 b - - 0 1",
        ];
        for (seed, hidden, out_buckets) in [(7u64, nnue::HIDDEN, OUTPUT_BUCKETS), (9, 256, 1)] {
            let n = INPUTS * hidden + hidden + out_buckets * (2 * hidden + 1);
            let out_start = INPUTS * hidden + hidden;
            let mut s = seed;
            let mut bytes = Vec::new();
            for i in 0..n {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                let v: i16 = if i >= out_start { (s % 255) as i16 - 127 } else { (s % 101) as i16 - 50 };
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            bytes.resize((2 * n).div_ceil(64) * 64, 0);
            let path = dir.join(format!("layout-{hidden}.nnue"));
            std::fs::write(&path, &bytes).unwrap();
            let net: Box<Network> = Network::from_bytes(&bytes).unwrap();
            let f = load(path.to_str().unwrap()).unwrap();
            assert!(!f.bucketed && f.hidden == hidden);
            for fen in fens {
                let b = Board::from_fen(fen).unwrap();
                let integer = f64::from(nnue::evaluate(&net, &AccPair::from_board(&net, &b), &b));
                let float = output(&f, &to_bullet(&b, 0, 0.5)) * f64::from(SCALE);
                let tol = 1.0 + float.abs() * 0.02 + f64::from(SCALE) / f64::from(QA * QB);
                assert!((integer - float).abs() <= tol, "hidden {hidden}: {integer} vs {float}");
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
