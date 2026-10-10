//! `trinity similarity`: a move-similarity test in the style of Don Dailey's
//! "sim" tool. Several UCI engines choose a move in the same positions with
//! the same short think time; the share of positions where two engines pick
//! the same move says how alike they play. Unrelated engines typically agree
//! on roughly 45-55% of moves; much higher values suggest shared origins.
//!
//! Positions come from short random games from the starting position,
//! keeping balanced ones (Trinity's evaluation within 1.5 pawns) that are
//! not in check and have several legal moves.
//!
//! Usage: trinity similarity <positions> <movetime ms> <name=path>...

use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use crate::board::Board;
use crate::movegen;
use crate::nnue;

struct Engine {
    name: String,
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Engine {
    fn start(name: &str, path: &str) -> Result<Engine, String> {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("cannot start {name} ({path}): {e}"))?;
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut e = Engine { name: name.to_string(), child, input, output };
        e.send("uci")?;
        e.wait_for("uciok")?;
        e.send("setoption name Threads value 1")?;
        e.send("setoption name Hash value 16")?;
        e.send("isready")?;
        e.wait_for("readyok")?;
        Ok(e)
    }

    fn send(&mut self, line: &str) -> Result<(), String> {
        writeln!(self.input, "{line}").and_then(|_| self.input.flush()).map_err(|e| format!("{}: {e}", self.name))
    }

    /// Read lines until one starts with `prefix`; return that line.
    fn wait_for(&mut self, prefix: &str) -> Result<String, String> {
        let mut line = String::new();
        loop {
            line.clear();
            let n = self.output.read_line(&mut line).map_err(|e| format!("{}: {e}", self.name))?;
            if n == 0 {
                return Err(format!("{} stopped unexpectedly", self.name));
            }
            if line.trim_start().starts_with(prefix) {
                return Ok(line.trim().to_string());
            }
        }
    }

    fn best_move(&mut self, fen: &str, movetime: u64) -> Result<String, String> {
        self.send(&format!("position fen {fen}"))?;
        self.send(&format!("go movetime {movetime}"))?;
        let line = self.wait_for("bestmove")?;
        Ok(line.split_whitespace().nth(1).unwrap_or("").to_string())
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.send("quit");
        let _ = self.child.wait();
    }
}

/// Balanced, varied test positions from seeded random games.
fn positions(count: usize) -> Vec<String> {
    let net = nnue::network();
    let mut seed = 0x51u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(count);
    while out.len() < count {
        let mut b = Board::startpos();
        let plies = 8 + (next() % 17) as usize;
        let mut ok = true;
        for _ in 0..plies {
            let moves = movegen::legal_moves(&b);
            if moves.is_empty() {
                ok = false;
                break;
            }
            b = b.make_move(moves.as_slice()[(next() % moves.len() as u64) as usize]);
        }
        if !ok || b.in_check() || movegen::legal_moves(&b).len() < 5 || !seen.insert(b.hash) {
            continue;
        }
        if let Some(net) = net {
            let eval = nnue::evaluate(net, &nnue::AccPair::from_board(net, &b), &b);
            if eval.abs() > 150 {
                continue;
            }
        }
        out.push(b.to_fen());
    }
    out
}

pub fn run(count: usize, movetime: u64, specs: &[String]) -> Result<(), String> {
    let mut engines = Vec::new();
    for spec in specs {
        let (name, path) = spec.split_once('=').ok_or(format!("expected name=path, got {spec}"))?;
        engines.push(Engine::start(name, path)?);
    }
    if engines.len() < 2 {
        return Err("need at least two engines".into());
    }
    let fens = positions(count);
    let mut moves: Vec<Vec<String>> = vec![Vec::with_capacity(count); engines.len()];
    for (i, fen) in fens.iter().enumerate() {
        for (e, list) in engines.iter_mut().zip(moves.iter_mut()) {
            list.push(e.best_move(fen, movetime)?);
        }
        if (i + 1) % 100 == 0 {
            println!("{} of {} positions", i + 1, fens.len());
        }
    }
    println!();
    println!(
        "MOVE SIMILARITY: share of {} positions where two engines chose the same move ({movetime} ms each)",
        fens.len()
    );
    let width = engines.iter().map(|e| e.name.len()).max().unwrap_or(8).max(8);
    print!("{:width$}", "");
    for e in &engines {
        print!(" {:>width$}", e.name);
    }
    println!();
    for (i, a) in engines.iter().enumerate() {
        print!("{:width$}", a.name);
        for j in 0..engines.len() {
            if i == j {
                print!(" {:>width$}", "-");
            } else {
                let same = moves[i].iter().zip(&moves[j]).filter(|(x, y)| x == y).count();
                print!(" {:>width$}", format!("{:.1}%", 100.0 * same as f64 / fens.len() as f64));
            }
        }
        println!();
    }
    Ok(())
}
