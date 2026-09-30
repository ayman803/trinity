//! Attack lookup tables. Leaper tables (knight, king, pawn) and the
//! between/line tables are computed at compile time with `const fn`; slider
//! tables come from `build.rs` (magic bitboards).

use crate::types::*;

pub(crate) struct Magic {
    pub mask: u64,
    pub magic: u64,
    pub shift: u32,
    pub offset: usize,
}

include!(concat!(env!("OUT_DIR"), "/magics.rs"));

static SLIDER_TABLE: [u64; SLIDER_TABLE_LEN] = unsafe {
    // The build script writes the table as little-endian u64s.
    std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/slider_attacks.bin")))
};

const fn leaper_table(deltas: &[(i32, i32)]) -> [Bitboard; 64] {
    let mut table = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        let (f, r) = ((sq % 8) as i32, (sq / 8) as i32);
        let mut i = 0;
        while i < deltas.len() {
            let (nf, nr) = (f + deltas[i].0, r + deltas[i].1);
            if nf >= 0 && nf < 8 && nr >= 0 && nr < 8 {
                table[sq] |= 1u64 << (nr * 8 + nf);
            }
            i += 1;
        }
        sq += 1;
    }
    table
}

static KNIGHT_ATTACKS: [Bitboard; 64] =
    leaper_table(&[(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)]);
static KING_ATTACKS: [Bitboard; 64] =
    leaper_table(&[(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)]);
static PAWN_ATTACKS: [[Bitboard; 64]; 2] =
    [leaper_table(&[(-1, 1), (1, 1)]), leaper_table(&[(-1, -1), (1, -1)])];

/// Squares strictly between two squares on a shared line (empty otherwise),
/// and the full board-spanning line through both.
const fn line_tables() -> ([[Bitboard; 64]; 64], [[Bitboard; 64]; 64]) {
    let mut between = [[0u64; 64]; 64];
    let mut line = [[0u64; 64]; 64];
    let dirs: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    let mut a = 0;
    while a < 64 {
        let (fa, ra) = ((a % 8) as i32, (a / 8) as i32);
        let mut d = 0;
        while d < 8 {
            let (df, dr) = dirs[d];
            // Full ray from a in this direction and the opposite one.
            let mut full = 1u64 << a;
            let (mut f, mut r) = (fa + df, ra + dr);
            while f >= 0 && f < 8 && r >= 0 && r < 8 {
                full |= 1u64 << (r * 8 + f);
                f += df;
                r += dr;
            }
            let (mut f, mut r) = (fa - df, ra - dr);
            while f >= 0 && f < 8 && r >= 0 && r < 8 {
                full |= 1u64 << (r * 8 + f);
                f -= df;
                r -= dr;
            }
            let mut acc = 0u64;
            let (mut f, mut r) = (fa + df, ra + dr);
            while f >= 0 && f < 8 && r >= 0 && r < 8 {
                let b = (r * 8 + f) as usize;
                between[a][b] = acc;
                line[a][b] = full;
                acc |= 1u64 << b;
                f += df;
                r += dr;
            }
            d += 1;
        }
        a += 1;
    }
    (between, line)
}

static LINE_TABLES: ([[Bitboard; 64]; 64], [[Bitboard; 64]; 64]) = line_tables();

#[inline(always)]
pub fn knight(sq: Square) -> Bitboard {
    KNIGHT_ATTACKS[sq]
}

#[inline(always)]
pub fn king(sq: Square) -> Bitboard {
    KING_ATTACKS[sq]
}

/// Squares attacked by a pawn of `color` standing on `sq`.
#[inline(always)]
pub fn pawn(color: usize, sq: Square) -> Bitboard {
    PAWN_ATTACKS[color][sq]
}

#[inline(always)]
fn magic_lookup(m: &Magic, occ: Bitboard) -> Bitboard {
    let idx = ((occ & m.mask).wrapping_mul(m.magic) >> m.shift) as usize;
    // SAFETY: the build script guarantees every index for this entry lies
    // inside the entry's slice of the table.
    unsafe { *SLIDER_TABLE.get_unchecked(m.offset + idx) }
}

#[inline(always)]
pub fn rook(sq: Square, occ: Bitboard) -> Bitboard {
    magic_lookup(&ROOK_MAGICS[sq], occ)
}

#[inline(always)]
pub fn bishop(sq: Square, occ: Bitboard) -> Bitboard {
    magic_lookup(&BISHOP_MAGICS[sq], occ)
}

#[inline(always)]
pub fn queen(sq: Square, occ: Bitboard) -> Bitboard {
    rook(sq, occ) | bishop(sq, occ)
}

#[inline(always)]
pub fn between(a: Square, b: Square) -> Bitboard {
    LINE_TABLES.0[a][b]
}

#[inline(always)]
pub fn line(a: Square, b: Square) -> Bitboard {
    LINE_TABLES.1[a][b]
}

/// Attacks of a non-pawn piece type from `sq`.
#[inline(always)]
pub fn piece_attacks(pt: usize, sq: Square, occ: Bitboard) -> Bitboard {
    match pt {
        KNIGHT => knight(sq),
        BISHOP => bishop(sq, occ),
        ROOK => rook(sq, occ),
        QUEEN => queen(sq, occ),
        KING => king(sq),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slow_rook(sq: Square, occ: Bitboard) -> Bitboard {
        let mut out = 0;
        for (df, dr) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let (mut f, mut r) = (file_of(sq) as i32 + df, rank_of(sq) as i32 + dr);
            while (0..8).contains(&f) && (0..8).contains(&r) {
                let s = (r * 8 + f) as usize;
                out |= bb(s);
                if occ & bb(s) != 0 {
                    break;
                }
                f += df;
                r += dr;
            }
        }
        out
    }

    #[test]
    fn rook_magics_match_slow_attacks() {
        let mut x = 0x1234_5678_9abc_def1u64;
        for _ in 0..20_000 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let occ = x & x.rotate_left(17);
            let sq = (x >> 58) as usize;
            assert_eq!(rook(sq, occ), slow_rook(sq, occ));
        }
    }

    #[test]
    fn line_and_between() {
        // a1 .. h8 diagonal
        assert_eq!(between(0, 63).count_ones(), 6);
        assert_eq!(line(9, 18).count_ones(), 8);
        assert_eq!(between(0, 17), 0); // not aligned
        assert_eq!(knight(0).count_ones(), 2);
        assert_eq!(king(27).count_ones(), 8);
        assert_eq!(pawn(WHITE, 8), bb(17));
    }
}
