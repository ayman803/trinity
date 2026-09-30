//! Hand-crafted evaluation (HCE). This is a deliberately simple stand-in
//! that is used until a trained NNUE network is available, and to generate
//! the first batch of training data. Scores are in centipawns from the side
//! to move's point of view.
//!
//! Terms are "tapered": each has a middlegame (mg) and endgame (eg) value,
//! blended by how much non-pawn material is left on the board.

use crate::attacks;
use crate::board::Board;
use crate::types::*;

#[derive(Clone, Copy)]
struct S(i32, i32);

impl std::ops::Add for S {
    type Output = S;
    fn add(self, o: S) -> S {
        S(self.0 + o.0, self.1 + o.1)
    }
}
impl std::ops::AddAssign for S {
    fn add_assign(&mut self, o: S) {
        self.0 += o.0;
        self.1 += o.1;
    }
}
impl std::ops::SubAssign for S {
    fn sub_assign(&mut self, o: S) {
        self.0 -= o.0;
        self.1 -= o.1;
    }
}
impl std::ops::Mul<i32> for S {
    type Output = S;
    fn mul(self, k: i32) -> S {
        S(self.0 * k, self.1 * k)
    }
}

const MATERIAL: [S; 6] = [S(100, 120), S(320, 300), S(335, 320), S(480, 530), S(960, 960), S(0, 0)];
const PHASE_WEIGHT: [i32; 6] = [0, 1, 1, 2, 4, 0];
const MAX_PHASE: i32 = 24;

const BISHOP_PAIR: S = S(30, 55);
const DOUBLED_PAWN: S = S(-10, -20);
const ISOLATED_PAWN: S = S(-12, -12);
/// Passed pawn bonus by relative rank.
const PASSED: [S; 8] = [S(0, 0), S(0, 10), S(5, 15), S(10, 30), S(25, 55), S(45, 95), S(70, 140), S(0, 0)];
const ROOK_OPEN_FILE: S = S(25, 10);
const ROOK_SEMI_OPEN_FILE: S = S(12, 8);
/// Per-square mobility weights for knight, bishop, rook, queen.
const MOBILITY: [S; 4] = [S(4, 4), S(5, 5), S(2, 4), S(1, 2)];
const TEMPO: i32 = 12;

/// Distance of a square from the centre, 0 (centre) .. 3 (edge).
fn centre_distance(sq: Square) -> i32 {
    let f = file_of(sq) as i32;
    let r = rank_of(sq) as i32;
    let df = (2 * f - 7).abs() / 2;
    let dr = (2 * r - 7).abs() / 2;
    df.max(dr)
}

/// Piece-square bonus for piece type `pt` on `sq`, from White's view.
fn psqt(pt: usize, sq: Square) -> S {
    let r = rank_of(sq) as i32;
    let f = file_of(sq) as i32;
    let cd = centre_distance(sq);
    match pt {
        PAWN => {
            let central = if (2..=5).contains(&f) { 1 } else { 0 };
            S((r - 1) * 3 + central * r * 3, (r - 1) * 8)
        }
        KNIGHT => S(12 - 10 * cd, 8 - 8 * cd),
        BISHOP => S(8 - 5 * cd, 6 - 5 * cd),
        ROOK => S(if r == 6 { 15 } else { 0 } + if (3..=4).contains(&f) { 5 } else { 0 }, 0),
        QUEEN => S(4 - 3 * cd, 8 - 6 * cd),
        KING => {
            // Middlegame: stay home, prefer the castled corners.
            let shelter = match f {
                0..=2 | 6..=7 => 20,
                _ => 0,
            };
            S(shelter - 30 * r.min(3), 20 - 12 * cd)
        }
        _ => S(0, 0),
    }
}

const fn build_file_masks() -> [Bitboard; 8] {
    let mut m = [0u64; 8];
    let mut f = 0;
    while f < 8 {
        m[f] = FILE_A << f;
        f += 1;
    }
    m
}
static FILE_MASKS: [Bitboard; 8] = build_file_masks();

fn adjacent_files(f: usize) -> Bitboard {
    let mut m = 0;
    if f > 0 {
        m |= FILE_MASKS[f - 1];
    }
    if f < 7 {
        m |= FILE_MASKS[f + 1];
    }
    m
}

/// Squares in front of a pawn (on its own and adjacent files) that an enemy
/// pawn would have to occupy to stop it.
fn passed_span(color: usize, sq: Square) -> Bitboard {
    let files = FILE_MASKS[file_of(sq)] | adjacent_files(file_of(sq));
    let r = rank_of(sq);
    let ahead = if color == WHITE {
        if r == 7 { 0 } else { !0u64 << ((r + 1) * 8) }
    } else if r == 0 {
        0
    } else {
        !0u64 >> ((8 - r) * 8)
    };
    files & ahead
}

fn eval_side(b: &Board, us: usize) -> S {
    let them = us ^ 1;
    let occ = b.occupied();
    let our_pawns = b.piece_bb(us, PAWN);
    let their_pawns = b.piece_bb(them, PAWN);
    let mut s = S(0, 0);

    // Squares attacked by enemy pawns are poor targets for mobility.
    let mut enemy_pawn_attacks = 0;
    for sq in Bits(their_pawns) {
        enemy_pawn_attacks |= attacks::pawn(them, sq);
    }
    let mobility_area = !b.colors[us] & !enemy_pawn_attacks;

    for pt in PAWN..=KING {
        for sq in Bits(b.piece_bb(us, pt)) {
            let rel = if us == WHITE { sq } else { flip(sq) };
            s += MATERIAL[pt] + psqt(pt, rel);
            if (KNIGHT..=QUEEN).contains(&pt) {
                let n = (attacks::piece_attacks(pt, sq, occ) & mobility_area).count_ones() as i32;
                let center = [4, 6, 7, 13][pt - KNIGHT];
                s += MOBILITY[pt - KNIGHT] * (n - center);
            }
            if pt == ROOK {
                let file = FILE_MASKS[file_of(sq)];
                if file & our_pawns == 0 {
                    s += if file & their_pawns == 0 { ROOK_OPEN_FILE } else { ROOK_SEMI_OPEN_FILE };
                }
            }
        }
    }

    if b.piece_bb(us, BISHOP).count_ones() >= 2 {
        s += BISHOP_PAIR;
    }

    for sq in Bits(our_pawns) {
        let f = file_of(sq);
        if FILE_MASKS[f] & our_pawns & !bb(sq) != 0 {
            s += DOUBLED_PAWN * 1; // counted once per pawn on a doubled file
        }
        if adjacent_files(f) & our_pawns == 0 {
            s += ISOLATED_PAWN;
        }
        if passed_span(us, sq) & their_pawns == 0 {
            s += PASSED[relative_rank(us, sq)];
        }
    }
    s
}

pub fn evaluate(b: &Board) -> i32 {
    let mut s = eval_side(b, WHITE);
    s -= eval_side(b, BLACK);
    let mut phase = 0;
    for pt in KNIGHT..=QUEEN {
        phase += PHASE_WEIGHT[pt] * b.pieces[pt].count_ones() as i32;
    }
    let phase = phase.min(MAX_PHASE);
    let score = (s.0 * phase + s.1 * (MAX_PHASE - phase)) / MAX_PHASE;
    let score = if b.stm == WHITE { score } else { -score };
    score + TEMPO
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symmetric() {
        // Mirroring the position (and swapping colours) must give the same score.
        let pairs = [
            (
                "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
                "rnbqk2r/pppp1ppp/5n2/2b1p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R b KQkq - 4 4",
            ),
            ("8/5pk1/6p1/8/3P4/8/5PPP/6K1 w - - 0 1", "6k1/5ppp/8/3p4/8/6P1/5PK1/8 b - - 0 1"),
        ];
        for (a, b) in pairs {
            let ea = evaluate(&Board::from_fen(a).unwrap());
            let eb = evaluate(&Board::from_fen(b).unwrap());
            assert_eq!(ea, eb, "{a} vs {b}");
        }
    }

    #[test]
    fn material_matters() {
        let up_a_queen = Board::from_fen("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert!(evaluate(&up_a_queen) > 700);
    }
}
