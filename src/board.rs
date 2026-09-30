//! Board representation: bitboards plus a square-indexed mailbox.
//! Moves are applied with copy-make: `make_move` returns a new board.

use crate::attacks;
use crate::types::*;
use crate::zobrist::KEYS;

pub const NO_SQ: u8 = 64;
pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Castling rights kept when a piece moves from or to a square.
const fn castle_masks() -> [u8; 64] {
    let mut m = [15u8; 64];
    m[0] = 15 & !CASTLE_WQ;
    m[4] = 15 & !(CASTLE_WK | CASTLE_WQ);
    m[7] = 15 & !CASTLE_WK;
    m[56] = 15 & !CASTLE_BQ;
    m[60] = 15 & !(CASTLE_BK | CASTLE_BQ);
    m[63] = 15 & !CASTLE_BK;
    m
}
static CASTLE_MASK: [u8; 64] = castle_masks();

/// Piece values used by static exchange evaluation (SEE).
pub const SEE_VALUES: [i32; 6] = [100, 300, 300, 500, 900, 0];

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Board {
    pub pieces: [Bitboard; 6],
    pub colors: [Bitboard; 2],
    pub mailbox: [Piece; 64],
    pub stm: usize,
    pub castling: u8,
    pub ep: u8,
    pub halfmove: u16,
    pub fullmove: u16,
    pub hash: u64,
    pub checkers: Bitboard,
}

impl Board {
    pub fn startpos() -> Board {
        Board::from_fen(START_FEN).unwrap()
    }

    fn empty() -> Board {
        Board {
            pieces: [0; 6],
            colors: [0; 2],
            mailbox: [NO_PIECE; 64],
            stm: WHITE,
            castling: 0,
            ep: NO_SQ,
            halfmove: 0,
            fullmove: 1,
            hash: 0,
            checkers: 0,
        }
    }

    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let mut b = Board::empty();
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(format!("bad FEN: {fen}"));
        }
        let mut rank = 7i32;
        let mut file = 0i32;
        for c in parts[0].chars() {
            match c {
                '/' => {
                    rank -= 1;
                    file = 0;
                }
                '1'..='8' => file += c as i32 - '0' as i32,
                _ => {
                    let p = char_piece(c).ok_or(format!("bad piece '{c}' in FEN"))?;
                    if !(0..8).contains(&rank) || !(0..8).contains(&file) {
                        return Err(format!("bad FEN board: {fen}"));
                    }
                    b.add_piece(p, (rank * 8 + file) as Square);
                    file += 1;
                }
            }
        }
        b.stm = match parts[1] {
            "w" => WHITE,
            "b" => BLACK,
            _ => return Err(format!("bad side to move in FEN: {fen}")),
        };
        if let Some(c) = parts.get(2) {
            for ch in c.chars() {
                b.castling |= match ch {
                    'K' => CASTLE_WK,
                    'Q' => CASTLE_WQ,
                    'k' => CASTLE_BK,
                    'q' => CASTLE_BQ,
                    _ => 0,
                };
            }
        }
        // Drop castling rights that the piece placement makes impossible.
        let rook_ok = |sq: Square, p: Piece| b.mailbox[sq] == p;
        if !(rook_ok(4, make_piece(WHITE, KING)) && rook_ok(7, make_piece(WHITE, ROOK))) {
            b.castling &= !CASTLE_WK;
        }
        if !(rook_ok(4, make_piece(WHITE, KING)) && rook_ok(0, make_piece(WHITE, ROOK))) {
            b.castling &= !CASTLE_WQ;
        }
        if !(rook_ok(60, make_piece(BLACK, KING)) && rook_ok(63, make_piece(BLACK, ROOK))) {
            b.castling &= !CASTLE_BK;
        }
        if !(rook_ok(60, make_piece(BLACK, KING)) && rook_ok(56, make_piece(BLACK, ROOK))) {
            b.castling &= !CASTLE_BQ;
        }
        if let Some(e) = parts.get(3) {
            if let Some(sq) = parse_square(e) {
                // Only keep the en-passant square if a capture is possible;
                // this keeps hashes of identical positions identical.
                let us = b.stm;
                if attacks::pawn(us ^ 1, sq) & b.pieces[PAWN] & b.colors[us] != 0 {
                    b.ep = sq as u8;
                }
            }
        }
        b.halfmove = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
        b.fullmove = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(1).max(1);

        for c in [WHITE, BLACK] {
            if (b.pieces[KING] & b.colors[c]).count_ones() != 1 {
                return Err("each side needs exactly one king".to_string());
            }
        }
        b.hash = b.compute_hash();
        b.checkers = b.attackers_to(b.king_sq(b.stm), b.occupied()) & b.colors[b.stm ^ 1];
        if b.attackers_to(b.king_sq(b.stm ^ 1), b.occupied()) & b.colors[b.stm] != 0 {
            return Err("side not to move is in check".to_string());
        }
        Ok(b)
    }

    pub fn to_fen(&self) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let p = self.mailbox[rank * 8 + file];
                if p == NO_PIECE {
                    empty += 1;
                } else {
                    if empty > 0 {
                        s.push_str(&empty.to_string());
                        empty = 0;
                    }
                    s.push(piece_char(p));
                }
            }
            if empty > 0 {
                s.push_str(&empty.to_string());
            }
            if rank > 0 {
                s.push('/');
            }
        }
        s.push_str(if self.stm == WHITE { " w " } else { " b " });
        if self.castling == 0 {
            s.push('-');
        } else {
            for (bit, c) in [(CASTLE_WK, 'K'), (CASTLE_WQ, 'Q'), (CASTLE_BK, 'k'), (CASTLE_BQ, 'q')] {
                if self.castling & bit != 0 {
                    s.push(c);
                }
            }
        }
        s.push(' ');
        if self.ep == NO_SQ {
            s.push('-');
        } else {
            s.push_str(&square_name(self.ep as Square));
        }
        s.push_str(&format!(" {} {}", self.halfmove, self.fullmove));
        s
    }

    pub fn compute_hash(&self) -> u64 {
        let mut h = 0;
        for sq in 0..64 {
            let p = self.mailbox[sq];
            if p != NO_PIECE {
                h ^= KEYS.pieces[p as usize][sq];
            }
        }
        h ^= KEYS.castling[self.castling as usize];
        if self.ep != NO_SQ {
            h ^= KEYS.ep_file[file_of(self.ep as Square)];
        }
        if self.stm == BLACK {
            h ^= KEYS.black_to_move;
        }
        h
    }

    #[inline(always)]
    fn add_piece(&mut self, p: Piece, sq: Square) {
        let b = bb(sq);
        self.pieces[piece_type(p)] |= b;
        self.colors[piece_color(p)] |= b;
        self.mailbox[sq] = p;
        self.hash ^= KEYS.pieces[p as usize][sq];
    }

    #[inline(always)]
    fn remove_piece(&mut self, p: Piece, sq: Square) {
        let b = bb(sq);
        self.pieces[piece_type(p)] ^= b;
        self.colors[piece_color(p)] ^= b;
        self.mailbox[sq] = NO_PIECE;
        self.hash ^= KEYS.pieces[p as usize][sq];
    }

    #[inline(always)]
    fn move_piece(&mut self, p: Piece, from: Square, to: Square) {
        let b = bb(from) | bb(to);
        self.pieces[piece_type(p)] ^= b;
        self.colors[piece_color(p)] ^= b;
        self.mailbox[from] = NO_PIECE;
        self.mailbox[to] = p;
        self.hash ^= KEYS.pieces[p as usize][from] ^ KEYS.pieces[p as usize][to];
    }

    #[inline(always)]
    pub fn occupied(&self) -> Bitboard {
        self.colors[WHITE] | self.colors[BLACK]
    }

    #[inline(always)]
    pub fn piece_bb(&self, color: usize, pt: usize) -> Bitboard {
        self.pieces[pt] & self.colors[color]
    }

    #[inline(always)]
    pub fn king_sq(&self, color: usize) -> Square {
        self.piece_bb(color, KING).trailing_zeros() as Square
    }

    #[inline(always)]
    pub fn piece_at(&self, sq: Square) -> Piece {
        self.mailbox[sq]
    }

    #[inline(always)]
    pub fn in_check(&self) -> bool {
        self.checkers != 0
    }

    /// All pieces (of both colours) attacking `sq` given occupancy `occ`.
    #[inline(always)]
    pub fn attackers_to(&self, sq: Square, occ: Bitboard) -> Bitboard {
        let rooks = self.pieces[ROOK] | self.pieces[QUEEN];
        let bishops = self.pieces[BISHOP] | self.pieces[QUEEN];
        (attacks::pawn(WHITE, sq) & self.piece_bb(BLACK, PAWN))
            | (attacks::pawn(BLACK, sq) & self.piece_bb(WHITE, PAWN))
            | (attacks::knight(sq) & self.pieces[KNIGHT])
            | (attacks::king(sq) & self.pieces[KING])
            | (attacks::rook(sq, occ) & rooks)
            | (attacks::bishop(sq, occ) & bishops)
    }

    /// Is `sq` attacked by any piece of colour `by`?
    #[inline(always)]
    pub fn is_attacked(&self, sq: Square, by: usize, occ: Bitboard) -> bool {
        let them = self.colors[by];
        (attacks::knight(sq) & self.pieces[KNIGHT] & them) != 0
            || (attacks::pawn(by ^ 1, sq) & self.pieces[PAWN] & them) != 0
            || (attacks::king(sq) & self.pieces[KING] & them) != 0
            || (attacks::rook(sq, occ) & (self.pieces[ROOK] | self.pieces[QUEEN]) & them) != 0
            || (attacks::bishop(sq, occ) & (self.pieces[BISHOP] | self.pieces[QUEEN]) & them) != 0
    }

    /// Piece captured by `m` (the pawn for en passant), or NO_PIECE.
    #[inline(always)]
    pub fn captured_piece(&self, m: Move) -> Piece {
        if m.is_ep() {
            make_piece(self.stm ^ 1, PAWN)
        } else if m.is_capture() {
            self.mailbox[m.to()]
        } else {
            NO_PIECE
        }
    }

    pub fn make_move(&self, m: Move) -> Board {
        let mut b = *self;
        let us = self.stm;
        let them = us ^ 1;
        let (from, to, flag) = (m.from(), m.to(), m.flag());
        let piece = self.mailbox[from];
        let pt = piece_type(piece);

        b.halfmove += 1;
        if b.ep != NO_SQ {
            b.hash ^= KEYS.ep_file[file_of(b.ep as Square)];
            b.ep = NO_SQ;
        }

        if m.is_capture() {
            let cap_sq = if flag == FLAG_EP { to ^ 8 } else { to };
            b.remove_piece(self.mailbox[cap_sq], cap_sq);
            b.halfmove = 0;
        }

        b.move_piece(piece, from, to);

        if pt == PAWN {
            b.halfmove = 0;
            if m.is_promotion() {
                b.remove_piece(piece, to);
                b.add_piece(make_piece(us, m.promo_piece()), to);
            } else if flag == FLAG_DOUBLE_PUSH {
                let ep = (from + to) / 2;
                if attacks::pawn(us, ep) & b.piece_bb(them, PAWN) != 0 {
                    b.ep = ep as u8;
                    b.hash ^= KEYS.ep_file[file_of(ep)];
                }
            }
        } else if flag == FLAG_KING_CASTLE {
            b.move_piece(make_piece(us, ROOK), to + 1, to - 1);
        } else if flag == FLAG_QUEEN_CASTLE {
            b.move_piece(make_piece(us, ROOK), to - 2, to + 1);
        }

        let new_rights = b.castling & CASTLE_MASK[from] & CASTLE_MASK[to];
        if new_rights != b.castling {
            b.hash ^= KEYS.castling[b.castling as usize] ^ KEYS.castling[new_rights as usize];
            b.castling = new_rights;
        }

        b.stm = them;
        b.hash ^= KEYS.black_to_move;
        if us == BLACK {
            b.fullmove += 1;
        }
        b.checkers = b.attackers_to(b.king_sq(them), b.occupied()) & b.colors[us];
        b
    }

    /// Pass the turn (used by null-move pruning). Must not be called in check.
    pub fn make_null(&self) -> Board {
        let mut b = *self;
        if b.ep != NO_SQ {
            b.hash ^= KEYS.ep_file[file_of(b.ep as Square)];
            b.ep = NO_SQ;
        }
        b.stm ^= 1;
        b.hash ^= KEYS.black_to_move;
        b.halfmove += 1;
        b.checkers = 0;
        b
    }

    /// Hash of the position after `m`, computed cheaply (ignores castling
    /// and en-passant changes). Used only to prefetch the hash table.
    #[inline(always)]
    pub fn approx_key_after(&self, m: Move) -> u64 {
        let p = self.mailbox[m.from()] as usize;
        let mut h = self.hash ^ KEYS.black_to_move ^ KEYS.pieces[p][m.from()] ^ KEYS.pieces[p][m.to()];
        let cap = self.mailbox[m.to()];
        if cap != NO_PIECE {
            h ^= KEYS.pieces[cap as usize][m.to()];
        }
        h
    }

    /// Does the side to move have any piece other than pawns and king?
    #[inline(always)]
    pub fn has_non_pawn_material(&self, color: usize) -> bool {
        self.colors[color] & !(self.pieces[PAWN] | self.pieces[KING]) != 0
    }

    /// Draw by insufficient material (K v K, K+minor v K).
    pub fn insufficient_material(&self) -> bool {
        if self.pieces[PAWN] | self.pieces[ROOK] | self.pieces[QUEEN] != 0 {
            return false;
        }
        (self.pieces[KNIGHT] | self.pieces[BISHOP]).count_ones() <= 1
    }

    /// Static exchange evaluation: does `m` win at least `threshold`
    /// centipawns once all exchanges on the target square are played out?
    pub fn see_ge(&self, m: Move, threshold: i32) -> bool {
        if m.is_castle() || m.is_promotion() {
            return threshold <= 0;
        }
        let (from, to) = (m.from(), m.to());
        let captured = self.captured_piece(m);
        let mut swap = if captured == NO_PIECE { 0 } else { SEE_VALUES[piece_type(captured)] } - threshold;
        if swap < 0 {
            return false;
        }
        swap = SEE_VALUES[piece_type(self.mailbox[from])] - swap;
        if swap <= 0 {
            return true;
        }

        let mut occ = self.occupied() ^ bb(from) ^ bb(to);
        if m.is_ep() {
            occ ^= bb(to ^ 8);
        }
        let diag = self.pieces[BISHOP] | self.pieces[QUEEN];
        let orth = self.pieces[ROOK] | self.pieces[QUEEN];
        let mut attackers = self.attackers_to(to, occ);
        let mut side = self.stm;
        let mut result = true;

        loop {
            side ^= 1;
            attackers &= occ;
            let ours = attackers & self.colors[side];
            if ours == 0 {
                break;
            }
            result = !result;

            // Capture with the least valuable attacker.
            let mut pt = PAWN;
            while ours & self.pieces[pt] == 0 {
                pt += 1;
            }
            if pt == KING {
                // The king may only recapture if the square is no longer defended.
                if attackers & self.colors[side ^ 1] != 0 {
                    result = !result;
                }
                break;
            }
            swap = SEE_VALUES[pt] - swap;
            if swap < result as i32 {
                break;
            }
            let lsb = ours & self.pieces[pt];
            occ ^= lsb & lsb.wrapping_neg();
            if pt == PAWN || pt == BISHOP || pt == QUEEN {
                attackers |= attacks::bishop(to, occ) & diag;
            }
            if pt == ROOK || pt == QUEEN {
                attackers |= attacks::rook(to, occ) & orth;
            }
        }
        result
    }

    /// Parse a move in UCI notation (e.g. "e2e4", "e7e8q") against the
    /// legal moves of this position.
    pub fn parse_uci_move(&self, s: &str) -> Option<Move> {
        crate::movegen::legal_moves(self).iter().copied().find(|m| m.to_uci() == s)
    }

    /// Pretty-print the board for the `d` debug command.
    pub fn pretty(&self) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            s.push_str(&format!(" {} ", rank + 1));
            for file in 0..8 {
                s.push(' ');
                s.push(piece_char(self.mailbox[rank * 8 + file]));
            }
            s.push('\n');
        }
        s.push_str("    a b c d e f g h\n\n");
        s.push_str(&format!("FEN: {}\nKey: {:016x}\n", self.to_fen(), self.hash));
        s
    }
}
