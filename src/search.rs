//! Alpha-beta search.
//!
//! Overview for readers new to chess programming:
//! - *Iterative deepening*: search depth 1, then 2, 3, ... until time runs
//!   out; each iteration's results make the next one faster.
//! - *Principal variation search (PVS)*: assume the first move is best and
//!   prove the others worse with cheap "null window" searches.
//! - *Pruning / reductions*: skip or search less deeply the moves and
//!   positions that are very unlikely to matter (null move, futility,
//!   late-move reductions, ...). Every such idea must be confirmed by SPRT
//!   testing before it is kept.
//! - *Quiescence search*: at the end of the main search, keep looking at
//!   captures so we never evaluate a position in the middle of a trade.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use crate::board::Board;
use crate::eval;
use crate::movegen;
use crate::nnue::{self, AccPair, Network};
use crate::tt::*;
use crate::types::*;

pub const MAX_PLY: usize = 128;
pub const INF: i32 = 32_000;
pub const MATE: i32 = 31_000;
/// Scores beyond this are "mate in N".
pub const MATE_BOUND: i32 = MATE - 2 * MAX_PLY as i32;
/// Tablebase win (minus the ply), kept below the mate range.
pub const TB_WIN: i32 = MATE_BOUND - 1 - MAX_PLY as i32;
/// Scores beyond this are tablebase wins or mates (adjusted by ply in the TT).
pub const TB_BOUND: i32 = TB_WIN - MAX_PLY as i32;

/// What the GUI (or datagen/bench) asked for.
#[derive(Clone, Default)]
pub struct Limits {
    pub depth: Option<i32>,
    /// Hard node limit (`go nodes`).
    pub nodes: Option<u64>,
    /// Soft node limit: finish the current iteration, then stop (datagen).
    pub soft_nodes: Option<u64>,
    pub movetime: Option<u64>,
    pub time: Option<u64>,
    pub inc: u64,
    pub movestogo: Option<u64>,
    pub infinite: bool,
    pub move_overhead: u64,
}

/// State shared by all search threads.
pub struct Shared {
    pub tt: TranspositionTable,
    pub stop: AtomicBool,
    pub nodes: AtomicU64,
}

impl Shared {
    pub fn new(hash_mb: usize) -> Arc<Shared> {
        Arc::new(Shared {
            tt: TranspositionTable::new(hash_mb),
            stop: AtomicBool::new(false),
            nodes: AtomicU64::new(0),
        })
    }
}

#[derive(Clone, Copy)]
struct Frame {
    static_eval: i32,
    /// Move played from this ply (NONE for a null move) and the piece moved.
    mv: Move,
    piece: Piece,
}

impl Default for Frame {
    fn default() -> Self {
        Frame { static_eval: -INF, mv: Move::NONE, piece: NO_PIECE }
    }
}

type History = [[[i16; 64]; 64]; 2]; // [colour][from][to]
type ContHistory = [[[[i16; 64]; 12]; 64]; 12]; // [prev piece][prev to][piece][to]
type CaptHistory = [[[i16; 6]; 64]; 12]; // [piece][to][captured type]

/// Heap-allocate a zeroed value (these tables are too big for the stack).
fn zeroed_box<T>() -> Box<T> {
    unsafe {
        let layout = std::alloc::Layout::new::<T>();
        let ptr = std::alloc::alloc_zeroed(layout) as *mut T;
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        Box::from_raw(ptr)
    }
}

const HISTORY_MAX: i32 = 16_384;

/// Correction history: per side to move and pawn structure, a running
/// average of how far the search result differed from the static eval.
/// Values are stored in units of 1/CORR_GRAIN centipawns.
const CORR_SIZE: usize = 16_384;
const CORR_GRAIN: i32 = 256;
const CORR_WEIGHT_SCALE: i32 = 256;
const CORR_MAX: i32 = CORR_GRAIN * 32;

/// Nudge a history value towards `bonus` while keeping it in range.
#[inline(always)]
fn apply_bonus(entry: &mut i16, bonus: i32) {
    let e = i32::from(*entry);
    *entry = (e + bonus - e * bonus.abs() / HISTORY_MAX) as i16;
}

#[inline(always)]
fn history_bonus(depth: i32) -> i32 {
    (170 * depth - 80).min(1700)
}

/// The moves of one node with their ordering scores. Scores are only
/// written for existing moves, so nothing is cleared up front.
struct ScoredMoves {
    list: MoveList,
    scores: [std::mem::MaybeUninit<i32>; 256],
}

impl ScoredMoves {
    #[inline(always)]
    fn new(list: MoveList) -> Self {
        ScoredMoves { list, scores: [std::mem::MaybeUninit::uninit(); 256] }
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.list.len()
    }

    #[inline(always)]
    fn score(&self, i: usize) -> i32 {
        debug_assert!(i < self.len());
        // SAFETY: `score_moves` writes a score for every move in the list.
        unsafe { self.scores[i].assume_init() }
    }

    /// Selection sort step: bring the best remaining move to position `i`.
    #[inline(always)]
    fn pick(&mut self, i: usize) -> Move {
        let mut best_i = i;
        for j in i + 1..self.len() {
            if self.score(j) > self.score(best_i) {
                best_i = j;
            }
        }
        self.scores.swap(i, best_i);
        self.list.swap(i, best_i);
        self.list[i]
    }
}

pub struct SearchResult {
    pub best_move: Move,
    pub score: i32,
}

/// One search thread's private state.
pub struct Searcher {
    pub shared: Arc<Shared>,
    pub id: usize,
    /// Suppress UCI output (bench, datagen, helper threads).
    pub silent: bool,
    pub nodes: u64,
    unflushed: u64,
    seldepth: usize,
    limits: Limits,
    start: Instant,
    soft_ms: u64,
    hard_ms: u64,
    root_depth: i32,

    history: Box<History>,
    cont_hist: Box<ContHistory>,
    capt_hist: Box<CaptHistory>,
    corr_hist: Box<[[i32; CORR_SIZE]; 2]>,
    killers: [[Move; 2]; MAX_PLY + 2],
    counters: Box<[[Move; 64]; 12]>,
    frames: [Frame; MAX_PLY + 2],
    /// Hashes of earlier positions (game + search path) for repetition checks.
    pub keys: Vec<u64>,
    pv: Box<[[Move; MAX_PLY + 1]; MAX_PLY + 1]>,
    pv_len: [usize; MAX_PLY + 1],
    net: Option<&'static Network>,
    /// Accumulators for each ply of the current search path.
    acc: Box<[AccPair; MAX_PLY + 2]>,
    lmr: Box<[[[i32; 64]; 64]; 2]>,
    /// Nodes spent below each root move (by from/to), for time management.
    root_nodes: Box<[[u64; 64]; 64]>,
}

impl Searcher {
    pub fn new(shared: Arc<Shared>, id: usize) -> Searcher {
        let mut lmr: Box<[[[i32; 64]; 64]; 2]> = zeroed_box();
        for d in 1..64 {
            for m in 1..64 {
                let l = (d as f64).ln() * (m as f64).ln();
                lmr[0][d][m] = (0.20 + l / 3.35) as i32; // noisy moves
                lmr[1][d][m] = (0.80 + l / 2.25) as i32; // quiet moves
            }
        }
        let net = nnue::network();
        Searcher {
            shared,
            id,
            silent: id != 0,
            nodes: 0,
            unflushed: 0,
            seldepth: 0,
            limits: Limits::default(),
            start: Instant::now(),
            soft_ms: u64::MAX,
            hard_ms: u64::MAX,
            root_depth: 0,
            history: zeroed_box(),
            cont_hist: zeroed_box(),
            capt_hist: zeroed_box(),
            corr_hist: zeroed_box(),
            killers: [[Move::NONE; 2]; MAX_PLY + 2],
            counters: zeroed_box(),
            frames: [Frame::default(); MAX_PLY + 2],
            keys: Vec::with_capacity(1024),
            pv: zeroed_box(),
            pv_len: [0; MAX_PLY + 1],
            net,
            acc: zeroed_box(),
            lmr,
            root_nodes: zeroed_box(),
        }
    }

    /// Forget everything learned (UCI `ucinewgame`).
    pub fn clear(&mut self) {
        self.history = zeroed_box();
        self.cont_hist = zeroed_box();
        self.capt_hist = zeroed_box();
        self.corr_hist = zeroed_box();
        self.counters = zeroed_box();
        self.killers = [[Move::NONE; 2]; MAX_PLY + 2];
    }

    fn setup_time(&mut self, stm_time: Option<u64>) {
        let l = &self.limits;
        self.soft_ms = u64::MAX;
        self.hard_ms = u64::MAX;
        if let Some(mt) = l.movetime {
            // Fixed time per move: use all of it (only the hard limit applies).
            self.hard_ms = mt.saturating_sub(l.move_overhead).max(1);
        } else if let Some(time) = stm_time {
            let t = time.saturating_sub(l.move_overhead).max(1);
            let base = match l.movestogo {
                Some(mtg) => t / (mtg.clamp(1, 50) + 1) + l.inc * 3 / 4,
                None => t / 25 + l.inc * 3 / 4,
            };
            self.soft_ms = base.min(t / 2).max(1);
            self.hard_ms = (base * 3).min(t * 3 / 5).max(1);
        }
    }

    fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Called at every node. Flushes node counts and, on the main thread,
    /// enforces the hard time and node limits.
    #[inline(always)]
    fn should_stop(&mut self) -> bool {
        self.nodes += 1;
        self.unflushed += 1;
        if self.unflushed >= 1024 {
            self.shared.nodes.fetch_add(self.unflushed, Ordering::Relaxed);
            self.unflushed = 0;
            if self.id == 0 && self.root_depth > 1 {
                let over_time = self.elapsed_ms() >= self.hard_ms;
                let over_nodes = self.limits.nodes.is_some_and(|n| self.shared.nodes.load(Ordering::Relaxed) >= n);
                if over_time || over_nodes {
                    self.shared.stop.store(true, Ordering::Relaxed);
                }
            }
        }
        self.root_depth > 1 && self.shared.stop.load(Ordering::Relaxed)
    }

    fn flush_nodes(&mut self) {
        self.shared.nodes.fetch_add(self.unflushed, Ordering::Relaxed);
        self.unflushed = 0;
    }

    #[inline(always)]
    fn stopped(&self) -> bool {
        self.root_depth > 1 && self.shared.stop.load(Ordering::Relaxed)
    }

    fn evaluate(&self, b: &Board, ply: usize) -> i32 {
        let raw = match self.net {
            Some(net) => nnue::evaluate(net, &self.acc[ply], b),
            None => eval::evaluate(b),
        };
        // Drift towards a draw as the fifty-move counter grows.
        let scaled = raw * (200 - i32::from(b.halfmove)) / 200;
        scaled.clamp(-TB_BOUND + 1, TB_BOUND - 1)
    }

    /// Static eval adjusted by the correction history for this pawn structure.
    #[inline(always)]
    fn corrected_eval(&self, b: &Board, raw: i32) -> i32 {
        let c = self.corr_hist[b.stm][b.pawn_key as usize & (CORR_SIZE - 1)];
        (raw + c / CORR_GRAIN).clamp(-TB_BOUND + 1, TB_BOUND - 1)
    }

    fn update_correction(&mut self, b: &Board, depth: i32, diff: i32) {
        let entry = &mut self.corr_hist[b.stm][b.pawn_key as usize & (CORR_SIZE - 1)];
        let weight = (depth + 1).min(16);
        let target = diff * CORR_GRAIN;
        *entry =
            ((*entry * (CORR_WEIGHT_SCALE - weight) + target * weight) / CORR_WEIGHT_SCALE).clamp(-CORR_MAX, CORR_MAX);
    }

    fn is_repetition(&self, b: &Board) -> bool {
        let n = self.keys.len();
        let limit = (b.halfmove as usize).min(n);
        let mut i = 2;
        while i <= limit {
            if self.keys[n - i] == b.hash {
                return true;
            }
            i += 2;
        }
        false
    }

    #[inline(always)]
    fn make(&mut self, b: &Board, m: Move, ply: usize) -> Board {
        self.shared.tt.prefetch(b.approx_key_after(m));
        self.keys.push(b.hash);
        self.frames[ply].mv = m;
        self.frames[ply].piece = b.piece_at(m.from());
        if let Some(net) = self.net {
            let (done, rest) = self.acc.split_at_mut(ply + 1);
            done[ply].update_into(&mut rest[0], net, b, m);
        }
        b.make_move(m)
    }

    #[inline(always)]
    fn unmake(&mut self) {
        self.keys.pop();
    }

    fn cont_score(&self, ply: usize, back: usize, piece: Piece, to: Square) -> i32 {
        if ply < back {
            return 0;
        }
        let f = &self.frames[ply - back];
        if f.piece == NO_PIECE {
            return 0;
        }
        i32::from(self.cont_hist[f.piece as usize][f.mv.to()][piece as usize][to])
    }

    fn quiet_score(&self, b: &Board, m: Move, ply: usize) -> i32 {
        let piece = b.piece_at(m.from());
        i32::from(self.history[b.stm][m.from()][m.to()])
            + self.cont_score(ply, 1, piece, m.to())
            + self.cont_score(ply, 2, piece, m.to())
    }

    fn update_quiet_histories(&mut self, b: &Board, ply: usize, best: Move, tried: &[Move], depth: i32) {
        let bonus = history_bonus(depth);
        for &m in tried.iter().chain(std::iter::once(&best)) {
            let delta = if m == best { bonus } else { -bonus };
            let piece = b.piece_at(m.from());
            apply_bonus(&mut self.history[b.stm][m.from()][m.to()], delta);
            for back in [1, 2] {
                if ply >= back {
                    let f = self.frames[ply - back];
                    if f.piece != NO_PIECE {
                        apply_bonus(&mut self.cont_hist[f.piece as usize][f.mv.to()][piece as usize][m.to()], delta);
                    }
                }
            }
        }
        if self.killers[ply][0] != best {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = best;
        }
        if ply > 0 {
            let f = self.frames[ply - 1];
            if f.piece != NO_PIECE {
                self.counters[f.piece as usize][f.mv.to()] = best;
            }
        }
    }

    fn capture_entry(&mut self, b: &Board, m: Move) -> &mut i16 {
        let piece = b.piece_at(m.from()) as usize;
        let cap = b.captured_piece(m);
        let ct = if cap == NO_PIECE { PAWN } else { piece_type(cap) };
        &mut self.capt_hist[piece][m.to()][ct]
    }

    /// Assign every move an ordering score (higher = searched earlier).
    fn score_moves(&self, b: &Board, moves: &mut ScoredMoves, tt_move: Move, ply: usize) {
        let counter = if ply > 0 {
            let f = self.frames[ply - 1];
            if f.piece != NO_PIECE { self.counters[f.piece as usize][f.mv.to()] } else { Move::NONE }
        } else {
            Move::NONE
        };
        for i in 0..moves.len() {
            let m = moves.list[i];
            let score = if m == tt_move {
                2_000_000_000
            } else if m.is_noisy() {
                let cap = b.captured_piece(m);
                let victim = if cap == NO_PIECE { 0 } else { crate::board::SEE_VALUES[piece_type(cap)] };
                let piece = b.piece_at(m.from()) as usize;
                let ct = if cap == NO_PIECE { PAWN } else { piece_type(cap) };
                let hist = i32::from(self.capt_hist[piece][m.to()][ct]);
                let promo = if m.is_promotion() && m.promo_piece() == QUEEN { 1000 } else { 0 };
                let base = if b.see_ge(m, -hist / 32) { 1_000_000_000 } else { -1_000_000_000 };
                base + 16 * (victim + promo) + hist
            } else if m == self.killers[ply][0] {
                900_000_000
            } else if m == self.killers[ply][1] {
                800_000_000
            } else if m == counter {
                700_000_000
            } else {
                self.quiet_score(b, m, ply)
            };
            moves.scores[i].write(score);
        }
    }

    fn update_pv(&mut self, ply: usize, m: Move) {
        self.pv[ply][ply] = m;
        let child_len = self.pv_len[ply + 1];
        for i in ply + 1..child_len {
            self.pv[ply][i] = self.pv[ply + 1][i];
        }
        self.pv_len[ply] = child_len.max(ply + 1);
    }

    fn negamax(
        &mut self,
        b: &Board,
        mut alpha: i32,
        mut beta: i32,
        mut depth: i32,
        ply: usize,
        cut_node: bool,
        excluded: Move,
    ) -> i32 {
        let pv_node = beta - alpha > 1;
        let root = ply == 0;
        let in_check = b.in_check();
        self.pv_len[ply] = ply;

        // Check extension: never stop searching while in check.
        if in_check {
            depth = depth.max(0) + 1;
        }
        if depth <= 0 {
            return self.qsearch(b, alpha, beta, ply);
        }
        if self.should_stop() {
            return 0;
        }
        self.seldepth = self.seldepth.max(ply);

        if !root {
            if b.halfmove >= 100 || b.insufficient_material() || self.is_repetition(b) {
                // Tiny randomisation helps avoid repetition blindness.
                return 1 - (self.nodes & 2) as i32;
            }
            if ply >= MAX_PLY - 1 {
                return if in_check { 0 } else { self.evaluate(b, ply) };
            }
            // Mate distance pruning: no line can beat a mate we already have.
            alpha = alpha.max(-MATE + ply as i32);
            beta = beta.min(MATE - ply as i32 - 1);
            if alpha >= beta {
                return alpha;
            }
        }

        let tt_entry = if excluded.is_none() { self.shared.tt.probe(b.hash) } else { None };
        let tt_move = tt_entry.map_or(Move::NONE, |e| e.mv);
        if let Some(e) = tt_entry {
            let s = score_from_tt(i32::from(e.score), ply);
            if !pv_node
                && i32::from(e.depth) >= depth
                && (e.bound == BOUND_EXACT
                    || (e.bound == BOUND_LOWER && s >= beta)
                    || (e.bound == BOUND_UPPER && s <= alpha))
            {
                return s;
            }
        }

        // Endgame tablebases: exact result right after a capture or pawn
        // move. Stored in the TT so that revisits cost no file lookup.
        if !root && excluded.is_none() && b.occupied().count_ones() <= crate::tb::max_pieces() {
            if let Some(wdl) = crate::tb::probe_wdl(b) {
                let (score, bound) = match wdl {
                    1 => (TB_WIN - ply as i32, BOUND_LOWER),
                    -1 => (-TB_WIN + ply as i32, BOUND_UPPER),
                    _ => (0, BOUND_EXACT),
                };
                let tb_depth = (depth + 6).min(MAX_PLY as i32 - 1);
                self.shared.tt.store(b.hash, Move::NONE, score_to_tt(score, ply), 0, tb_depth, bound);
                return score;
            }
        }

        // `raw_eval` (what the TT stores) is the network's opinion;
        // `static_eval` adds the learned correction for this pawn structure.
        let static_eval;
        let mut raw_eval = -INF;
        let mut eval;
        if in_check {
            static_eval = -INF;
            eval = -INF;
        } else if !excluded.is_none() {
            static_eval = self.frames[ply].static_eval;
            eval = static_eval;
        } else {
            raw_eval = match tt_entry {
                Some(e) => i32::from(e.eval),
                None => self.evaluate(b, ply),
            };
            static_eval = self.corrected_eval(b, raw_eval);
            eval = static_eval;
            if let Some(e) = tt_entry {
                let s = score_from_tt(i32::from(e.score), ply);
                if (e.bound == BOUND_LOWER && s > eval)
                    || (e.bound == BOUND_UPPER && s < eval)
                    || e.bound == BOUND_EXACT
                {
                    eval = s;
                }
            }
        }
        self.frames[ply].static_eval = static_eval;
        let improving = !in_check && ply >= 2 && static_eval > self.frames[ply - 2].static_eval;
        self.killers[ply + 1] = [Move::NONE; 2];

        if !pv_node && !in_check && excluded.is_none() {
            // Reverse futility pruning: we are so far ahead that even a
            // generous margin keeps us above beta.
            if depth <= 8 && eval.abs() < TB_BOUND && eval - 75 * (depth - i32::from(improving)) >= beta {
                return (eval + beta) / 2;
            }

            // Razoring: hopelessly behind; check if captures can save us.
            if depth <= 3 && eval + 200 + 250 * depth <= alpha {
                let q = self.qsearch(b, alpha, alpha + 1, ply);
                if q <= alpha {
                    return q;
                }
            }

            // Null move pruning: give the opponent a free move. If we are
            // still above beta, this node is almost surely a cut-off.
            if depth >= 3
                && eval >= beta
                && static_eval >= beta
                && ply > 0
                && self.frames[ply - 1].piece != NO_PIECE
                && b.has_non_pawn_material(b.stm)
            {
                let r = 3 + depth / 3 + ((eval - beta) / 200).min(3);
                let nb = b.make_null();
                self.keys.push(b.hash);
                self.frames[ply].mv = Move::NONE;
                self.frames[ply].piece = NO_PIECE;
                if self.net.is_some() {
                    self.acc[ply + 1] = self.acc[ply];
                }
                let score = -self.negamax(&nb, -beta, -beta + 1, depth - r, ply + 1, !cut_node, Move::NONE);
                self.keys.pop();
                if self.stopped() {
                    return 0;
                }
                if score >= beta {
                    return if score >= TB_BOUND { beta } else { score };
                }
            }
        }

        // Internal iterative reduction: without a hash move our ordering is
        // poor, so spend less effort here.
        if tt_move.is_none() && depth >= 4 && (pv_node || cut_node) {
            depth -= 1;
        }

        let mut list = MoveList::new();
        movegen::generate(b, &mut list, false);
        if list.is_empty() {
            return if in_check { -MATE + ply as i32 } else { 0 };
        }
        let mut moves = ScoredMoves::new(list);
        self.score_moves(b, &mut moves, tt_move, ply);

        let orig_alpha = alpha;
        let mut best_score = -INF;
        let mut best_move = Move::NONE;
        let mut moves_searched = 0usize;
        let mut skip_quiets = false;
        let mut quiets_tried = MoveList::new();
        let mut noisies_tried = MoveList::new();

        for i in 0..moves.len() {
            let m = moves.pick(i);
            if m == excluded {
                continue;
            }
            let quiet = !m.is_noisy();
            if quiet && skip_quiets {
                continue;
            }

            if !root && best_score > -TB_BOUND {
                let lmr_depth =
                    (depth - self.lmr[usize::from(quiet)][depth.min(63) as usize][moves_searched.min(63)]).max(0);
                if quiet {
                    // Late move pruning: after enough quiet moves, the rest
                    // are very unlikely to be good.
                    if moves_searched as i32 >= (3 + depth * depth) / (2 - i32::from(improving)) {
                        skip_quiets = true;
                        continue;
                    }
                    // Futility pruning: this quiet move can't lift us to alpha.
                    if !in_check && lmr_depth <= 8 && eval + 90 + 100 * lmr_depth <= alpha {
                        skip_quiets = true;
                        continue;
                    }
                    // SEE pruning: the move hangs material.
                    if lmr_depth <= 8 && !b.see_ge(m, -30 * lmr_depth * lmr_depth) {
                        continue;
                    }
                } else if depth <= 8 && !b.see_ge(m, -90 * depth) {
                    continue;
                }
            }

            // Singular extension: if the hash move is much better than every
            // alternative, search it deeper.
            let mut extension = 0;
            if let Some(e) = tt_entry {
                if !root
                    && m == tt_move
                    && excluded.is_none()
                    && depth >= 8
                    && i32::from(e.depth) >= depth - 3
                    && e.bound != BOUND_UPPER
                    && i32::from(e.score).abs() < TB_BOUND
                {
                    let tt_score = score_from_tt(i32::from(e.score), ply);
                    let s_beta = tt_score - depth;
                    let s_score = self.negamax(b, s_beta - 1, s_beta, (depth - 1) / 2, ply, cut_node, m);
                    if self.stopped() {
                        return 0;
                    }
                    if s_score < s_beta {
                        extension = 1;
                    } else if s_beta >= beta {
                        // Multi-cut: several moves beat beta.
                        return s_beta;
                    } else if tt_score >= beta {
                        extension = -1;
                    }
                }
            }

            let child = self.make(b, m, ply);
            let nodes_before = self.nodes;
            moves_searched += 1;
            let new_depth = depth - 1 + extension;
            let mut score;
            if moves_searched == 1 {
                score = -self.negamax(&child, -beta, -alpha, new_depth, ply + 1, !pv_node && !cut_node, Move::NONE);
            } else {
                // Late move reductions.
                let mut r = 0;
                if depth >= 3 && moves_searched > 1 + usize::from(root) {
                    r = self.lmr[usize::from(quiet)][depth.min(63) as usize][moves_searched.min(63)];
                    r -= i32::from(pv_node);
                    r += i32::from(cut_node);
                    r += i32::from(!improving);
                    r -= i32::from(child.in_check());
                    if quiet {
                        r -= self.quiet_score(b, m, ply) / 8192;
                        if m == self.killers[ply][0] || m == self.killers[ply][1] {
                            r -= 1;
                        }
                    }
                    r = r.clamp(0, (new_depth - 1).max(0));
                }
                score = -self.negamax(&child, -alpha - 1, -alpha, new_depth - r, ply + 1, true, Move::NONE);
                if score > alpha && r > 0 {
                    score = -self.negamax(&child, -alpha - 1, -alpha, new_depth, ply + 1, !cut_node, Move::NONE);
                }
                if score > alpha && pv_node {
                    score = -self.negamax(&child, -beta, -alpha, new_depth, ply + 1, false, Move::NONE);
                }
            }
            self.unmake();
            if root {
                self.root_nodes[m.from()][m.to()] += self.nodes - nodes_before;
            }
            if self.stopped() {
                return 0;
            }

            if score > best_score {
                best_score = score;
                if score > alpha {
                    best_move = m;
                    alpha = score;
                    if pv_node {
                        self.update_pv(ply, m);
                    }
                    if score >= beta {
                        break;
                    }
                }
            }
            if m != best_move {
                if quiet {
                    quiets_tried.push(m);
                } else {
                    noisies_tried.push(m);
                }
            }
        }

        if moves_searched == 0 {
            // Only possible when the single legal move was excluded.
            return if !excluded.is_none() {
                alpha
            } else if in_check {
                -MATE + ply as i32
            } else {
                0
            };
        }

        if best_score >= beta {
            let bonus = history_bonus(depth);
            if best_move.is_noisy() {
                apply_bonus(self.capture_entry(b, best_move), bonus);
            } else {
                self.update_quiet_histories(b, ply, best_move, quiets_tried.as_slice(), depth);
            }
            for &m in noisies_tried.iter() {
                apply_bonus(self.capture_entry(b, m), -bonus);
            }
        }

        if excluded.is_none() {
            let bound = if best_score >= beta {
                BOUND_LOWER
            } else if alpha > orig_alpha {
                BOUND_EXACT
            } else {
                BOUND_UPPER
            };
            // Learn how wrong the static eval was, when the search result is
            // trustworthy in that direction (quiet best move, bound agrees).
            if !in_check
                && !best_move.is_noisy()
                && best_score.abs() < TB_BOUND
                && !(bound == BOUND_LOWER && best_score <= static_eval)
                && !(bound == BOUND_UPPER && best_score >= static_eval)
            {
                self.update_correction(b, depth, best_score - static_eval);
            }
            self.shared.tt.store(b.hash, best_move, score_to_tt(best_score, ply), raw_eval, depth, bound);
        }
        best_score
    }

    fn qsearch(&mut self, b: &Board, mut alpha: i32, beta: i32, ply: usize) -> i32 {
        if self.should_stop() {
            return 0;
        }
        self.seldepth = self.seldepth.max(ply);
        self.pv_len[ply] = ply;
        let in_check = b.in_check();
        if ply >= MAX_PLY - 1 {
            return if in_check { 0 } else { self.evaluate(b, ply) };
        }
        let pv_node = beta - alpha > 1;

        let tt_entry = self.shared.tt.probe(b.hash);
        if let Some(e) = tt_entry {
            let s = score_from_tt(i32::from(e.score), ply);
            if !pv_node
                && (e.bound == BOUND_EXACT
                    || (e.bound == BOUND_LOWER && s >= beta)
                    || (e.bound == BOUND_UPPER && s <= alpha))
            {
                return s;
            }
        }

        let mut best;
        let static_eval;
        let mut raw_eval = -INF;
        if in_check {
            static_eval = -INF;
            best = -MATE + ply as i32;
        } else {
            raw_eval = match tt_entry {
                Some(e) => i32::from(e.eval),
                None => self.evaluate(b, ply),
            };
            static_eval = self.corrected_eval(b, raw_eval);
            best = static_eval;
            if let Some(e) = tt_entry {
                let s = score_from_tt(i32::from(e.score), ply);
                if (e.bound == BOUND_LOWER && s > best)
                    || (e.bound == BOUND_UPPER && s < best)
                    || e.bound == BOUND_EXACT
                {
                    best = s;
                }
            }
            // Stand pat: we may decline to capture anything.
            if best >= beta {
                return best;
            }
            alpha = alpha.max(best);
        }

        let mut list = MoveList::new();
        movegen::generate(b, &mut list, !in_check);
        let tt_move = tt_entry.map_or(Move::NONE, |e| e.mv);
        let mut moves = ScoredMoves::new(list);
        self.score_moves(b, &mut moves, tt_move, ply);

        let mut best_move = Move::NONE;
        let mut searched = 0;
        for i in 0..moves.len() {
            let m = moves.pick(i);

            if !in_check {
                // Skip captures that lose material or can't reach alpha.
                if !b.see_ge(m, 0) {
                    continue;
                }
                if !m.is_promotion() {
                    let cap = b.captured_piece(m);
                    let gain = crate::board::SEE_VALUES[piece_type(cap)];
                    if static_eval + 150 + gain <= alpha {
                        best = best.max(static_eval + 150 + gain);
                        continue;
                    }
                }
            }

            let child = self.make(b, m, ply);
            searched += 1;
            let score = -self.qsearch(&child, -beta, -alpha, ply + 1);
            self.unmake();
            if self.stopped() {
                return 0;
            }
            if score > best {
                best = score;
                if score > alpha {
                    alpha = score;
                    best_move = m;
                    if pv_node {
                        self.update_pv(ply, m);
                    }
                    if score >= beta {
                        break;
                    }
                }
            }
        }

        if in_check && searched == 0 {
            return -MATE + ply as i32; // checkmate: no evasions exist
        }

        let bound = if best >= beta { BOUND_LOWER } else { BOUND_UPPER };
        self.shared.tt.store(b.hash, best_move, score_to_tt(best, ply), raw_eval.max(-MATE_BOUND), 0, bound);
        best
    }

    /// Run iterative deepening on `root`. `history` holds the hashes of the
    /// positions played before `root` (for repetition detection).
    pub fn search(&mut self, root: &Board, history: &[u64], limits: &Limits) -> SearchResult {
        self.limits = limits.clone();
        self.start = Instant::now();
        let stm_time = limits.time;
        self.setup_time(stm_time);
        self.nodes = 0;
        self.unflushed = 0;
        self.keys.clear();
        self.keys.extend_from_slice(history);
        self.frames = [Frame::default(); MAX_PLY + 2];
        *self.root_nodes = [[0; 64]; 64];
        if let Some(net) = self.net {
            self.acc[0] = AccPair::from_board(net, root);
        }

        let legal = movegen::legal_moves(root);
        let mut result = SearchResult { best_move: legal.as_slice().first().copied().unwrap_or(Move::NONE), score: 0 };
        if legal.is_empty() {
            return result;
        }

        let max_depth = limits.depth.unwrap_or(MAX_PLY as i32 - 1).clamp(1, MAX_PLY as i32 - 1);
        let mut prev_score = 0;
        let mut stability = 0;
        for depth in 1..=max_depth {
            self.root_depth = depth;
            self.seldepth = 0;
            let mut delta = 20;
            let (mut alpha, mut beta) = if depth >= 5 { (prev_score - delta, prev_score + delta) } else { (-INF, INF) };
            let mut score;
            loop {
                score = self.negamax(root, alpha, beta, depth, 0, false, Move::NONE);
                if self.stopped() {
                    break;
                }
                if score <= alpha {
                    beta = (alpha + beta) / 2;
                    alpha = (score - delta).max(-INF);
                } else if score >= beta {
                    beta = (score + delta).min(INF);
                } else {
                    break;
                }
                delta += delta / 2;
            }
            if self.stopped() {
                break;
            }

            let best = if self.pv_len[0] > 0 { self.pv[0][0] } else { result.best_move };
            stability = if best == result.best_move { stability + 1 } else { 0 };
            let score_drop = if depth > 1 { prev_score - score } else { 0 };
            result = SearchResult { best_move: best, score };
            prev_score = score;

            if self.id == 0 {
                self.flush_nodes();
                if !self.silent {
                    self.print_info(depth, score);
                }
                // Soft limits: don't start an iteration we can't finish.
                // Think longer when the best move keeps changing, when the
                // search effort is spread over several moves, or when the
                // score just dropped; stop sooner when all signs agree.
                let stability_scale = [2.0, 1.4, 1.1, 0.9, 0.8][stability.min(4)];
                let node_scale = if depth >= 6 {
                    let best_nodes = self.root_nodes[best.from()][best.to()] as f64;
                    let fraction = best_nodes / self.nodes.max(1) as f64;
                    (1.5 - fraction) * 1.35
                } else {
                    1.0
                };
                let score_scale = (1.0 + f64::from(score_drop) * 0.01).clamp(0.75, 1.5);
                let scale = stability_scale * node_scale * score_scale;
                if !limits.infinite && self.elapsed_ms() as f64 >= self.soft_ms as f64 * scale * 0.6 {
                    break;
                }
                if limits.soft_nodes.is_some_and(|n| self.nodes >= n) {
                    break;
                }
                // A forced mate that has been confirmed well beyond its
                // length will not change with more depth.
                if !limits.infinite && score.abs() >= MATE_BOUND && depth > 2 * (MATE - score.abs()) + 8 {
                    break;
                }
            }
            if self.shared.stop.load(Ordering::Relaxed) {
                break;
            }
        }
        self.flush_nodes();
        result
    }

    fn print_info(&self, depth: i32, score: i32) {
        let ms = self.elapsed_ms().max(1);
        let nodes = self.shared.nodes.load(Ordering::Relaxed);
        let score_str = if score.abs() >= MATE_BOUND {
            let plies = MATE - score.abs();
            let moves = (plies + 1) / 2;
            format!("mate {}", if score > 0 { moves } else { -moves })
        } else {
            format!("cp {score}")
        };
        let pv: Vec<String> = self.pv[0][..self.pv_len[0]].iter().map(|m| m.to_uci()).collect();
        println!(
            "info depth {depth} seldepth {} score {score_str} nodes {nodes} nps {} hashfull {} time {ms} pv {}",
            self.seldepth,
            nodes * 1000 / ms,
            self.shared.tt.hashfull(),
            pv.join(" ")
        );
    }
}

fn score_to_tt(score: i32, ply: usize) -> i32 {
    if score >= TB_BOUND {
        score + ply as i32
    } else if score <= -TB_BOUND {
        score - ply as i32
    } else {
        score
    }
}

fn score_from_tt(score: i32, ply: usize) -> i32 {
    if score >= TB_BOUND {
        score - ply as i32
    } else if score <= -TB_BOUND {
        score + ply as i32
    } else {
        score
    }
}

/// Run a search with `searchers.len()` threads (Lazy SMP: all threads
/// search the same position and share results through the hash table).
/// Returns the main thread's result.
///
/// The caller must clear `shared.stop` beforehand (the UCI loop does so
/// before starting the search thread, so an early `stop` is never lost).
pub fn search_parallel(searchers: &mut [Searcher], root: &Board, history: &[u64], limits: &Limits) -> SearchResult {
    let shared = searchers[0].shared.clone();
    shared.nodes.store(0, Ordering::Relaxed);
    shared.tt.new_search();
    // In tablebase territory, play the tablebase move directly.
    if let Some((m, outcome)) = crate::tb::root_move(root) {
        let score = outcome * TB_WIN;
        println!("info depth 1 score cp {score} nodes 0 pv {m}");
        println!("info string tablebase move");
        while limits.infinite && !shared.stop.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        return SearchResult { best_move: m, score };
    }
    let (main, helpers) = searchers.split_first_mut().unwrap();
    std::thread::scope(|s| {
        for h in helpers.iter_mut() {
            let mut helper_limits = limits.clone();
            helper_limits.infinite = true;
            helper_limits.nodes = None;
            helper_limits.soft_nodes = None;
            std::thread::Builder::new()
                .stack_size(crate::uci::STACK_SIZE)
                .spawn_scoped(s, move || {
                    h.search(root, history, &helper_limits);
                })
                .expect("failed to spawn helper thread");
        }
        let result = main.search(root, history, limits);
        // UCI: after `go infinite`, report only once the GUI says `stop`.
        while limits.infinite && !shared.stop.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        shared.stop.store(true, Ordering::Relaxed);
        result
    })
}

/// Single-threaded search for internal callers (bench, datagen).
pub fn search_once(searcher: &mut Searcher, root: &Board, history: &[u64], limits: &Limits) -> SearchResult {
    searcher.shared.stop.store(false, Ordering::Relaxed);
    search_parallel(std::slice::from_mut(searcher), root, history, limits)
}
