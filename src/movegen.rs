//! Fully legal move generation using check masks and pin rays, so the
//! search never has to make a move just to find out it was illegal.

use crate::attacks;
use crate::board::{Board, NO_SQ};
use crate::types::*;

/// Generate all legal moves. With `noisy_only`, only captures and
/// promotions are produced (used by quiescence search).
pub fn generate(b: &Board, list: &mut MoveList, noisy_only: bool) {
    let us = b.stm;
    let them = us ^ 1;
    let occ = b.occupied();
    let own = b.colors[us];
    let enemy = b.colors[them];
    let ksq = b.king_sq(us);

    // King moves: target squares must not be attacked with the king lifted
    // off the board (so it cannot hide behind itself from a slider).
    let occ_no_king = occ ^ bb(ksq);
    let king_targets = attacks::king(ksq) & !own & if noisy_only { enemy } else { !0 };
    for to in Bits(king_targets) {
        if !b.is_attacked(to, them, occ_no_king) {
            let flag = if enemy & bb(to) != 0 { FLAG_CAPTURE } else { FLAG_QUIET };
            list.push(Move::new(ksq, to, flag));
        }
    }

    let checkers = b.checkers;
    if checkers.count_ones() > 1 {
        return; // double check: only the king can move
    }
    // Squares a non-king move must land on: anywhere, or (in check) on the
    // checker or between it and the king.
    let check_mask = if checkers != 0 {
        let c = checkers.trailing_zeros() as Square;
        checkers | attacks::between(ksq, c)
    } else {
        !0
    };

    // Pinned pieces: own pieces that are the only blocker between our king
    // and an enemy slider.
    let mut pinned = 0u64;
    let snipers = (attacks::rook(ksq, 0) & (b.pieces[ROOK] | b.pieces[QUEEN])
        | attacks::bishop(ksq, 0) & (b.pieces[BISHOP] | b.pieces[QUEEN]))
        & enemy;
    for s in Bits(snipers) {
        let blockers = attacks::between(ksq, s) & occ;
        if blockers.count_ones() == 1 && blockers & own != 0 {
            pinned |= blockers;
        }
    }

    let target = if noisy_only { enemy } else { !own } & check_mask;

    // Knights (a pinned knight can never move).
    for from in Bits(b.piece_bb(us, KNIGHT) & !pinned) {
        push_targets(list, from, attacks::knight(from) & target, enemy);
    }
    // Sliders.
    for from in Bits((b.pieces[BISHOP] | b.pieces[QUEEN]) & own) {
        let mut t = attacks::bishop(from, occ) & target;
        if pinned & bb(from) != 0 {
            t &= attacks::line(ksq, from);
        }
        push_targets(list, from, t, enemy);
    }
    for from in Bits((b.pieces[ROOK] | b.pieces[QUEEN]) & own) {
        let mut t = attacks::rook(from, occ) & target;
        if pinned & bb(from) != 0 {
            t &= attacks::line(ksq, from);
        }
        push_targets(list, from, t, enemy);
    }

    gen_pawns(b, list, noisy_only, pinned, check_mask, ksq);

    if !noisy_only && checkers == 0 {
        gen_castling(b, list, occ);
    }
}

#[inline(always)]
fn push_targets(list: &mut MoveList, from: Square, targets: Bitboard, enemy: Bitboard) {
    for to in Bits(targets) {
        let flag = if enemy & bb(to) != 0 { FLAG_CAPTURE } else { FLAG_QUIET };
        list.push(Move::new(from, to, flag));
    }
}

#[inline(always)]
fn push_promos(list: &mut MoveList, from: Square, to: Square, capture: bool, noisy_only: bool) {
    let base = if capture { FLAG_PROMO_CAPTURE } else { FLAG_PROMO };
    // Queen first; in quiescence we skip under-promotions entirely.
    list.push(Move::new(from, to, base + 3));
    if !noisy_only {
        list.push(Move::new(from, to, base));
        list.push(Move::new(from, to, base + 2));
        list.push(Move::new(from, to, base + 1));
    }
}

fn gen_pawns(b: &Board, list: &mut MoveList, noisy_only: bool, pinned: Bitboard, check_mask: Bitboard, ksq: Square) {
    let us = b.stm;
    let them = us ^ 1;
    let occ = b.occupied();
    let enemy = b.colors[them];
    let promo_rank = if us == WHITE { RANK_8 } else { RANK_1 };
    let double_rank = if us == WHITE { RANK_4 } else { RANK_5 };

    for from in Bits(b.piece_bb(us, PAWN)) {
        let pin_ray = if pinned & bb(from) != 0 { attacks::line(ksq, from) } else { !0 };
        let allowed = check_mask & pin_ray;

        // Pushes.
        let one = if us == WHITE { from + 8 } else { from.wrapping_sub(8) };
        if occ & bb(one) == 0 {
            if bb(one) & allowed != 0 {
                if bb(one) & promo_rank != 0 {
                    push_promos(list, from, one, false, noisy_only);
                } else if !noisy_only {
                    list.push(Move::new(from, one, FLAG_QUIET));
                }
            }
            let two = if us == WHITE { from + 16 } else { from.wrapping_sub(16) };
            if !noisy_only && two < 64 && bb(two) & double_rank != 0 && occ & bb(two) == 0 && bb(two) & allowed != 0 {
                list.push(Move::new(from, two, FLAG_DOUBLE_PUSH));
            }
        }

        // Captures.
        let caps = attacks::pawn(us, from) & enemy & allowed;
        for to in Bits(caps) {
            if bb(to) & promo_rank != 0 {
                push_promos(list, from, to, true, noisy_only);
            } else {
                list.push(Move::new(from, to, FLAG_CAPTURE));
            }
        }

        // En passant: verify legality directly by checking that the king is
        // not exposed once both pawns leave their squares.
        if b.ep != NO_SQ {
            let ep = b.ep as Square;
            if attacks::pawn(us, from) & bb(ep) != 0 {
                let cap_sq = ep ^ 8;
                let occ_after = occ ^ bb(from) ^ bb(cap_sq) | bb(ep);
                let rooks = (b.pieces[ROOK] | b.pieces[QUEEN]) & enemy;
                let bishops = (b.pieces[BISHOP] | b.pieces[QUEEN]) & enemy;
                let exposed = attacks::rook(ksq, occ_after) & rooks | attacks::bishop(ksq, occ_after) & bishops;
                // In check, en passant is only legal if it removes the checker
                // or blocks the check.
                let resolves = check_mask & (bb(cap_sq) | bb(ep)) != 0;
                if exposed == 0 && resolves {
                    list.push(Move::new(from, ep, FLAG_EP));
                }
            }
        }
    }
}

fn gen_castling(b: &Board, list: &mut MoveList, occ: Bitboard) {
    let us = b.stm;
    let them = us ^ 1;
    let (k_right, q_right, base) = if us == WHITE { (CASTLE_WK, CASTLE_WQ, 0) } else { (CASTLE_BK, CASTLE_BQ, 56) };
    if b.castling & k_right != 0
        && occ & (bb(base + 5) | bb(base + 6)) == 0
        && !b.is_attacked(base + 5, them, occ)
        && !b.is_attacked(base + 6, them, occ)
    {
        list.push(Move::new(base + 4, base + 6, FLAG_KING_CASTLE));
    }
    if b.castling & q_right != 0
        && occ & (bb(base + 1) | bb(base + 2) | bb(base + 3)) == 0
        && !b.is_attacked(base + 3, them, occ)
        && !b.is_attacked(base + 2, them, occ)
    {
        list.push(Move::new(base + 4, base + 2, FLAG_QUEEN_CASTLE));
    }
}

pub fn legal_moves(b: &Board) -> MoveList {
    let mut list = MoveList::new();
    generate(b, &mut list, false);
    list
}

/// Count leaf nodes of the legal move tree to `depth` (move-generator test).
pub fn perft(b: &Board, depth: u32) -> u64 {
    let mut list = MoveList::new();
    generate(b, &mut list, false);
    if depth <= 1 {
        return if depth == 0 { 1 } else { list.len() as u64 };
    }
    list.iter().map(|&m| perft(&b.make_move(m), depth - 1)).sum()
}

/// Perft broken down per root move (the output format of `go perft`).
pub fn divide(b: &Board, depth: u32) -> u64 {
    let mut total = 0;
    for &m in legal_moves(b).iter() {
        let n = if depth <= 1 { 1 } else { perft(&b.make_move(m), depth - 1) };
        println!("{m}: {n}");
        total += n;
    }
    println!("\nNodes searched: {total}");
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Perft positions (standard test suite plus en-passant, castling and
    /// promotion edge cases). Node counts cross-checked with the python-chess
    /// library, an independent move generator.
    const CASES: &[(&str, &[u64])] = &[
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", &[20, 400, 8_902, 197_281, 4_865_609]),
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", &[48, 2_039, 97_862, 4_085_603]),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", &[14, 191, 2_812, 43_238, 674_624]),
        ("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", &[6, 264, 9_467, 422_333]),
        ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", &[44, 1_486, 62_379, 2_103_487]),
        ("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", &[46, 2_079, 89_890, 3_894_594]),
        ("8/8/8/2k5/3Pp3/8/8/4K2R b K d3 0 1", &[9, 140, 1_021, 17_523, 121_692, 2_123_164]),
        ("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", &[18, 92, 1_670, 10_138, 185_429, 1_134_888]),
        ("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", &[13, 102, 1_266, 10_276, 135_655, 1_015_133]),
        ("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", &[15, 126, 1_928, 13_931, 206_379, 1_440_467]),
        ("5k2/8/8/8/8/8/8/4K2R w K - 0 1", &[15, 66, 1_198, 6_399, 120_330, 661_072]),
        ("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", &[16, 71, 1_286, 7_418, 141_077, 803_711]),
        ("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", &[26, 1_141, 27_826, 1_274_206]),
        ("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", &[44, 1_494, 50_509, 1_720_476]),
        ("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", &[11, 133, 1_442, 19_174, 266_199, 3_821_001]),
        ("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", &[29, 165, 5_160, 31_961, 1_004_658]),
        ("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", &[9, 40, 472, 2_661, 38_983, 217_342]),
        ("8/P1k5/K7/8/8/8/8/8 w - - 0 1", &[6, 27, 273, 1_329, 18_135, 92_683]),
        ("K1k5/8/P7/8/8/8/8/8 w - - 0 1", &[2, 6, 13, 63, 382, 2_217]),
        ("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", &[10, 25, 268, 926, 10_857, 43_261]),
        ("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", &[37, 183, 6_559, 23_527, 811_573]),
    ];

    #[test]
    fn perft_suite() {
        for (fen, counts) in CASES {
            let b = Board::from_fen(fen).unwrap();
            for (d, &expected) in counts.iter().enumerate() {
                assert_eq!(perft(&b, d as u32 + 1), expected, "perft({}) failed for {fen}", d + 1);
            }
        }
    }

    #[test]
    fn incremental_hash_matches_full_recompute() {
        fn walk(b: &Board, depth: u32) {
            assert_eq!(b.hash, b.compute_hash(), "hash mismatch at {}", b.to_fen());
            let fen_round_trip = Board::from_fen(&b.to_fen()).unwrap();
            assert_eq!(fen_round_trip.hash, b.hash, "FEN round trip changed hash at {}", b.to_fen());
            if depth == 0 {
                return;
            }
            for &m in legal_moves(b).iter() {
                walk(&b.make_move(m), depth - 1);
            }
        }
        walk(&Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap(), 3);
        walk(&Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap(), 4);
    }

    #[test]
    fn noisy_generation_is_subset() {
        let b = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
        let all = legal_moves(&b);
        let mut noisy = MoveList::new();
        generate(&b, &mut noisy, true);
        let expected = all.iter().filter(|m| m.is_capture() || (m.is_promotion() && m.promo_piece() == QUEEN)).count();
        assert_eq!(noisy.len(), expected);
        assert!(noisy.iter().all(|m| all.contains(*m)));
    }
}
