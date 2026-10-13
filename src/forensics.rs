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

struct Ply {
    board: Board,
    mv: Move,
    eval: Option<i32>,
    secs: f64,
}

struct Loss {
    opponent: String,
    trinity: usize,
    termination: String,
    plies: Vec<Ply>,
}

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
    let secs = parts.next().and_then(|t| t.trim_end_matches('s').parse::<f64>().ok()).unwrap_or(0.0);
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
        if !lost {
            continue;
        }
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
            out.push(Loss { opponent, trinity, termination, plies });
        }
    }
}

struct Verdict {
    class: &'static str,
    phase: &'static str,
    before: &'static str,
    detail: String,
}

fn piece_count(b: &Board) -> u32 {
    b.occupied().count_ones()
}

fn phase(b: &Board) -> &'static str {
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
        return Ok(Verdict { class: "time", phase: phase(&last), before: "-", detail: String::new() });
    }
    // Turning point: Trinity's last move with an eval still >= STILL_OK,
    // provided its eval later fell to LOST or below.
    let turn = mine.iter().rposition(|&i| loss.plies[i].eval.unwrap() >= STILL_OK);
    let Some(t) = turn else {
        let p = mine.first().map(|&i| loss.plies[i].board).unwrap_or(last);
        return Ok(Verdict { class: "lost from the book", phase: phase(&p), before: "-", detail: String::new() });
    };
    if !mine[t + 1..].iter().any(|&i| loss.plies[i].eval.unwrap() <= LOST) {
        return Ok(Verdict { class: "unclear (no clear fall)", phase: phase(&last), before: "-", detail: String::new() });
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
            });
        }
    }
    Ok(Verdict { class: "evaluation", phase: phase(&loss.plies[i].board), before, detail: String::new() })
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
        return Ok(Verdict { class: "time", phase: phase(&last), before: "-", detail: String::new() });
    }
    let mine: Vec<usize> = (0..loss.plies.len())
        .filter(|&i| loss.plies[i].board.stm == loss.trinity && loss.plies[i].eval.is_some())
        .collect();
    let mut first = true;
    for &k in &mine {
        let ply = &loss.plies[k];
        let fen = ply.board.to_fen();
        let played = ply.mv.to_uci();
        let (_, before_score) = referee.analyse(&fen, "", REFEREE_MS)?;
        if before_score < -MISTAKE {
            if first {
                return Ok(Verdict { class: "lost from the book", phase: phase(&ply.board), before: "-", detail: String::new() });
            }
            break;
        }
        first = false;
        let after_score = -referee.analyse(&fen, &played, REFEREE_MS)?.1;
        if before_score - after_score < MISTAKE {
            continue;
        }
        let before = match k.checked_sub(1).map(|j| (&loss.plies[j].board, loss.plies[j].mv)) {
            Some((b, m)) if b.make_move(m).in_check() => "check",
            Some((_, m)) if m.is_capture() => "capture",
            Some(_) => "quiet",
            None => "-",
        };
        let ms = ((ply.secs * 10_000.0) as u64).clamp(200, 5_000);
        engine.send("ucinewgame")?;
        let (longer, _) = engine.analyse(&fen, "", ms)?;
        let longer_score = if longer == played { after_score } else { -referee.analyse(&fen, &longer, REFEREE_MS)?.1 };
        let class = if longer_score - after_score >= MARGIN { "search" } else { "evaluation" };
        return Ok(Verdict {
            class,
            phase: phase(&ply.board),
            before,
            detail: format!(
                "played {} (referee {:+} -> {:+}); with 10x time Trinity plays {} (referee {:+})",
                played, before_score, after_score, longer, longer_score
            ),
        });
    }
    Ok(Verdict { class: "gradual (no single mistake)", phase: phase(&last), before: "-", detail: String::new() })
}

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
