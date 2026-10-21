//! `trinity forensics`: why does Trinity lose? Reads the PGN files of a
//! calibration, keeps the games Trinity lost and finds each one's turning
//! point: Trinity's last move before its own evaluation fell for good. The
//! position before that move (and the few before it) is searched again with
//! ten times the time Trinity had used. If the longer search prefers another
//! move by a clear margin, the loss is counted as a search loss (more time or
//! a better search would have helped); otherwise as an evaluation loss (it
//! misjudged the position at any depth we can afford). Losses on time and
//! losses already lost out of the opening book are counted separately.
//!
//! Trinity is its own referee, so a mistake that even the longer search
//! does not see is counted as "evaluation" although deeper search might
//! find it: the search count is a lower bound.
//!
//! Usage: trinity forensics <max losses> <workers> <pgn files...>

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::board::Board;
use crate::movegen;
use crate::similarity::Engine;
use crate::types::*;

/// Trinity's eval must have been at least this (cp) before the turning point...
const STILL_OK: i32 = -50;
/// ...and later fall to this or below for good.
const LOST: i32 = -150;
/// The longer search must prefer another move by this much (cp).
const MARGIN: i32 = 50;
/// Positions searched again: the turning point and the ones before it.
const LOOK_BACK: usize = 3;

pub(crate) struct Ply {
    pub board: Board,
    pub mv: Move,
    pub eval: Option<i32>,
    pub secs: f64,
}

/// One of Trinity's games from a fastchess PGN file.
pub(crate) struct Game {
    pub opponent: String,
    pub trinity: usize,
    pub termination: String,
    /// Trinity's score: 1.0, 0.5 or 0.0.
    pub score: f64,
    pub time_control: String,
    pub opponent_time_control: String,
    pub plies: Vec<Ply>,
}

type Loss = Game;

/// SAN of a legal move (without check marks).
fn san(b: &Board, m: Move) -> String {
    if m.flag() == FLAG_KING_CASTLE {
        return "O-O".into();
    }
    if m.flag() == FLAG_QUEEN_CASTLE {
        return "O-O-O".into();
    }
    let pt = piece_type(b.piece_at(m.from()));
    let mut s = String::new();
    if pt == PAWN {
        if m.is_capture() {
            s.push((b'a' + file_of(m.from()) as u8) as char);
        }
    } else {
        s.push(b"PNBRQK"[pt] as char);
        let others: Vec<Move> = movegen::legal_moves(b)
            .iter()
            .copied()
            .filter(|o| o.to() == m.to() && o.from() != m.from() && piece_type(b.piece_at(o.from())) == pt)
            .collect();
        if !others.is_empty() {
            let same_file = others.iter().any(|o| file_of(o.from()) == file_of(m.from()));
            let same_rank = others.iter().any(|o| rank_of(o.from()) == rank_of(m.from()));
            if !same_file {
                s.push((b'a' + file_of(m.from()) as u8) as char);
            } else if !same_rank {
                s.push((b'1' + rank_of(m.from()) as u8) as char);
            } else {
                s.push_str(&square_name(m.from()));
            }
        }
    }
    if m.is_capture() {
        s.push('x');
    }
    s.push_str(&square_name(m.to()));
    if m.is_promotion() {
        s.push('=');
        s.push(b"PNBRQK"[m.promo_piece()] as char);
    }
    s
}

fn parse_san(b: &Board, text: &str) -> Option<Move> {
    let t = text.trim_end_matches(['+', '#', '!', '?']).replace('0', "O");
    movegen::legal_moves(b).iter().copied().find(|&m| san(b, m) == t)
}

/// "+0.64/12 0.091s" or "-M5/20 0.5s" -> (cp from the mover's view, seconds).
fn parse_comment(c: &str) -> (Option<i32>, f64) {
    let mut parts = c.split_whitespace();
    let eval = parts.next().and_then(|e| {
        let e = e.split('/').next()?;
        if let Some(m) = e.strip_prefix("+M").or_else(|| e.strip_prefix("M")) {
            m.parse::<i32>().ok().map(|_| 10_000)
        } else if e.starts_with("-M") {
            Some(-10_000)
        } else {
            e.parse::<f64>().ok().map(|p| (p * 100.0).round() as i32)
        }
    });
    let secs = parts.next().and_then(|t| t.trim_end_matches(',').trim_end_matches('s').parse::<f64>().ok()).unwrap_or(0.0);
    (eval, secs)
}

fn tag(header: &str, name: &str) -> Option<String> {
    let key = format!("[{name} \"");
    let start = header.find(&key)? + key.len();
    let end = header[start..].find('"')? + start;
    Some(header[start..end].to_string())
}

/// Trinity's lost games in one PGN file.
fn read_losses(text: &str, out: &mut Vec<Loss>) {
    read_games(text, out, true);
}

/// Trinity's games in one PGN file (only the lost ones if `only_lost`).
pub(crate) fn read_games(text: &str, out: &mut Vec<Game>, only_lost: bool) {
    for game in text.split("[Event ").skip(1) {
        let Some(split) = game.find("\n\n").or_else(|| game.find("\r\n\r\n")) else { continue };
        let (header, moves) = game.split_at(split);
        let (Some(white), Some(black), Some(result)) = (tag(header, "White"), tag(header, "Black"), tag(header, "Result"))
        else {
            continue;
        };
        let trinity = if white == "Trinity" {
            WHITE
        } else if black == "Trinity" {
            BLACK
        } else {
            continue;
        };
        let lost = (trinity == WHITE && result == "0-1") || (trinity == BLACK && result == "1-0");
        if only_lost && !lost {
            continue;
        }
        let score = match (result.as_str(), trinity) {
            ("1/2-1/2", _) => 0.5,
            ("1-0", WHITE) | ("0-1", BLACK) => 1.0,
            _ => 0.0,
        };
        let mut board = match tag(header, "FEN") {
            Some(fen) => match Board::from_fen(&fen) {
                Ok(b) => b,
                Err(_) => continue,
            },
            None => Board::startpos(),
        };
        let mut plies: Vec<Ply> = Vec::new();
        let mut rest = moves;
        let mut ok = true;
        while let Some(c) = rest.chars().next() {
            if c.is_whitespace() {
                rest = &rest[c.len_utf8()..];
            } else if c == '{' {
                let end = rest.find('}').unwrap_or(rest.len() - 1);
                if let Some(last) = plies.last_mut() {
                    (last.eval, last.secs) = parse_comment(&rest[1..end]);
                }
                rest = &rest[end + 1..];
            } else {
                let end = rest.find(|ch: char| ch.is_whitespace() || ch == '{').unwrap_or(rest.len());
                let token = &rest[..end];
                rest = &rest[end..];
                if token.ends_with('.') || token.contains("...") || ["1-0", "0-1", "1/2-1/2", "*"].contains(&token) {
                    continue;
                }
                let token = token.trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == '.');
                if token.is_empty() {
                    continue;
                }
                match parse_san(&board, token) {
                    Some(m) => {
                        plies.push(Ply { board, mv: m, eval: None, secs: 0.0 });
                        board = board.make_move(m);
                    }
                    None => {
                        ok = false;
                        break;
                    }
                }
            }
        }
        if ok {
            let opponent = if trinity == WHITE { black } else { white };
            let termination = tag(header, "Termination").unwrap_or_default();
            // fastchess writes one tag for both sides, or one per colour.
            let side_tc = |side: usize| {
                tag(header, if side == WHITE { "WhiteTimeControl" } else { "BlackTimeControl" })
                    .or_else(|| tag(header, "TimeControl"))
                    .unwrap_or_default()
            };
            let (time_control, opponent_time_control) = (side_tc(trinity), side_tc(1 - trinity));
            out.push(Game { opponent, trinity, termination, score, time_control, opponent_time_control, plies });
        }
    }
}

#[derive(Default)]
struct Verdict {
    class: &'static str,
    phase: &'static str,
    before: &'static str,
    detail: String,
    /// Gradual losses: of the worst few moves, how many 10x time fixes.
    small_search: u32,
    small_eval: u32,
    /// Clear mistakes: time spent on the mistake / Trinity's average move time.
    time_ratio: Option<f64>,
    /// Clear mistakes: how many of Trinity's own moves after the book.
    out_of_book: Option<usize>,
}

fn piece_count(b: &Board) -> u32 {
    b.occupied().count_ones()
}

pub(crate) fn phase(b: &Board) -> &'static str {
    match piece_count(b) {
        n if n > 24 => "opening",
        n if n > 12 => "middlegame",
        _ => "endgame",
    }
}

fn judge(engine: &mut Engine, loss: &Loss) -> Result<Verdict, String> {
    let mine: Vec<usize> = (0..loss.plies.len())
        .filter(|&i| loss.plies[i].board.stm == loss.trinity && loss.plies[i].eval.is_some())
        .collect();
    let last = loss.plies.last().map(|p| p.board).unwrap_or_else(Board::startpos);
    if loss.termination.contains("time") {
        return Ok(Verdict { class: "time", phase: phase(&last), before: "-", detail: String::new(), ..Default::default() });
    }
    // Turning point: Trinity's last move with an eval still >= STILL_OK,
    // provided its eval later fell to LOST or below.
    let turn = mine.iter().rposition(|&i| loss.plies[i].eval.unwrap() >= STILL_OK);
    let Some(t) = turn else {
        let p = mine.first().map(|&i| loss.plies[i].board).unwrap_or(last);
        return Ok(Verdict { class: "lost from the book", phase: phase(&p), before: "-", detail: String::new(), ..Default::default() });
    };
    if !mine[t + 1..].iter().any(|&i| loss.plies[i].eval.unwrap() <= LOST) {
        return Ok(Verdict { class: "unclear (no clear fall)", phase: phase(&last), before: "-", detail: String::new(), ..Default::default() });
    }
    let i = mine[t];
    let before = match i.checked_sub(1).map(|j| (&loss.plies[j].board, loss.plies[j].mv)) {
        Some((b, m)) if b.make_move(m).in_check() => "check",
        Some((_, m)) if m.is_capture() => "capture",
        Some(_) => "quiet",
        None => "-",
    };
    for &k in mine[t.saturating_sub(LOOK_BACK)..=t].iter().rev() {
        let ply = &loss.plies[k];
        let ms = ((ply.secs * 10_000.0) as u64).clamp(200, 5_000);
        let fen = ply.board.to_fen();
        engine.send("ucinewgame")?;
        let (best, best_score) = engine.analyse(&fen, "", ms)?;
        let played = ply.mv.to_uci();
        if best == played {
            continue;
        }
        engine.send("ucinewgame")?;
        let (_, reply) = engine.analyse(&fen, &played, ms)?;
        let played_score = -reply;
        if best_score - played_score >= MARGIN {
            let back = t - mine.iter().position(|&x| x == k).unwrap();
            return Ok(Verdict {
                class: "search",
                phase: phase(&loss.plies[i].board),
                before,
                detail: format!(
                    "{} moves before the turning point: played {} ({}), longer search prefers {} ({})",
                    back, played, played_score, best, best_score
                ),
                ..Default::default()
            });
        }
    }
    Ok(Verdict { class: "evaluation", phase: phase(&loss.plies[i].board), before, detail: String::new(), ..Default::default() })
}

/// Referee search time per position (ms).
const REFEREE_MS: u64 = 100;
/// A move that loses at least this much (cp, by the referee) is a mistake.
const MISTAKE: i32 = 100;

/// With a stronger engine as referee: the turning point is Trinity's first
/// move that, by the referee's judgment, threw away at least MISTAKE cp
/// while the position was not yet lost. Trinity then searches that
/// position for ten times as long; if it now plays a move the referee
/// rates clearly better, the loss is a search loss, otherwise an
/// evaluation loss. The referee only judges positions.
fn judge_with_referee(engine: &mut Engine, referee: &mut Engine, loss: &Loss) -> Result<Verdict, String> {
    let last = loss.plies.last().map(|p| p.board).unwrap_or_else(Board::startpos);
    if loss.termination.contains("time") {
        return Ok(Verdict { class: "time", phase: phase(&last), before: "-", ..Default::default() });
    }
    let mine: Vec<usize> = (0..loss.plies.len())
        .filter(|&i| loss.plies[i].board.stm == loss.trinity && loss.plies[i].eval.is_some())
        .collect();
    let avg_secs = mine.iter().map(|&i| loss.plies[i].secs).sum::<f64>() / mine.len().max(1) as f64;
    let retry = |engine: &mut Engine, referee: &mut Engine, k: usize, after: i32, margin: i32| {
        retry_longer(engine, referee, &loss.plies[k], after, margin)
    };
    // The referee's view of each of Trinity's moves until the game is
    // clearly lost: (index, score before, score after).
    let mut scored = Vec::new();
    for (n, &k) in mine.iter().enumerate() {
        let ply = &loss.plies[k];
        let fen = ply.board.to_fen();
        let before_score = referee_score(referee, &fen, "")?;
        if before_score < -MISTAKE {
            if n == 0 {
                return Ok(Verdict { class: "lost from the book", phase: phase(&ply.board), before: "-", ..Default::default() });
            }
            break;
        }
        let after_score = -referee_score(referee, &fen, &ply.mv.to_uci())?;
        scored.push((n, k, before_score, after_score));
        if before_score - after_score < MISTAKE {
            continue;
        }
        let before = match k.checked_sub(1).map(|j| (&loss.plies[j].board, loss.plies[j].mv)) {
            Some((b, m)) if b.make_move(m).in_check() => "check",
            Some((_, m)) if m.is_capture() => "capture",
            Some(_) => "quiet",
            None => "-",
        };
        let (fixed, longer, longer_score) = retry(engine, referee, k, after_score, MARGIN)?;
        return Ok(Verdict {
            class: if fixed { "search" } else { "evaluation" },
            phase: phase(&ply.board),
            before,
            detail: format!(
                "played {} (referee {:+} -> {:+}); with 10x time Trinity plays {} (referee {:+})",
                ply.mv.to_uci(), before_score, after_score, longer, longer_score
            ),
            time_ratio: (avg_secs > 0.0).then(|| ply.secs / avg_secs),
            out_of_book: Some(n + 1),
            ..Default::default()
        });
    }
    // No single big mistake: test Trinity's worst few moves the same way.
    scored.sort_by_key(|&(_, _, b, a)| a - b);
    let (mut small_search, mut small_eval) = (0, 0);
    for &(_, k, b, a) in scored.iter().take(SMALL_MOVES) {
        let drop = b - a;
        if drop < SMALL_MIN {
            break;
        }
        if retry(engine, referee, k, a, (drop / 2).max(15))?.0 {
            small_search += 1;
        } else {
            small_eval += 1;
        }
    }
    Ok(Verdict { class: "gradual (no single mistake)", phase: phase(&last), before: "-", small_search, small_eval, ..Default::default() })
}

/// Ten times the time Trinity used: does it now play a move the referee
/// rates at least `margin` better than the move it played (`after`)?
fn retry_longer(engine: &mut Engine, referee: &mut Engine, ply: &Ply, after: i32, margin: i32) -> Result<(bool, String, i32), String> {
    let fen = ply.board.to_fen();
    let ms = ((ply.secs * 10_000.0) as u64).clamp(200, 5_000);
    engine.send("ucinewgame")?;
    let (longer, _) = engine.analyse(&fen, "", ms)?;
    let score = if longer == ply.mv.to_uci() { after } else { -referee_score(referee, &fen, &longer)? };
    Ok((score - after >= margin, longer, score))
}

fn referee_score(referee: &mut Engine, fen: &str, moves: &str) -> Result<i32, String> {
    Ok(referee.analyse(fen, moves, REFEREE_MS)?.1)
}

/// Gradual losses: how many of the worst moves to test, and the smallest
/// drop (cp) worth testing.
const SMALL_MOVES: usize = 3;
const SMALL_MIN: i32 = 30;

pub fn run(max: usize, workers: usize, args: &[String]) -> Result<(), String> {
    let referee_path = args.iter().find_map(|a| a.strip_prefix("--referee=")).map(str::to_string);
    let files: Vec<&String> = args.iter().filter(|a| !a.starts_with("--referee=")).collect();
    let mut losses = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(f).map_err(|e| format!("{f}: {e}"))?;
        read_losses(&text, &mut losses);
    }
    let total = losses.len();
    // An even sample over all opponents if there are more than `max`.
    let chosen: Vec<usize> = if total > max && max > 0 {
        (0..max).map(|k| k * total / max).collect()
    } else {
        (0..total).collect()
    };
    println!("{total} lost games found; examining {}", chosen.len());
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe = exe.to_string_lossy().to_string();
    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, Verdict)>> = Mutex::new(Vec::new());
    let error: Mutex<Option<String>> = Mutex::new(None);
    std::thread::scope(|s| {
        for _ in 0..workers.max(1) {
            s.spawn(|| {
                let mut engine = match Engine::start("Trinity", &exe) {
                    Ok(e) => e,
                    Err(e) => {
                        *error.lock().unwrap() = Some(e);
                        return;
                    }
                };
                let mut referee = match referee_path.as_deref().map(|p| Engine::start("referee", p)).transpose() {
                    Ok(r) => r,
                    Err(e) => {
                        *error.lock().unwrap() = Some(e);
                        return;
                    }
                };
                loop {
                    let k = next.fetch_add(1, Ordering::Relaxed);
                    if k >= chosen.len() {
                        break;
                    }
                    let i = chosen[k];
                    let verdict = match referee.as_mut() {
                        Some(r) => judge_with_referee(&mut engine, r, &losses[i]),
                        None => judge(&mut engine, &losses[i]),
                    };
                    match verdict {
                        Ok(v) => results.lock().unwrap().push((i, v)),
                        Err(e) => {
                            *error.lock().unwrap() = Some(e);
                            return;
                        }
                    }
                    let d = done.fetch_add(1, Ordering::Relaxed) + 1;
                    if d % 20 == 0 {
                        println!("{d} of {} losses examined", chosen.len());
                    }
                }
            });
        }
    });
    if let Some(e) = error.into_inner().unwrap() {
        return Err(e);
    }
    let results = results.into_inner().unwrap();
    let n = results.len().max(1) as f64;
    let count = |f: &dyn Fn(&Verdict) -> bool| results.iter().filter(|(_, v)| f(v)).count();
    println!();
    println!("LOSS FORENSICS: {} of Trinity's {total} lost games (re-searched at 10x the time it used, 0.2-5 s)", results.len());
    match &referee_path {
        Some(p) => println!("Referee for mistakes: {p} ({REFEREE_MS} ms per position)"),
        None => println!("No referee: Trinity judges itself (search losses are undercounted)"),
    }
    for class in ["search", "evaluation", "time", "lost from the book", "gradual (no single mistake)", "unclear (no clear fall)"] {
        let c = count(&|v: &Verdict| v.class == class);
        println!("  {class:<28} {c:>5}  ({:.0}%)", 100.0 * c as f64 / n);
    }
    println!("By phase at the turning point (search / evaluation):");
    for ph in ["opening", "middlegame", "endgame"] {
        let s = count(&|v: &Verdict| v.phase == ph && v.class == "search");
        let e = count(&|v: &Verdict| v.phase == ph && v.class == "evaluation");
        println!("  {ph:<12} {s:>4} / {e:<4}");
    }
    println!("Opponent's move just before the turning point (search / evaluation):");
    for b in ["capture", "check", "quiet"] {
        let s = count(&|v: &Verdict| v.before == b && v.class == "search");
        let e = count(&|v: &Verdict| v.before == b && v.class == "evaluation");
        println!("  {b:<12} {s:>4} / {e:<4}");
    }
    println!("By opponent (search / evaluation / other):");
    let mut opponents: Vec<&str> = chosen.iter().map(|&i| losses[i].opponent.as_str()).collect();
    opponents.sort();
    opponents.dedup();
    for o in opponents {
        let s = results.iter().filter(|(i, v)| losses[*i].opponent == o && v.class == "search").count();
        let e = results.iter().filter(|(i, v)| losses[*i].opponent == o && v.class == "evaluation").count();
        let all = results.iter().filter(|(i, _)| losses[*i].opponent == o).count();
        println!("  {o:<12} {s:>4} / {e:>4} / {:>4}", all - s - e);
    }
    let (gs, ge) = results.iter().fold((0, 0), |(s, e), (_, v)| (s + v.small_search, e + v.small_eval));
    println!(
        "Gradual losses, their {SMALL_MOVES} worst moves (drop >= {SMALL_MIN} cp) retried at 10x time: search {gs} / evaluation {ge}"
    );
    let mut ratios: Vec<f64> = results.iter().filter(|(_, v)| v.class == "search").filter_map(|(_, v)| v.time_ratio).collect();
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if !ratios.is_empty() {
        println!(
            "Search mistakes: time spent on the mistake / Trinity's average move time in that game: median {:.2} ({} cases)",
            ratios[ratios.len() / 2],
            ratios.len()
        );
    }
    for (label, lo, hi) in [("first 5 moves out of the book", 1, 5), ("later moves", 6, usize::MAX)] {
        let mut r: Vec<f64> = results
            .iter()
            .filter(|(_, v)| v.class == "search" && v.out_of_book.is_some_and(|n| n >= lo && n <= hi))
            .filter_map(|(_, v)| v.time_ratio)
            .collect();
        r.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if !r.is_empty() {
            println!("  ...on the {label}: {} search mistakes, median time ratio {:.2}", r.len(), r[r.len() / 2]);
        }
    }
    println!("Examples of search losses:");
    for (_, v) in results.iter().filter(|(_, v)| v.class == "search").take(5) {
        println!("  {}", v.detail);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn san_round_trip() {
        let b = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
        for m in movegen::legal_moves(&b).iter().copied() {
            assert_eq!(parse_san(&b, &san(&b, m)), Some(m), "{}", san(&b, m));
        }
        assert_eq!(parse_san(&b, "O-O").map(|m| m.to_uci()), Some("e1g1".into()));
        assert_eq!(parse_san(&b, "Nxf7+").map(|m| m.to_uci()), Some("e5f7".into()));
    }

    #[test]
    fn reads_a_lost_game() {
        let pgn = "[Event \"x\"]\n[White \"Trinity\"]\n[Black \"akimbo\"]\n[Result \"0-1\"]\n\n\
                   1. f3 {book} e5 {book} 2. g4 {-0.20/10 0.10s} Qh4# {+M1/1 0.01s} 0-1\n";
        let mut out = Vec::new();
        read_losses(pgn, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].plies.len(), 4);
        assert_eq!(out[0].plies[2].eval, Some(-20));
        assert_eq!(out[0].opponent, "akimbo");
    }
}

#[cfg(test)]
mod ablation {
    use super::*;
    use crate::search::{self, Limits, Searcher, Shared, ABLATE};
    use std::sync::atomic::Ordering as O;

    const RULES: &[(&str, u32)] = &[
        ("RFP", 0), ("razoring", 1), ("null move", 2), ("IIR", 3), ("LMP", 4), ("futility", 5),
        ("SEE quiet prune", 6), ("SEE capture prune", 7), ("singular ext", 8), ("LMR (all)", 9),
        ("QS futility", 10), ("QS SEE prune", 11), ("LMR unsure term", 12), ("LMR history term", 13),
    ];

    fn best_moves(boards: &[Board], nodes: u64, hash: usize) -> Vec<Move> {
        let threads = 4;
        let chunks: Vec<Vec<(usize, Board)>> = (0..threads)
            .map(|t| boards.iter().copied().enumerate().filter(|(i, _)| i % threads == t).collect())
            .collect();
        let mut out = vec![Move::NONE; boards.len()];
        std::thread::scope(|sc| {
            let hs: Vec<_> = chunks
                .into_iter()
                .map(|c| {
                    sc.spawn(move || {
                        let mut s = Searcher::new(Shared::new(hash), 0);
                        s.silent = true;
                        let mut r = Vec::new();
                        for (i, b) in c {
                            s.clear();
                            let lim = Limits { nodes: Some(nodes), ..Default::default() };
                            r.push((i, search::search_once(&mut s, &b, &[], &lim).best_move));
                        }
                        r
                    })
                })
                .collect();
            for h in hs {
                for (i, m) in h.join().unwrap() {
                    out[i] = m;
                }
            }
        });
        out
    }

    #[test]
    #[ignore]
    fn ablation_audit() {
        let n: usize = std::env::var("N").ok().and_then(|v| v.parse().ok()).unwrap_or(800);
        let base: u64 = std::env::var("BASE").ok().and_then(|v| v.parse().ok()).unwrap_or(250_000);
        let mult: u64 = std::env::var("MULT").ok().and_then(|v| v.parse().ok()).unwrap_or(16);
        let mut games = Vec::new();
        for f in ["games/calibration-2026-10-14_18-06-main-losses.pgn", "games/calibration-2026-10-17_09-41-tm-reserve-losses.pgn"] {
            if let Ok(text) = std::fs::read_to_string(f) {
                read_games(&text, &mut games, false);
            }
        }
        let mut boards = Vec::new();
        for g in &games {
            for (i, p) in g.plies.iter().enumerate() {
                let mine = p.board.stm == g.trinity;
                let pieces = p.board.occupied().count_ones();
                if mine && i >= 16 && !p.board.in_check() && pieces >= 12 && p.eval.is_some_and(|e| e.abs() <= 300) {
                    boards.push(p.board);
                }
            }
        }
        let step = (boards.len() / n).max(1);
        let boards: Vec<Board> = boards.into_iter().step_by(step).take(n).collect();
        println!("ABL positions {} from {} games; base {} nodes, reference {}x", boards.len(), games.len(), base, mult);
        ABLATE.store(0, O::Relaxed);
        let reference = best_moves(&boards, base * mult, 16);
        let baseline = best_moves(&boards, base, 16);
        let agree = |v: &[Move]| v.iter().zip(&reference).filter(|(a, b)| a == b).count();
        let b0 = agree(&baseline);
        println!("ABL baseline agrees with reference: {b0} / {}", boards.len());
        // No-change baseline (Scout, 21 October): the same engine with only
        // the hash size different; its "fixed" and "broken" counts are
        // what chance alone produces.
        if std::env::var("NOISE").is_ok() {
            // A bigger table may change nothing at this budget, so the
            // node budget is also moved by 4% either way.
            for (hash, nodes) in [(8, base), (64, base), (16, base * 96 / 100), (16, base * 104 / 100)] {
                let v = best_moves(&boards, nodes, hash);
                let fixed = (0..boards.len()).filter(|&i| baseline[i] != reference[i] && v[i] == reference[i]).count();
                let broken = (0..boards.len()).filter(|&i| baseline[i] == reference[i] && v[i] != reference[i]).count();
                println!(
                    "ABL no change, hash {hash:>2} MB, {nodes} nodes: agrees {} ({:+}), fixed {fixed}, broken {broken}, net {:+}",
                    agree(&v),
                    agree(&v) as i64 - b0 as i64,
                    fixed as i64 - broken as i64
                );
            }
            return;
        }
        for &(name, bit) in RULES {
            ABLATE.store(1 << bit, O::Relaxed);
            let v = best_moves(&boards, base, 16);
            let fixed = (0..boards.len()).filter(|&i| baseline[i] != reference[i] && v[i] == reference[i]).count();
            let broken = (0..boards.len()).filter(|&i| baseline[i] == reference[i] && v[i] != reference[i]).count();
            println!(
                "ABL {name:<18} off: agrees {} ({:+}), fixed {fixed}, broken {broken}, net {:+} (sd about {:.0})",
                agree(&v),
                agree(&v) as i64 - b0 as i64,
                fixed as i64 - broken as i64,
                ((fixed + broken) as f64).sqrt()
            );
        }
        ABLATE.store(0, O::Relaxed);
    }
}
