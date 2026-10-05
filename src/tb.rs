//! Syzygy endgame tablebases (through the `shakmaty-syzygy` library).
//!
//! Set with the UCI option `SyzygyPath` (several folders separated by `;`
//! on Windows or `:` elsewhere). Two uses:
//! * at the root, pick the move that converts the win (or holds the draw)
//!   according to the distance-to-zero tables;
//! * inside the search, after a capture or pawn move (when the 50-move
//!   counter is zero), replace the search by the exact win/draw/loss.

use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use shakmaty::fen::Fen;
use shakmaty::{CastlingMode, Chess};
use shakmaty_syzygy::{Tablebase, Wdl};

use crate::board::Board;
use crate::types::Move;

static TABLES: RwLock<Option<Tablebase<Chess>>> = RwLock::new(None);
static MAX_PIECES: AtomicUsize = AtomicUsize::new(0);

/// Load the tables found in `paths`. Returns how many files were loaded.
pub fn set_path(paths: &str) -> Result<usize, String> {
    let mut tb = Tablebase::new();
    let mut count = 0;
    let separator = if cfg!(windows) { ';' } else { ':' };
    for dir in paths.split(separator).map(str::trim).filter(|d| !d.is_empty() && *d != "<empty>") {
        count += tb.add_directory(dir).map_err(|e| format!("cannot read {dir}: {e}"))?;
    }
    let max = if count > 0 { tb.max_pieces() } else { 0 };
    *TABLES.write().unwrap() = if count > 0 { Some(tb) } else { None };
    MAX_PIECES.store(max, Ordering::Relaxed);
    Ok(count)
}

/// Largest number of pieces (kings included) the loaded tables cover.
#[inline]
pub fn max_pieces() -> u32 {
    MAX_PIECES.load(Ordering::Relaxed) as u32
}

fn covered(b: &Board) -> bool {
    let n = b.occupied().count_ones();
    n <= max_pieces() && b.castling == 0
}

fn to_chess(b: &Board) -> Option<Chess> {
    let fen: Fen = b.to_fen().parse().ok()?;
    fen.into_position(CastlingMode::Standard).ok()
}

/// Win (+1), draw (0) or loss (-1) for the side to move, with the 50-move
/// rule taken into account. Only valid right after a capture or pawn move.
pub fn probe_wdl(b: &Board) -> Option<i32> {
    if b.halfmove != 0 || !covered(b) {
        return None;
    }
    let guard = TABLES.read().ok()?;
    let tb = guard.as_ref()?;
    let pos = to_chess(b)?;
    Some(match tb.probe_wdl_after_zeroing(&pos).ok()? {
        Wdl::Win => 1,
        Wdl::Loss => -1,
        Wdl::CursedWin | Wdl::BlessedLoss | Wdl::Draw => 0,
    })
}

/// The tablebase's best move at the root and its outcome (+1/0/-1).
pub fn root_move(b: &Board) -> Option<(Move, i32)> {
    if !covered(b) {
        return None;
    }
    let guard = TABLES.read().ok()?;
    let tb = guard.as_ref()?;
    let pos = to_chess(b)?;
    let (m, dtz) = tb.best_move(&pos).ok()??;
    let uci = m.to_uci(CastlingMode::Standard).to_string();
    let ours = b.parse_uci_move(&uci)?;
    // DTZ of the position after the move, from the opponent's view:
    // negative means the opponent loses, i.e. we win.
    let d = dtz.ignore_rounding().0;
    let outcome = if d == 0 {
        0
    } else if d < 0 && b.halfmove as i32 + (-d) <= 100 {
        1
    } else if d > 0 && b.halfmove as i32 + d <= 100 {
        -1
    } else {
        0
    };
    Some((ours, outcome))
}
