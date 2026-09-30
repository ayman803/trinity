//! `bench`: search a fixed set of positions to a fixed depth and report
//! the total node count and speed. The node count acts as a fingerprint of
//! the search: a change that is meant to be purely a speed-up must not
//! change it. Test frameworks (e.g. OpenBench) also use it to measure speed.

use std::time::Instant;

use crate::board::Board;
use crate::search::{Limits, Searcher, Shared};

pub const DEFAULT_DEPTH: i32 = 10;

const POSITIONS: &[&str] = &[
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "2r2k2/3p1p2/8/1p3N1p/1P4r1/p2K2n1/2P3P1/8 b - - 3 34",
    "rn1k4/5p2/8/pp3b2/8/P1P3K1/8/8 b - - 3 42",
    "1n3kn1/5p2/5P2/1r1P4/1p6/8/4N3/5K2 b - - 0 40",
    "4n3/7r/R1p2k2/5P1p/4P3/2b4P/6N1/6KR b - - 3 37",
    "1n3k2/5p2/3p2b1/r2Pp3/2R5/P2q4/8/K7 w - - 1 28",
    "3r4/4nkbp/5p2/4p2p/7P/1PP1P3/5PP1/1b2KBNR w - - 0 20",
    "rnb1k1n1/pp3r2/3p2p1/q7/2PBP2P/8/P2P1PP1/RN2KBNR w KQq - 0 14",
    "5r2/3p2p1/8/4P3/4k2P/RP2P3/4N1PR/2B2BK1 b - - 0 33",
    "8/5k2/3p4/3P1pn1/3K4/2B5/8/8 b - - 3 43",
    "4kb1r/4p2p/1p5n/5pp1/3P4/R1NPK1P1/5P1P/2Q2B1R b - - 1 23",
    "2b3N1/1R1ppk2/2p5/1p6/3q4/3P1P2/1PP5/2Q2K1B b - - 4 25",
    "8/6kp/8/3r3p/8/1QP3N1/4PKB1/2B5 w - - 0 36",
    "rnbqkb1r/1ppp3p/5np1/p3p3/7P/3P4/PPP1PP2/RNBQKBNR w KQkq - 0 6",
    "rn6/1k3p2/8/1pb3p1/8/5P2/1K6/1R6 w - - 0 30",
    "r1b5/n2k1p2/1p6/p7/Pb6/5N2/1B2K3/5n2 b - - 1 26",
    "7R/4nk2/8/6P1/P2P4/2RN2K1/8/5B2 w - - 3 39",
    "4r3/8/4k3/KPp1pp1P/5P2/1PN3n1/4P1BR/8 w - - 2 42",
    "rn1qkb1r/p1p1pppp/bp1p3B/8/3P4/2N3P1/PPP1PP1P/R2QKBNR w KQkq - 2 5",
    "2k1r2r/p3R3/6Pp/3p4/1n1P4/1P3BP1/PBPP4/RN1K4 w - - 0 27",
    "2b1r3/prppkp1p/1p1b2pn/P3p3/2PPP3/2N2P2/PB5P/RN1KQB1R w - - 1 15",
    "rn3k1r/p1pp1ppp/b3p3/8/2Q5/5P2/PP1PP2N/RNB1KBb1 w Q - 0 13",
    "r3kb1r/pp2pp1p/n6n/2Pp1qpP/P2P4/2P5/4PPP1/RNBQKB2 w Qkq - 1 11",
    "8/2pk3p/5Pp1/pPK5/8/5b2/8/8 w - - 1 42",
    "r7/3bp3/2p5/1p2q3/p4k2/P7/8/1K6 b - - 0 35",
    "1n1k4/r1N2p2/pp3P2/3q4/b6r/2B1Q3/2P5/R3K1NR b KQ - 0 24",
    "6n1/4k3/1Bb1p1p1/3p4/R2P4/5KPr/2P2P1P/6R1 b - - 2 28",
    "4k3/8/5K2/4rN2/8/2P5/3p4/8 w - - 8 46",
    "1n6/2p5/rp6/6k1/p1PB4/NP6/P1Q5/4K3 b - - 2 37",
    "rn3b1r/pp4pp/4kn2/2p5/7P/8/P1bP1R2/R1B1K1N1 b - - 1 18",
    "r1bq1r2/p1pBp2k/1p6/2n5/2P5/b2P3P/P2N1PP1/R2QK1R1 w Q - 1 17",
    "4n3/6b1/4knb1/2p3r1/Q1PR3P/PP6/4K1P1/1N6 w - - 4 29",
    "r1bq4/p1pp1k1n/7p/1p3p1p/1bPN4/2N3PP/4BP2/1R1Q1K1R b - - 1 17",
    "r3kb1r/1b1pp1p1/2p2n1p/p7/1P1P4/P3P2N/2BB2PP/R3K2R w KQkq - 0 16",
    "b7/8/8/1N1k1K2/2R5/1n1P3P/8/5R2 w - - 3 45",
    "2b1k1n1/1p1p2p1/2p5/4pp1P/2P4P/NP2P2N/3qP1K1/2B2B1R b - - 7 20",
    "2b5/r2p4/p4k2/1p3np1/1P6/4b3/8/K7 b - - 0 40",
    "8/7p/7k/4K3/4P3/8/7n/3r4 w - - 1 41",
    "r1bqkb2/ppppppp1/5n2/6p1/8/1QN1P3/P2P1PPr/R1B1KB1R w KQq - 0 8",
    "rnbqkbn1/2ppppp1/pp6/7r/8/3PP3/PPP2PPP/RNB1KBNR b KQq - 1 5",
];

pub fn run(depth: i32) {
    let shared = Shared::new(16);
    let mut searcher = Searcher::new(shared.clone(), 0);
    searcher.silent = true;
    let limits = Limits { depth: Some(depth), ..Limits::default() };
    let mut total_nodes = 0u64;
    let start = Instant::now();
    for fen in POSITIONS {
        let board = Board::from_fen(fen).expect("bench FEN");
        shared.tt.clear();
        searcher.clear();
        crate::search::search_once(&mut searcher, &board, &[], &limits);
        total_nodes += searcher.nodes;
    }
    let secs = start.elapsed().as_secs_f64().max(1e-3);
    println!("{total_nodes} nodes {} nps", (total_nodes as f64 / secs) as u64);
}
