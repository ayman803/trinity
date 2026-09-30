//! Self-play training data generation for NNUE.
//!
//! The engine plays fast games against itself (a fixed number of nodes per
//! move) from randomised openings, records quiet positions together with
//! the search score, and labels them with the final game result. Output is
//! in bullet's `ChessBoard` format ("bulletformat", 32 bytes per position),
//! which the trainer in `trainer/` reads directly.
//!
//! Usage: `trinity datagen threads=14 positions=20000000 nodes=5000 out=data`

use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use bulletformat::{BulletFormat, ChessBoard};

use crate::board::Board;
use crate::movegen;
use crate::search::{Limits, MATE_BOUND, Searcher, Shared, search_once};
use crate::types::*;

pub struct Config {
    pub threads: usize,
    pub positions: u64,
    pub nodes: u64,
    pub out: PathBuf,
    pub random_plies: usize,
}

impl Config {
    pub fn from_args(args: &[String]) -> Result<Config, String> {
        let mut c =
            Config { threads: 1, positions: 1_000_000, nodes: 5000, out: PathBuf::from("data"), random_plies: 8 };
        for a in args {
            let (k, v) = a.split_once('=').ok_or(format!("expected key=value, got '{a}'"))?;
            let bad = |_| format!("bad value for {k}: {v}");
            match k {
                "threads" => c.threads = v.parse().map_err(bad)?,
                "positions" => c.positions = v.parse().map_err(bad)?,
                "nodes" => c.nodes = v.parse().map_err(bad)?,
                "out" => c.out = PathBuf::from(v),
                "random_plies" => c.random_plies = v.parse().map_err(bad)?,
                _ => return Err(format!("unknown datagen option '{k}'")),
            }
        }
        c.threads = c.threads.max(1);
        Ok(c)
    }
}

/// Small, fast random number generator (xorshift64*).
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// Convert a position to bullet's format. `score` and `result` are from
/// White's point of view (result: 1.0 win, 0.5 draw, 0.0 loss).
pub fn to_bullet(b: &Board, white_score: i32, white_result: f32) -> ChessBoard {
    let bbs = [
        b.colors[WHITE],
        b.colors[BLACK],
        b.pieces[PAWN],
        b.pieces[KNIGHT],
        b.pieces[BISHOP],
        b.pieces[ROOK],
        b.pieces[QUEEN],
        b.pieces[KING],
    ];
    let score = white_score.clamp(i16::MIN as i32 + 1, i16::MAX as i32) as i16;
    ChessBoard::from_raw(bbs, b.stm, score, white_result).expect("valid board")
}

enum Outcome {
    WhiteWin,
    Draw,
    BlackWin,
}

impl Outcome {
    fn white_result(&self) -> f32 {
        match self {
            Outcome::WhiteWin => 1.0,
            Outcome::Draw => 0.5,
            Outcome::BlackWin => 0.0,
        }
    }
}

fn play_game(searcher: &mut Searcher, rng: &mut Rng, cfg: &Config, out: &mut Vec<ChessBoard>) -> usize {
    // Random opening.
    let mut board = Board::startpos();
    let plies = cfg.random_plies + rng.below(2);
    for _ in 0..plies {
        let moves = movegen::legal_moves(&board);
        if moves.is_empty() {
            return 0;
        }
        board = board.make_move(moves[rng.below(moves.len())]);
    }
    if movegen::legal_moves(&board).is_empty() {
        return 0;
    }

    searcher.shared.tt.clear();
    searcher.clear();
    let mut history: Vec<u64> = Vec::new();

    // Skip openings that are already decided.
    let check = Limits { depth: Some(10), nodes: Some(cfg.nodes * 20), ..Limits::default() };
    let r = search_once(searcher, &board, &history, &check);
    if r.score.abs() > 1000 {
        return 0;
    }

    let limits = Limits { soft_nodes: Some(cfg.nodes), nodes: Some(cfg.nodes * 20), ..Limits::default() };
    let mut positions: Vec<(Board, i32)> = Vec::with_capacity(256);
    let mut win_streak = 0;
    let mut draw_streak = 0;
    let outcome = loop {
        let moves = movegen::legal_moves(&board);
        if moves.is_empty() {
            break if board.in_check() {
                if board.stm == WHITE { Outcome::BlackWin } else { Outcome::WhiteWin }
            } else {
                Outcome::Draw
            };
        }
        let repetitions = history.iter().filter(|&&h| h == board.hash).count();
        if board.halfmove >= 100 || board.insufficient_material() || repetitions >= 2 {
            break Outcome::Draw;
        }

        let r = search_once(searcher, &board, &history, &limits);
        let white_score = if board.stm == WHITE { r.score } else { -r.score };

        // Adjudication keeps games short once the result is clear.
        if r.score.abs() >= 1500 {
            win_streak += 1;
        } else {
            win_streak = 0;
        }
        if win_streak >= 4 || r.score.abs() >= MATE_BOUND {
            break if white_score > 0 { Outcome::WhiteWin } else { Outcome::BlackWin };
        }
        if history.len() >= 80 && r.score.abs() <= 10 {
            draw_streak += 1;
        } else {
            draw_streak = 0;
        }
        if draw_streak >= 12 {
            break Outcome::Draw;
        }

        // Keep only "quiet" positions: the eval should describe the
        // position itself, not a pending capture or a check.
        if !board.in_check() && !r.best_move.is_noisy() {
            positions.push((board, white_score));
        }
        history.push(board.hash);
        board = board.make_move(r.best_move);
    };

    let result = outcome.white_result();
    let n = positions.len();
    out.extend(positions.into_iter().map(|(b, s)| to_bullet(&b, s, result)));
    n
}

/// Ask Windows not to sleep while this process runs. The request belongs to
/// the calling thread and is released automatically when the process exits.
#[cfg(windows)]
fn keep_system_awake() {
    const ES_CONTINUOUS: u32 = 0x8000_0000;
    const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetThreadExecutionState(flags: u32) -> u32;
    }
    // SAFETY: plain Win32 call with constant flags.
    if unsafe { SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED) } == 0 {
        eprintln!("warning: could not ask Windows to stay awake");
    }
}

#[cfg(not(windows))]
fn keep_system_awake() {}

pub fn run(cfg: Config) {
    keep_system_awake();
    std::fs::create_dir_all(&cfg.out).expect("cannot create output directory");
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let total = Arc::new(AtomicU64::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let start = Instant::now();
    println!(
        "datagen: {} threads, {} positions, {} nodes/move, writing to {}",
        cfg.threads,
        cfg.positions,
        cfg.nodes,
        cfg.out.display()
    );
    let cfg = Arc::new(cfg);

    let mut handles = Vec::new();
    for id in 0..cfg.threads {
        let (cfg, total, done) = (cfg.clone(), total.clone(), done.clone());
        let path = cfg.out.join(format!("trinity_{stamp}_{id}.data"));
        let h = std::thread::Builder::new()
            .stack_size(crate::uci::STACK_SIZE)
            .spawn(move || {
                let file = OpenOptions::new().create(true).append(true).open(&path).expect("cannot open output file");
                let mut writer = BufWriter::new(file);
                let mut searcher = Searcher::new(Shared::new(16), 0);
                searcher.silent = true;
                let seed =
                    stamp.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (id as u64 + 1).wrapping_mul(0xD1B5_4A32_D192_ED03);
                let mut rng = Rng(seed | 1);
                let mut buffer = Vec::with_capacity(4096);
                while !done.load(Ordering::Relaxed) {
                    let n = play_game(&mut searcher, &mut rng, &cfg, &mut buffer);
                    if buffer.len() >= 2048 {
                        write(&mut writer, &mut buffer);
                    }
                    let so_far = total.fetch_add(n as u64, Ordering::Relaxed) + n as u64;
                    if so_far >= cfg.positions {
                        done.store(true, Ordering::Relaxed);
                    }
                }
                write(&mut writer, &mut buffer);
            })
            .expect("failed to spawn datagen thread");
        handles.push(h);
    }

    let mut last_report = Instant::now();
    while !done.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(200));
        if last_report.elapsed().as_secs() >= 10 {
            last_report = Instant::now();
            let n = total.load(Ordering::Relaxed);
            let secs = start.elapsed().as_secs_f64();
            let rate = n as f64 / secs.max(1.0);
            let eta_min = (cfg.positions.saturating_sub(n)) as f64 / rate.max(1.0) / 60.0;
            println!("{n} positions ({rate:.0}/s), about {eta_min:.0} min left");
        }
    }
    for h in handles {
        let _ = h.join();
    }
    println!("done: {} positions in {:.0} s", total.load(Ordering::Relaxed), start.elapsed().as_secs_f64());
}

fn write(writer: &mut BufWriter<File>, buffer: &mut Vec<ChessBoard>) {
    use std::io::Write;
    ChessBoard::write_to_bin(writer, buffer).expect("write failed");
    writer.flush().expect("flush failed");
    buffer.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decode our bulletformat records and check every piece, the score and
    /// the result against the original board. Records are stored from the
    /// side to move's point of view (Black's boards are flipped vertically).
    #[test]
    fn bulletformat_round_trip() {
        let fens = [
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 b - - 0 1",
        ];
        for fen in fens {
            let b = Board::from_fen(fen).unwrap();
            let rec = to_bullet(&b, 123, 1.0);
            let stm = b.stm;
            // Score and result are relative to the side to move.
            assert_eq!(rec.score(), if stm == WHITE { 123 } else { -123 });
            assert_eq!(rec.result(), if stm == WHITE { 1.0 } else { 0.0 });

            let mut seen = 0;
            for (piece, sq) in rec {
                let rel_color = usize::from(piece >> 3); // 0 = side to move
                let pt = usize::from(piece & 7);
                let color = rel_color ^ stm;
                let real_sq = if stm == WHITE { sq as usize } else { flip(sq as usize) };
                assert_eq!(b.piece_at(real_sq), make_piece(color, pt), "{fen} square {real_sq}");
                seen += 1;
            }
            assert_eq!(seen, b.occupied().count_ones());
        }
    }
}
