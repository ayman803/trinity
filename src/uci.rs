//! UCI protocol: the text interface chess GUIs and match runners use to
//! talk to engines. See https://www.wbec-ridderkerk.nl/html/UCIProtocol.html

use std::io::BufRead;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::board::Board;
use crate::search::{self, Limits, Searcher, Shared};
use crate::types::*;

pub const NAME: &str = "Trinity";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "the Trinity developers";

/// Search threads recurse deeply; give them plenty of stack.
pub const STACK_SIZE: usize = 64 * 1024 * 1024;

pub struct Engine {
    board: Board,
    /// Hashes of the positions before `board` in the current game.
    history: Vec<u64>,
    shared: Arc<Shared>,
    searchers: Arc<Mutex<Vec<Searcher>>>,
    handle: Option<JoinHandle<()>>,
    hash_mb: usize,
    threads: usize,
    move_overhead: u64,
}

impl Engine {
    pub fn new() -> Engine {
        let hash_mb = 16;
        let shared = Shared::new(hash_mb);
        Engine {
            board: Board::startpos(),
            history: Vec::new(),
            searchers: Arc::new(Mutex::new(vec![Searcher::new(shared.clone(), 0)])),
            shared,
            handle: None,
            hash_mb,
            threads: 1,
            move_overhead: 10,
        }
    }

    fn wait(&mut self) {
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    fn rebuild(&mut self) {
        self.wait();
        self.shared = Shared::new(self.hash_mb);
        let searchers = (0..self.threads).map(|i| Searcher::new(self.shared.clone(), i)).collect();
        self.searchers = Arc::new(Mutex::new(searchers));
    }

    pub fn run(&mut self) {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            if !self.command(line.trim()) {
                break;
            }
        }
        self.shared.stop.store(true, Ordering::Relaxed);
        self.wait();
    }

    /// Handle one command; returns false on `quit`.
    pub fn command(&mut self, line: &str) -> bool {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        match tokens.first().copied() {
            Some("uci") => {
                println!("id name {NAME} {VERSION}");
                println!("id author {AUTHOR}");
                println!("option name Hash type spin default 16 min 1 max 65536");
                println!("option name Threads type spin default 1 min 1 max 1024");
                println!("option name Move Overhead type spin default 10 min 0 max 5000");
                let eval = if crate::nnue::network().is_some() { "NNUE" } else { "hand-crafted" };
                println!("info string evaluation: {eval}");
                println!("uciok");
            }
            Some("isready") => println!("readyok"),
            Some("setoption") => self.set_option(&tokens),
            Some("ucinewgame") => {
                self.wait();
                self.shared.tt.clear();
                for s in self.searchers.lock().unwrap().iter_mut() {
                    s.clear();
                }
            }
            Some("position") => {
                self.wait();
                if let Err(e) = self.set_position(&tokens) {
                    println!("info string error: {e}");
                }
            }
            Some("go") => self.go(&tokens),
            Some("stop") => self.shared.stop.store(true, Ordering::Relaxed),
            Some("quit") => return false,
            // Non-standard helpers for debugging.
            Some("d") => print!("{}", self.board.pretty()),
            Some("eval") => {
                println!("static eval (hand-crafted, side to move): {} cp", crate::eval::evaluate(&self.board));
                if let Some(net) = crate::nnue::network() {
                    let acc = crate::nnue::AccPair::from_board(net, &self.board);
                    println!(
                        "static eval (NNUE, side to move): {} cp",
                        crate::nnue::evaluate(net, &acc, self.board.stm)
                    );
                }
            }
            Some("bench") => {
                self.wait();
                let depth = tokens.get(1).and_then(|d| d.parse().ok()).unwrap_or(crate::bench::DEFAULT_DEPTH);
                crate::bench::run(depth);
            }
            Some(other) => println!("info string unknown command: {other}"),
            None => {}
        }
        true
    }

    fn set_option(&mut self, tokens: &[&str]) {
        // setoption name <name words...> value <value>
        let name_start = tokens.iter().position(|&t| t == "name").map(|i| i + 1);
        let value_pos = tokens.iter().position(|&t| t == "value");
        let Some(ns) = name_start else { return };
        let name = tokens[ns..value_pos.unwrap_or(tokens.len())].join(" ").to_lowercase();
        let value = value_pos.and_then(|v| tokens.get(v + 1)).copied().unwrap_or("");
        match name.as_str() {
            "hash" => {
                if let Ok(mb) = value.parse::<usize>() {
                    self.hash_mb = mb.clamp(1, 65536);
                    self.rebuild();
                }
            }
            "threads" => {
                if let Ok(t) = value.parse::<usize>() {
                    self.threads = t.clamp(1, 1024);
                    self.rebuild();
                }
            }
            "move overhead" => {
                if let Ok(ms) = value.parse::<u64>() {
                    self.move_overhead = ms.min(5000);
                }
            }
            _ => println!("info string unknown option: {name}"),
        }
    }

    fn set_position(&mut self, tokens: &[&str]) -> Result<(), String> {
        let moves_at = tokens.iter().position(|&t| t == "moves");
        let mut board = match tokens.get(1).copied() {
            Some("startpos") => Board::startpos(),
            Some("fen") => {
                let end = moves_at.unwrap_or(tokens.len());
                Board::from_fen(&tokens[2..end].join(" "))?
            }
            _ => return Err("expected 'startpos' or 'fen'".to_string()),
        };
        let mut history = Vec::new();
        if let Some(i) = moves_at {
            for mstr in &tokens[i + 1..] {
                let m = board.parse_uci_move(mstr).ok_or(format!("illegal move {mstr}"))?;
                history.push(board.hash);
                board = board.make_move(m);
            }
        }
        self.board = board;
        self.history = history;
        Ok(())
    }

    fn go(&mut self, tokens: &[&str]) {
        self.wait();
        let mut limits = Limits { move_overhead: self.move_overhead, ..Limits::default() };
        let num = |i: usize| tokens.get(i + 1).and_then(|v| v.parse::<i64>().ok()).map(|v| v.max(0) as u64);
        let us = self.board.stm;
        for (i, &t) in tokens.iter().enumerate() {
            match t {
                "depth" => limits.depth = num(i).map(|d| d as i32),
                "nodes" => limits.nodes = num(i),
                "movetime" => limits.movetime = num(i),
                "wtime" if us == WHITE => limits.time = num(i),
                "btime" if us == BLACK => limits.time = num(i),
                "winc" if us == WHITE => limits.inc = num(i).unwrap_or(0),
                "binc" if us == BLACK => limits.inc = num(i).unwrap_or(0),
                "movestogo" => limits.movestogo = num(i),
                "infinite" => limits.infinite = true,
                "perft" => {
                    let depth = num(i).unwrap_or(1) as u32;
                    crate::movegen::divide(&self.board, depth);
                    return;
                }
                _ => {}
            }
        }
        if limits.time.is_none() && limits.movetime.is_none() && limits.depth.is_none() && limits.nodes.is_none() {
            limits.infinite = true;
        }

        let searchers = self.searchers.clone();
        let board = self.board;
        let history = self.history.clone();
        self.shared.stop.store(false, Ordering::Relaxed);
        let handle = std::thread::Builder::new()
            .stack_size(STACK_SIZE)
            .spawn(move || {
                let mut s = searchers.lock().unwrap();
                let result = search::search_parallel(&mut s, &board, &history, &limits);
                println!("bestmove {}", result.best_move);
            })
            .expect("failed to spawn search thread");
        self.handle = Some(handle);
    }
}
