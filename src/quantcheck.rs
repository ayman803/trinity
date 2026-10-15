//! `trinity quantcheck`: how far the integer (quantised) network's
//! evaluation is from the floating-point network it was rounded from.
//!
//! Usage: trinity quantcheck <weights.bin> <max positions> <pgn files...>
//!
//! `weights.bin` is the trainer's float checkpoint
//! (checkpoints\<run>\optimiser_state\weights.bin); the integer network is
//! the one compiled into this engine. Positions come from Trinity's own
//! games (fastchess PGN files), from ply 16 on, side to move not in check,
//! spread evenly over all games.
//!
//! Reported: the gap `integer - float` in centipawns (bias, mean absolute,
//! percentiles, by output bucket), and the same gap when only the first
//! layer or only the output layer is quantised, so we know where the
//! rounding error comes from. Before that, a layout self-check: the float
//! weights are rounded the way the trainer does it and compared with the
//! integer file, number by number; anything but a near-perfect match means
//! the two files do not belong together (or the layout guess is wrong).

use crate::board::Board;
use crate::forensics::{Game, read_games};
use crate::nnue::{self, AccPair, HIDDEN, INPUTS, KING_BUCKETS, Network, OUTPUT_BUCKETS, QA, QB, SCALE};
use crate::types::*;

/// Float weights, in the engine's own layout.
struct FloatNet {
    /// `[feature][neuron]`, feature = king bucket * 768 + piece feature.
    l0w: Vec<f32>,
    l0b: Vec<f32>,
    /// `[bucket][2 * HIDDEN]`: side to move first, then the other side.
    l1w: Vec<f32>,
    l1b: Vec<f32>,
}

/// The trainer's weights file: per tensor, its name and a newline, its
/// length as a little-endian u64, then f32 values.
pub(crate) fn read_weights(path: &str) -> Result<Vec<(String, Vec<f32>)>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let nl = bytes[i..].iter().position(|&c| c == b'\n').ok_or("bad weights file")? + i;
        let name = String::from_utf8_lossy(&bytes[i..nl]).into_owned();
        let len_bytes: [u8; 8] = bytes.get(nl + 1..nl + 9).ok_or("bad weights file")?.try_into().unwrap();
        let len = u64::from_le_bytes(len_bytes) as usize;
        let start = nl + 9;
        let raw = bytes.get(start..start + 4 * len).ok_or("bad weights file (truncated)")?;
        let vals = raw.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
        out.push((name, vals));
        i = start + 4 * len;
    }
    Ok(out)
}

/// Round to the nearest integer as the trainer's `.round().quantise()` does.
fn quant(v: f32, q: i32) -> i16 {
    (v * q as f32).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

/// Build the float network from the checkpoint tensors. Per the trainer,
/// each king bucket's input weights are its own part (`l0w`) plus the shared
/// part (`l0f`); the output weights are stored input-major and transposed
/// into bucket-major order when the integer file is written.
fn load_float(path: &str) -> Result<FloatNet, String> {
    let tensors = read_weights(path)?;
    let get = |name: &str| -> Result<&Vec<f32>, String> {
        tensors.iter().find(|(n, _)| n == name).map(|(_, v)| v).ok_or(format!("{path} has no tensor '{name}'"))
    };
    let l0b = get("l0b")?.clone();
    let hidden = l0b.len();
    if hidden != HIDDEN {
        return Err(format!("{path} has {hidden} neurons; this engine is built for {HIDDEN}"));
    }
    let per_bucket = get("l0w")?;
    if per_bucket.len() != KING_BUCKETS * INPUTS * HIDDEN {
        return Err(format!(
            "{path}: l0w has {} values, expected {}",
            per_bucket.len(),
            KING_BUCKETS * INPUTS * HIDDEN
        ));
    }
    let shared: Vec<f32> = match get("l0f") {
        Ok(f) if f.len() == INPUTS * HIDDEN => f.clone(),
        Ok(f) => return Err(format!("{path}: l0f has {} values, expected {}", f.len(), INPUTS * HIDDEN)),
        Err(_) => vec![0.0; INPUTS * HIDDEN],
    };
    let l0w: Vec<f32> = per_bucket.iter().enumerate().map(|(i, w)| w + shared[i % (INPUTS * HIDDEN)]).collect();
    let l1_raw = get("l1w")?;
    if l1_raw.len() != 2 * HIDDEN * OUTPUT_BUCKETS {
        return Err(format!("{path}: l1w has {} values, expected {}", l1_raw.len(), 2 * HIDDEN * OUTPUT_BUCKETS));
    }
    // Input-major `[input][bucket]` -> bucket-major `[bucket][input]`.
    let mut l1w = vec![0f32; 2 * HIDDEN * OUTPUT_BUCKETS];
    for input in 0..2 * HIDDEN {
        for bucket in 0..OUTPUT_BUCKETS {
            l1w[bucket * 2 * HIDDEN + input] = l1_raw[input * OUTPUT_BUCKETS + bucket];
        }
    }
    let l1b = get("l1b")?.clone();
    if l1b.len() != OUTPUT_BUCKETS {
        return Err(format!("{path}: l1b has {} values, expected {OUTPUT_BUCKETS}", l1b.len()));
    }
    Ok(FloatNet { l0w, l0b, l1w, l1b })
}

/// Compare the rounded float weights with the integer network, tensor by
/// tensor: (name, total, exact matches, off by one, off by more).
fn layout_check(f: &FloatNet, net: &Network) -> Vec<(&'static str, usize, usize, usize, usize)> {
    let mut rows = Vec::new();
    let mut tally = |name: &'static str, pairs: &mut dyn Iterator<Item = (i16, i16)>| {
        let (mut n, mut exact, mut one, mut more) = (0, 0, 0, 0);
        for (a, b) in pairs {
            n += 1;
            match (i32::from(a) - i32::from(b)).abs() {
                0 => exact += 1,
                1 => one += 1,
                _ => more += 1,
            }
        }
        rows.push((name, n, exact, one, more));
    };
    tally(
        "input weights",
        &mut (0..KING_BUCKETS * INPUTS)
            .flat_map(|feat| (0..HIDDEN).map(move |i| (feat, i)))
            .map(|(feat, i)| (quant(f.l0w[feat * HIDDEN + i], QA), net.feature_weights(feat)[i])),
    );
    tally("input biases", &mut (0..HIDDEN).map(|i| (quant(f.l0b[i], QA), net.feature_bias()[i])));
    tally(
        "output weights",
        &mut (0..OUTPUT_BUCKETS)
            .flat_map(|b| (0..2 * HIDDEN).map(move |i| (b, i)))
            .map(|(b, i)| (quant(f.l1w[b * 2 * HIDDEN + i], QB), net.output_weights(b)[i])),
    );
    tally("output biases", &mut (0..OUTPUT_BUCKETS).map(|b| (quant(f.l1b[b], QA * QB), net.output_bias(b))));
    rows
}

/// The float network's hidden layer for one perspective (before SCReLU).
fn float_hidden(f: &FloatNet, net: &Network, b: &Board, perspective: usize) -> Vec<f64> {
    let v = nnue::view(net, b, perspective);
    let mut h: Vec<f64> = f.l0b.iter().map(|&x| f64::from(x)).collect();
    for sq in Bits(b.occupied()) {
        let feat = nnue::feature(perspective, v, b.piece_at(sq), sq);
        let col = &f.l0w[feat * HIDDEN..(feat + 1) * HIDDEN];
        for (a, w) in h.iter_mut().zip(col) {
            *a += f64::from(*w);
        }
    }
    h
}

/// Evaluation in centipawns from hidden layers (in units of 1.0) and output
/// weights (in units of 1.0), the way the float network computes it.
fn float_output(hidden: [&[f64]; 2], l1w: &[f64], l1b: f64) -> f64 {
    let mut out = l1b;
    for (k, h) in hidden.iter().enumerate() {
        for (i, &x) in h.iter().enumerate() {
            let v = x.clamp(0.0, 1.0);
            out += v * v * l1w[k * HIDDEN + i];
        }
    }
    out * f64::from(SCALE)
}

/// The four evaluations of one position: fully integer (the engine's own),
/// fully float, and the two half-way cases (first layer rounded only,
/// output layer rounded only). All from the side to move's view, in cp.
fn evals(f: &FloatNet, net: &Network, b: &Board) -> [f64; 4] {
    let pair = AccPair::from_board(net, b);
    let integer = f64::from(nnue::evaluate(net, &pair, b));
    let bucket = nnue::output_bucket(b);
    let sides = [b.stm, b.stm ^ 1];
    let fh: Vec<Vec<f64>> = sides.iter().map(|&p| float_hidden(f, net, b, p)).collect();
    let qh: Vec<Vec<f64>> =
        sides.iter().map(|&p| pair.acc[p].vals.iter().map(|&x| f64::from(x) / f64::from(QA)).collect()).collect();
    let fw: Vec<f64> = f.l1w[bucket * 2 * HIDDEN..(bucket + 1) * 2 * HIDDEN].iter().map(|&x| f64::from(x)).collect();
    let qw: Vec<f64> = net.output_weights(bucket).iter().map(|&x| f64::from(x) / f64::from(QB)).collect();
    let fb = f64::from(f.l1b[bucket]);
    let qb = f64::from(net.output_bias(bucket)) / f64::from(QA * QB);
    let float = float_output([&fh[0], &fh[1]], &fw, fb);
    let first_only = float_output([&qh[0], &qh[1]], &fw, fb);
    let output_only = float_output([&fh[0], &fh[1]], &qw, qb);
    [integer, float, first_only, output_only]
}

struct Stats {
    n: usize,
    bias: f64,
    mean_abs: f64,
    median: f64,
    p95: f64,
    p99: f64,
    max: f64,
    over5: f64,
    over10: f64,
}

fn stats(gaps: &[f64]) -> Stats {
    let n = gaps.len();
    if n == 0 {
        return Stats {
            n,
            bias: 0.0,
            mean_abs: 0.0,
            median: 0.0,
            p95: 0.0,
            p99: 0.0,
            max: 0.0,
            over5: 0.0,
            over10: 0.0,
        };
    }
    let mut abs: Vec<f64> = gaps.iter().map(|g| g.abs()).collect();
    abs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pct = |p: f64| abs[((n as f64 * p) as usize).min(n - 1)];
    Stats {
        n,
        bias: gaps.iter().sum::<f64>() / n as f64,
        mean_abs: abs.iter().sum::<f64>() / n as f64,
        median: pct(0.5),
        p95: pct(0.95),
        p99: pct(0.99),
        max: abs[n - 1],
        over5: 100.0 * abs.iter().filter(|&&a| a >= 5.0).count() as f64 / n as f64,
        over10: 100.0 * abs.iter().filter(|&&a| a >= 10.0).count() as f64 / n as f64,
    }
}

/// Positions from Trinity's games: from ply 16 on, side to move not in
/// check, at most `max`, spread evenly.
fn positions(files: &[String], max: usize) -> Result<(Vec<Board>, usize, usize), String> {
    let mut games: Vec<Game> = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(f).map_err(|e| format!("{f}: {e}"))?;
        read_games(&text, &mut games, false);
    }
    let all: Vec<Board> =
        games.iter().flat_map(|g| g.plies.iter().skip(16).map(|p| p.board)).filter(|b| !b.in_check()).collect();
    let total = all.len();
    let chosen = if total > max && max > 0 { (0..max).map(|k| all[k * total / max]).collect() } else { all };
    Ok((chosen, games.len(), total))
}

pub fn run(weights: &str, max: usize, files: &[String]) -> Result<(), String> {
    let net = nnue::network().ok_or("this engine was built without a network")?;
    let f = load_float(weights)?;
    println!("QUANTISATION CHECK: integer network (built into this engine) vs float weights");
    println!("Float weights: {weights}");
    println!("Layout check (float weights rounded as the trainer does, compared with the integer file):");
    let mut worst_more = 0.0f64;
    for (name, n, exact, one, more) in layout_check(&f, net) {
        let pc = |x: usize| 100.0 * x as f64 / n.max(1) as f64;
        println!(
            "  {name:15} {n:9} values: {:.3}% equal, {:.3}% off by one, {:.3}% off by more",
            pc(exact),
            pc(one),
            pc(more)
        );
        worst_more = worst_more.max(pc(more));
    }
    if worst_more > 0.1 {
        println!(
            "  WARNING: the float weights do not match the integer network; the numbers below are not meaningful."
        );
    }
    let (boards, games, total) = positions(files, max)?;
    if boards.is_empty() {
        return Err("no positions found in the PGN files".into());
    }
    println!("Positions: {} (from {games} games, {total} candidate positions, ply 16 on, not in check)", boards.len());

    let mut full: Vec<f64> = Vec::with_capacity(boards.len());
    let mut first: Vec<f64> = Vec::with_capacity(boards.len());
    let mut output: Vec<f64> = Vec::with_capacity(boards.len());
    let mut by_bucket: Vec<Vec<f64>> = vec![Vec::new(); OUTPUT_BUCKETS];
    let mut by_size: [Vec<f64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut sign_flips = 0usize;
    for b in &boards {
        let [integer, float, first_only, output_only] = evals(&f, net, b);
        full.push(integer - float);
        first.push(first_only - float);
        output.push(output_only - float);
        by_bucket[nnue::output_bucket(b)].push(integer - float);
        let size = if float.abs() < 50.0 {
            0
        } else if float.abs() < 200.0 {
            1
        } else {
            2
        };
        by_size[size].push(integer - float);
        if (integer > 0.0) != (float > 0.0) && integer != 0.0 {
            sign_flips += 1;
        }
    }
    let line = |label: &str, s: &Stats| {
        println!(
            "  {label:28} n {:6}  bias {:+.2}  mean|gap| {:.2}  median {:.2}  p95 {:.2}  p99 {:.2}  max {:.1}  >=5cp {:.1}%  >=10cp {:.1}%",
            s.n, s.bias, s.mean_abs, s.median, s.p95, s.p99, s.max, s.over5, s.over10
        );
    };
    println!("Gap = integer eval - float eval, in centipawns, side to move's view:");
    line("all positions", &stats(&full));
    line("first layer rounded only", &stats(&first));
    line("output layer rounded only", &stats(&output));
    println!("By output bucket (piece count: 2-5, 6-9, ..., 30-32):");
    for (i, g) in by_bucket.iter().enumerate() {
        line(&format!("bucket {i} ({}-{} pieces)", 4 * i + 2, (4 * i + 5).min(32)), &stats(g));
    }
    println!("By the float eval's size:");
    for (i, g) in by_size.iter().enumerate() {
        line(["|eval| < 50 cp", "50 <= |eval| < 200 cp", "|eval| >= 200 cp"][i], &stats(g));
    }
    println!("Positions where the integer and float evals have different signs: {sign_flips} of {}", boards.len());
    println!("Reading: the engine reports whole centipawns, so a gap under about 1 cp is the integer rounding alone.");
    Ok(())
}

/// The float network's eval of `b` in cp (for the netloss tests).
#[cfg(test)]
pub(crate) fn float_eval_for_test(weights: &str, b: &Board) -> f64 {
    let net = nnue::network().expect("engine built without a network");
    let f = load_float(weights).unwrap();
    evals(&f, net, b)[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Write a weights file that holds the embedded network's own values
    /// (de-quantised), then check that the layout check finds them equal and
    /// that the eval gap is the final integer rounding alone.
    #[test]
    fn dequantised_network_matches_itself() {
        let Some(net) = nnue::network() else { return };
        let mut tensors: Vec<(String, Vec<f32>)> = Vec::new();
        let l0w: Vec<f32> = (0..KING_BUCKETS * INPUTS)
            .flat_map(|f| net.feature_weights(f).iter().map(|&w| f32::from(w) / QA as f32).collect::<Vec<_>>())
            .collect();
        tensors.push(("l0w".into(), l0w));
        tensors.push(("l0f".into(), vec![0.0; INPUTS * HIDDEN]));
        tensors.push(("l0b".into(), net.feature_bias().iter().map(|&w| f32::from(w) / QA as f32).collect()));
        let mut l1w = vec![0f32; 2 * HIDDEN * OUTPUT_BUCKETS];
        for b in 0..OUTPUT_BUCKETS {
            for i in 0..2 * HIDDEN {
                l1w[i * OUTPUT_BUCKETS + b] = f32::from(net.output_weights(b)[i]) / QB as f32;
            }
        }
        tensors.push(("l1w".into(), l1w));
        tensors.push((
            "l1b".into(),
            (0..OUTPUT_BUCKETS).map(|b| f32::from(net.output_bias(b)) / (QA * QB) as f32).collect(),
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
        let path = std::env::temp_dir().join("trinity-quantcheck-test.bin");
        std::fs::write(&path, buf).unwrap();
        let f = load_float(path.to_str().unwrap()).unwrap();
        for (name, n, exact, _, _) in layout_check(&f, net) {
            assert_eq!(exact, n, "{name}");
        }
        let fens = [
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "rnbqkb1r/pp1p1ppp/2p2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 1",
            "8/8/3k4/8/8/4K3/8/8 w - - 0 1",
        ];
        for fen in fens {
            let b = Board::from_fen(fen).unwrap();
            let [integer, float, first_only, output_only] = evals(&f, net, &b);
            assert!((integer - float).abs() <= 1.0 + float.abs() * 1e-3, "{fen}: {integer} vs {float}");
            assert!((first_only - float).abs() <= 1e-6 * (1.0 + float.abs()), "{fen}");
            assert!((output_only - float).abs() <= 1e-6 * (1.0 + float.abs()), "{fen}");
        }
        let _ = std::fs::remove_file(&path);
    }
}
