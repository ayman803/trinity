//! `trinity select`: build a "drill" training set from converted Leela
//! data (bulletformat). It keeps the positions where Trinity's own network
//! disagrees most with Leela, restricted to positions where Leela is likely
//! to be right:
//!
//! * quiet: the side to move is not in check and has no winning capture
//!   (the conversion already dropped positions whose best move is a capture
//!   or promotion);
//! * Leela's score agrees with the game result (above +1 pawn and won,
//!   below -1 pawn and lost, or in between and drawn).
//!
//! The most-disputed `fraction` of those is written out together with an
//! equal number of ordinary positions, so that the short training stage on
//! this file does not make the network forget everything else.

use std::fs::File;
use std::io::{BufWriter, Read, Write};

use bulletformat::{BulletFormat, ChessBoard};

use crate::board::Board;
use crate::movegen;
use crate::nnue;
use crate::types::*;

const CHUNK: usize = 1 << 20;
/// Disagreements are bucketed in centipawns up to this value.
const MAX_DIFF: usize = 4000;
/// Marks a position that failed the filters.
const REJECTED: u16 = u16::MAX;

/// Rebuild a board from a record. Records are stored from the side to
/// move's point of view, so this is the position seen with that side as
/// White: equivalent for evaluation (castling and en passant are unknown
/// and irrelevant here).
fn to_board(rec: &ChessBoard) -> Option<Board> {
    let mut grid = [b'.'; 64];
    for (piece, sq) in *rec {
        let rel_color = usize::from(piece >> 3);
        let pt = usize::from(piece & 7);
        let c = b"pnbrqk".get(pt).copied()?;
        grid[sq as usize] = if rel_color == 0 { c.to_ascii_uppercase() } else { c };
    }
    let mut fen = String::with_capacity(90);
    for rank in (0..8).rev() {
        let mut empty = 0;
        for file in 0..8 {
            let c = grid[rank * 8 + file];
            if c == b'.' {
                empty += 1;
            } else {
                if empty > 0 {
                    fen.push((b'0' + empty) as char);
                    empty = 0;
                }
                fen.push(c as char);
            }
        }
        if empty > 0 {
            fen.push((b'0' + empty) as char);
        }
        if rank > 0 {
            fen.push('/');
        }
    }
    fen.push_str(" w - - 0 1");
    Board::from_fen(&fen).ok()
}

/// Disagreement in centipawns, or REJECTED if the position fails a filter.
fn score_record(net: &nnue::Network, rec: &ChessBoard) -> u16 {
    let leela = i32::from(rec.score());
    let result = rec.result();
    let agrees = (leela > 100 && result > 0.75)
        || (leela < -100 && result < 0.25)
        || (leela.abs() <= 100 && (result - 0.5).abs() < 0.01);
    if !agrees {
        return REJECTED;
    }
    let Some(b) = to_board(rec) else { return REJECTED };
    if b.in_check() {
        return REJECTED;
    }
    let mut noisy = MoveList::new();
    movegen::generate(&b, &mut noisy, true);
    if noisy.as_slice().iter().any(|&m| m.is_capture() && b.see_ge(m, 1)) {
        return REJECTED;
    }
    let pair = nnue::AccPair::from_board(net, &b);
    let ours = nnue::evaluate(net, &pair, &b);
    (ours - leela).unsigned_abs().min(MAX_DIFF as u32) as u16
}

fn read_chunk(file: &mut File, buf: &mut Vec<ChessBoard>) -> usize {
    buf.resize(CHUNK, ChessBoard::default());
    // SAFETY: ChessBoard is repr(C), 32 bytes, and valid for any bit pattern.
    let bytes = unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, CHUNK * 32) };
    let mut filled = 0;
    while filled < bytes.len() {
        match file.read(&mut bytes[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(e) => panic!("read failed: {e}"),
        }
    }
    let n = filled / 32;
    buf.truncate(n);
    n
}

pub fn run(input: &str, output: &str, fraction: f64, threads: usize) -> Result<(), String> {
    let net = nnue::network().ok_or("this build has no network")?;
    let open = || File::open(input).map_err(|e| format!("cannot open {input}: {e}"));
    let total = open()?.metadata().map(|m| m.len() / 32).unwrap_or(0);

    // Pass 1: score every record (keeps 2 bytes per record in memory).
    let mut diffs: Vec<u16> = Vec::with_capacity(total as usize);
    let mut file = open()?;
    let mut buf = Vec::with_capacity(CHUNK);
    let start = std::time::Instant::now();
    loop {
        let n = read_chunk(&mut file, &mut buf);
        if n == 0 {
            break;
        }
        let base = diffs.len();
        diffs.resize(base + n, 0);
        let per = n.div_ceil(threads);
        std::thread::scope(|s| {
            for (recs, out) in buf.chunks(per).zip(diffs[base..].chunks_mut(per)) {
                s.spawn(move || {
                    for (r, d) in recs.iter().zip(out.iter_mut()) {
                        *d = score_record(net, r);
                    }
                });
            }
        });
        if diffs.len() % (CHUNK * 50) < n {
            println!(
                "scored {} of {} positions ({:.0}/s)",
                diffs.len(),
                total,
                diffs.len() as f64 / start.elapsed().as_secs_f64()
            );
        }
    }

    // Threshold for the most-disputed `fraction` of the positions that passed.
    let mut hist = vec![0u64; MAX_DIFF + 1];
    for &d in &diffs {
        if d != REJECTED {
            hist[d as usize] += 1;
        }
    }
    let passed: u64 = hist.iter().sum();
    if passed == 0 {
        return Err("no position passed the filters".into());
    }
    let wanted = (passed as f64 * fraction) as u64;
    let (mut threshold, mut above) = (MAX_DIFF, 0u64);
    while threshold > 0 && above + hist[threshold] <= wanted {
        above += hist[threshold];
        threshold -= 1;
    }
    let threshold = threshold + 1;
    // Fill the same number again with ordinary passing positions, spread
    // evenly over the file.
    let ordinary = passed - above;
    let stride = (ordinary / above.max(1)).max(1);
    println!(
        "{passed} of {} positions passed the filters ({:.1}%); keeping {above} with disagreement >= {threshold} cp plus about {} others",
        diffs.len(),
        100.0 * passed as f64 / diffs.len().max(1) as f64,
        ordinary / stride
    );

    // Pass 2: write the selection.
    let mut file = open()?;
    let mut writer = BufWriter::with_capacity(1 << 22, File::create(output).map_err(|e| e.to_string())?);
    let (mut idx, mut seen_ordinary, mut written) = (0usize, 0u64, 0u64);
    let mut out = Vec::with_capacity(CHUNK);
    loop {
        let n = read_chunk(&mut file, &mut buf);
        if n == 0 {
            break;
        }
        out.clear();
        for rec in &buf {
            let d = diffs[idx];
            idx += 1;
            if d == REJECTED {
                continue;
            }
            if usize::from(d) >= threshold {
                out.push(*rec);
            } else {
                seen_ordinary += 1;
                if seen_ordinary % stride == 0 {
                    out.push(*rec);
                }
            }
        }
        written += out.len() as u64;
        ChessBoard::write_to_bin(&mut writer, &out).map_err(|e| e.to_string())?;
    }
    writer.flush().map_err(|e| e.to_string())?;
    println!("done: wrote {written} positions to {output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datagen::to_bullet;

    #[test]
    fn record_round_trip_keeps_evaluation() {
        let Some(net) = nnue::network() else { return };
        for fen in [
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 b - - 0 1",
        ] {
            let b = Board::from_fen(fen).unwrap();
            let rec = to_bullet(&b, 0, 0.5);
            let back = to_board(&rec).unwrap();
            let eval = |x: &Board| nnue::evaluate(net, &nnue::AccPair::from_board(net, x), x);
            assert_eq!(eval(&back), eval(&b), "{fen}");
        }
    }
}
