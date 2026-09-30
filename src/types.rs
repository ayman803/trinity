//! Basic chess vocabulary: colours, pieces, squares and moves.
//!
//! Squares are numbered a1 = 0, b1 = 1, ..., h1 = 7, a2 = 8, ..., h8 = 63.

use std::fmt;

pub type Bitboard = u64;
pub type Square = usize;

pub const WHITE: usize = 0;
pub const BLACK: usize = 1;

pub const PAWN: usize = 0;
pub const KNIGHT: usize = 1;
pub const BISHOP: usize = 2;
pub const ROOK: usize = 3;
pub const QUEEN: usize = 4;
pub const KING: usize = 5;

/// A coloured piece packed as `colour * 6 + piece_type`; `NO_PIECE` marks an
/// empty square in the mailbox.
pub type Piece = u8;
pub const NO_PIECE: Piece = 12;

#[inline(always)]
pub const fn make_piece(color: usize, pt: usize) -> Piece {
    (color * 6 + pt) as Piece
}

#[inline(always)]
pub const fn piece_type(p: Piece) -> usize {
    (p % 6) as usize
}

#[inline(always)]
pub const fn piece_color(p: Piece) -> usize {
    (p / 6) as usize
}

pub fn piece_char(p: Piece) -> char {
    b"PNBRQKpnbrqk."[p as usize] as char
}

pub fn char_piece(c: char) -> Option<Piece> {
    "PNBRQKpnbrqk".find(c).map(|i| i as Piece)
}

#[inline(always)]
pub const fn file_of(sq: Square) -> usize {
    sq & 7
}

#[inline(always)]
pub const fn rank_of(sq: Square) -> usize {
    sq >> 3
}

/// Mirror a square vertically (a1 <-> a8), i.e. view the board from Black's side.
#[inline(always)]
pub const fn flip(sq: Square) -> Square {
    sq ^ 56
}

/// Rank as seen from `color`'s side (rank 1 = own back rank).
#[inline(always)]
pub const fn relative_rank(color: usize, sq: Square) -> usize {
    if color == WHITE { rank_of(sq) } else { 7 - rank_of(sq) }
}

pub fn square_name(sq: Square) -> String {
    let f = (b'a' + file_of(sq) as u8) as char;
    let r = (b'1' + rank_of(sq) as u8) as char;
    format!("{f}{r}")
}

pub fn parse_square(s: &str) -> Option<Square> {
    let b = s.as_bytes();
    if b.len() != 2 || !(b'a'..=b'h').contains(&b[0]) || !(b'1'..=b'8').contains(&b[1]) {
        return None;
    }
    Some(((b[1] - b'1') * 8 + (b[0] - b'a')) as Square)
}

pub const CASTLE_WK: u8 = 1;
pub const CASTLE_WQ: u8 = 2;
pub const CASTLE_BK: u8 = 4;
pub const CASTLE_BQ: u8 = 8;

/// A move packed into 16 bits: from (6) | to (6) | flag (4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Move(pub u16);

pub const FLAG_QUIET: u16 = 0;
pub const FLAG_DOUBLE_PUSH: u16 = 1;
pub const FLAG_KING_CASTLE: u16 = 2;
pub const FLAG_QUEEN_CASTLE: u16 = 3;
pub const FLAG_CAPTURE: u16 = 4;
pub const FLAG_EP: u16 = 5;
/// Promotion flags: 8 + (piece - KNIGHT), with bit 4 (value 4) set for captures.
pub const FLAG_PROMO: u16 = 8;
pub const FLAG_PROMO_CAPTURE: u16 = 12;

impl Move {
    pub const NONE: Move = Move(0);

    #[inline(always)]
    pub const fn new(from: Square, to: Square, flag: u16) -> Move {
        Move(from as u16 | (to as u16) << 6 | flag << 12)
    }

    #[inline(always)]
    pub const fn from(self) -> Square {
        (self.0 & 63) as Square
    }

    #[inline(always)]
    pub const fn to(self) -> Square {
        ((self.0 >> 6) & 63) as Square
    }

    #[inline(always)]
    pub const fn flag(self) -> u16 {
        self.0 >> 12
    }

    #[inline(always)]
    pub const fn is_none(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        self.flag() & FLAG_CAPTURE != 0
    }

    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        self.flag() & FLAG_PROMO != 0
    }

    #[inline(always)]
    pub const fn is_ep(self) -> bool {
        self.flag() == FLAG_EP
    }

    #[inline(always)]
    pub const fn is_castle(self) -> bool {
        self.flag() == FLAG_KING_CASTLE || self.flag() == FLAG_QUEEN_CASTLE
    }

    /// Captures and promotions: moves that change material.
    #[inline(always)]
    pub const fn is_noisy(self) -> bool {
        self.flag() & (FLAG_CAPTURE | FLAG_PROMO) != 0
    }

    /// Piece type promoted to (only meaningful if `is_promotion`).
    #[inline(always)]
    pub const fn promo_piece(self) -> usize {
        KNIGHT + (self.flag() & 3) as usize
    }

    pub fn to_uci(self) -> String {
        if self.is_none() {
            return "0000".to_string();
        }
        let mut s = format!("{}{}", square_name(self.from()), square_name(self.to()));
        if self.is_promotion() {
            s.push(b"nbrq"[self.promo_piece() - KNIGHT] as char);
        }
        s
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_uci())
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_uci())
    }
}

/// Fixed-capacity move list (no heap allocation). 218 is the most legal
/// moves known in any chess position.
pub struct MoveList {
    moves: [Move; 256],
    len: usize,
}

impl MoveList {
    #[inline(always)]
    pub fn new() -> Self {
        MoveList { moves: [Move::NONE; 256], len: 0 }
    }

    #[inline(always)]
    pub fn push(&mut self, m: Move) {
        self.moves[self.len] = m;
        self.len += 1;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Move> {
        self.as_slice().iter()
    }

    pub fn contains(&self, m: Move) -> bool {
        self.as_slice().contains(&m)
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = Move;
    #[inline(always)]
    fn index(&self, i: usize) -> &Move {
        &self.moves[i]
    }
}

/// Iterate over the set squares of a bitboard.
pub struct Bits(pub Bitboard);

impl Iterator for Bits {
    type Item = Square;
    #[inline(always)]
    fn next(&mut self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            let sq = self.0.trailing_zeros() as Square;
            self.0 &= self.0 - 1;
            Some(sq)
        }
    }
}

#[inline(always)]
pub const fn bb(sq: Square) -> Bitboard {
    1u64 << sq
}

pub const FILE_A: Bitboard = 0x0101_0101_0101_0101;
pub const FILE_H: Bitboard = FILE_A << 7;
pub const RANK_1: Bitboard = 0xFF;
pub const RANK_2: Bitboard = RANK_1 << 8;
pub const RANK_4: Bitboard = RANK_1 << 24;
pub const RANK_5: Bitboard = RANK_1 << 32;
pub const RANK_7: Bitboard = RANK_1 << 48;
pub const RANK_8: Bitboard = RANK_1 << 56;
