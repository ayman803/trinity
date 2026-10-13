//! `trinity timeuse <pgn files...>`: how Trinity spends its clock, from the
//! move times fastchess records in its PGN files. Reports how much of the
//! available time is left over, how often moves stop at the hard limit or
//! very early, how the time is spread over the game, and base rates for the
//! loss-forensics leads (time ratio, phase, quiet opponent moves). The
//! opponent's time use is shown next to Trinity's for comparison.

use crate::forensics::{phase, read_games, Game};

/// A time control from the PGN tag: "10+0.1", "40/20" or "40/20+1".
#[derive(Clone, Copy)]
struct TimeControl {
    moves: Option<u32>,
    secs: f64,
    inc: f64,
}

fn parse_tc(tag: &str) -> Option<TimeControl> {
    let (moves, rest) = match tag.split_once('/') {
        Some((m, r)) => (Some(m.trim().parse().ok()?), r),
        None => (None, tag),
    };
    let (secs, inc): (f64, f64) = match rest.split_once('+') {
        Some((s, i)) => (s.trim().parse().ok()?, i.trim().parse().ok()?),
        None => (rest.trim().parse().ok()?, 0.0),
    };
    (secs > 0.0 && secs.is_finite() && inc >= 0.0 && inc.is_finite()).then_some(TimeControl { moves, secs, inc })
}

/// Trinity's own soft and hard limits (seconds) for a move, as search.rs
/// computes them from the clock before the move.
fn limits(tc: TimeControl, clock: f64, moves_made_in_period: u32) -> (f64, f64) {
    let t = (clock - 0.010).max(0.001);
    let base = match tc.moves {
        Some(m) => t / (m.saturating_sub(moves_made_in_period).clamp(1, 50) as f64 + 1.0) + tc.inc * 0.75,
        None => t / 25.0 + tc.inc * 0.75,
    };
    (base.min(t / 2.0), (base * 3.0).min(t * 0.6))
}

#[derive(Default)]
struct SideStats {
    games: usize,
    moves: usize,
    /// Share of the time available in the game that was left unused.
    left_share: Vec<f64>,
    /// Seconds left at the end of each 40-move period (repeating controls).
    period_left: Vec<f64>,
    quick: usize,
    near_hard: usize,
    ratios: Vec<f64>,
    /// Time ratios of moves after which the mover's own eval fell >= 100 cp.
    swing_ratios: Vec<f64>,
    by_move: [Vec<f64>; 6],
    by_phase: [(usize, f64); 3],
    after_quiet: usize,
    after_opponent: usize,
    /// Own moves still to come, by move-number bucket and by piece count
    /// (for a remaining-moves model of the game length).
    remaining_by_move: [Vec<f64>; 6],
    remaining_by_pieces: [Vec<f64>; 4],
}

fn piece_bucket(count: u32) -> usize {
    match count {
        25..=32 => 0,
        17..=24 => 1,
        9..=16 => 2,
        _ => 3,
    }
}

fn quantile(v: &mut [f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() - 1) as f64 * q).round() as usize]
}

fn median(v: &mut [f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn pct(n: usize, d: usize) -> f64 {
    100.0 * n as f64 / d.max(1) as f64
}

fn bucket(fullmove: u16) -> usize {
    match fullmove {
        0..=10 => 0,
        11..=20 => 1,
        21..=30 => 2,
        31..=40 => 3,
        41..=60 => 4,
        _ => 5,
    }
}

fn phase_index(p: &str) -> usize {
    match p {
        "opening" => 0,
        "middlegame" => 1,
        _ => 2,
    }
}

/// Walk one game for one side (Trinity or its opponent) and add to `s`.
fn walk(game: &Game, side: usize, tc: TimeControl, s: &mut SideStats) {
    let own: Vec<usize> = (0..game.plies.len()).filter(|&i| game.plies[i].board.stm == side).collect();
    if own.is_empty() {
        return;
    }
    s.games += 1;
    let avg = own.iter().map(|&i| game.plies[i].secs).sum::<f64>() / own.len() as f64;
    let mut clock = tc.secs;
    let mut used_total = 0.0;
    let mut available = tc.secs;
    let mut made = 0u32;
    for (n, &i) in own.iter().enumerate() {
        let ply = &game.plies[i];
        let (soft, hard) = limits(tc, clock, made);
        let t = ply.secs;
        s.moves += 1;
        if t <= 0.35 * soft {
            s.quick += 1;
        }
        if t >= 0.9 * hard {
            s.near_hard += 1;
        }
        let ratio = if avg > 0.0 { t / avg } else { 1.0 };
        s.ratios.push(ratio);
        s.by_move[bucket(ply.board.fullmove)].push(t);
        let remaining = (own.len() - n - 1) as f64;
        s.remaining_by_move[bucket(ply.board.fullmove)].push(remaining);
        s.remaining_by_pieces[piece_bucket(ply.board.occupied().count_ones())].push(remaining);
        let ph = phase_index(phase(&ply.board));
        s.by_phase[ph].0 += 1;
        s.by_phase[ph].1 += t;
        if i > 0 {
            s.after_opponent += 1;
            let prev = &game.plies[i - 1];
            if !prev.mv.is_capture() && !prev.mv.is_promotion() && !prev.board.make_move(prev.mv).in_check() {
                s.after_quiet += 1;
            }
        }
        if let (Some(now), Some(next)) = (ply.eval, own.get(n + 1).and_then(|&j| game.plies[j].eval)) {
            if now - next >= 100 && now.abs() < 1000 {
                s.swing_ratios.push(ratio);
            }
        }
        used_total += t;
        clock = clock - t + tc.inc;
        available += tc.inc;
        made += 1;
        if let Some(m) = tc.moves {
            if made == m {
                s.period_left.push(clock);
                clock += tc.secs;
                available += tc.secs;
                made = 0;
            }
        }
    }
    if available > 0.0 {
        s.left_share.push((available - used_total) / available);
    }
}

fn report(name: &str, s: &mut SideStats) {
    println!("{name}: {} games, {} moves", s.games, s.moves);
    println!(
        "  time left unused at the end of the game: median {:.0}% of the time available",
        100.0 * median(&mut s.left_share)
    );
    if !s.period_left.is_empty() {
        println!("  time carried over at the end of a period: median {:.1} s", median(&mut s.period_left));
    }
    println!(
        "  moves stopped very early (under 35% of the soft limit): {:.0}%; near the hard limit: {:.0}%",
        pct(s.quick, s.moves),
        pct(s.near_hard, s.moves)
    );
    let all = median(&mut s.ratios);
    let below_half = s.ratios.iter().filter(|&&r| r < 0.5).count();
    println!(
        "  time ratio (move time / this game's average): median {:.2} over all moves, {:.0}% of moves below 0.5",
        all,
        pct(below_half, s.ratios.len())
    );
    if !s.swing_ratios.is_empty() {
        println!(
            "  ...for the {} moves after which the mover's own eval fell by 100+ cp: median {:.2}",
            s.swing_ratios.len(),
            median(&mut s.swing_ratios)
        );
    }
    let labels = ["1-10", "11-20", "21-30", "31-40", "41-60", "61+"];
    let parts: Vec<String> = labels
        .iter()
        .zip(s.by_move.iter_mut())
        .filter(|(_, v)| !v.is_empty())
        .map(|(l, v)| format!("{l}: {:.2}s", median(v)))
        .collect();
    println!("  median seconds per move by move number: {}", parts.join(", "));
    let total_moves: usize = s.by_phase.iter().map(|p| p.0).sum();
    let phases: Vec<String> = ["opening", "middlegame", "endgame"]
        .iter()
        .zip(&s.by_phase)
        .map(|(l, (n, t))| format!("{l} {:.0}% of moves, {:.2}s each", pct(*n, total_moves), t / (*n).max(1) as f64))
        .collect();
    println!("  by phase: {}", phases.join("; "));
    println!("  moves played after a quiet opponent move: {:.0}%", pct(s.after_quiet, s.after_opponent));
    let rem: Vec<String> = labels
        .iter()
        .zip(s.remaining_by_move.iter_mut())
        .filter(|(_, v)| !v.is_empty())
        .map(|(l, v)| format!("{l}: {:.0} (80%: {:.0})", quantile(v, 0.5), quantile(v, 0.8)))
        .collect();
    println!("  own moves still to come, median (and 80th percentile) by move number: {}", rem.join(", "));
    let rem: Vec<String> = ["25-32 pieces", "17-24", "9-16", "8 or fewer"]
        .iter()
        .zip(s.remaining_by_pieces.iter_mut())
        .filter(|(_, v)| !v.is_empty())
        .map(|(l, v)| format!("{l}: {:.0} (80%: {:.0})", quantile(v, 0.5), quantile(v, 0.8)))
        .collect();
    println!("  ...by pieces on the board: {}", rem.join(", "));
}

pub fn run(files: &[String]) -> Result<(), String> {
    let mut games = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(f).map_err(|e| format!("{f}: {e}"))?;
        read_games(&text, &mut games, false);
    }
    if games.is_empty() {
        return Err("no games of Trinity found".into());
    }
    let mut tcs: Vec<String> = games.iter().map(|g| g.time_control.clone()).collect();
    tcs.sort();
    tcs.dedup();
    println!("TIME USE: {} games of Trinity, time control {}", games.len(), tcs.join(", "));
    let mut trinity = SideStats::default();
    let mut opponents = SideStats::default();
    let mut skipped = 0;
    for g in &games {
        let Some(tc) = parse_tc(&g.time_control) else {
            skipped += 1;
            continue;
        };
        walk(g, g.trinity, tc, &mut trinity);
        if let Some(otc) = parse_tc(&g.opponent_time_control) {
            walk(g, 1 - g.trinity, otc, &mut opponents);
        }
    }
    if skipped > 0 {
        println!("({skipped} games skipped: time control not understood)");
    }
    report("Trinity", &mut trinity);
    report("Opponents", &mut opponents);
    let score: f64 = games.iter().map(|g| g.score).sum::<f64>() / games.len() as f64;
    println!("Trinity's score in these games: {:.1}%", 100.0 * score);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_time_controls() {
        let tc = parse_tc("10+0.1").unwrap();
        assert!(tc.moves.is_none() && (tc.secs - 10.0).abs() < 1e-9 && (tc.inc - 0.1).abs() < 1e-9);
        let tc = parse_tc("40/20").unwrap();
        assert_eq!(tc.moves, Some(40));
        assert!((tc.secs - 20.0).abs() < 1e-9 && tc.inc == 0.0);
        assert!(parse_tc("inf").is_none());
    }

    #[test]
    fn limits_match_search() {
        // Sudden death 10+0.1 with 10 s left: base 0.4 + 0.075.
        let (soft, hard) = limits(parse_tc("10+0.1").unwrap(), 10.0, 0);
        assert!((soft - (9.99 / 25.0 + 0.075)).abs() < 1e-6);
        assert!((hard - 3.0 * (9.99 / 25.0 + 0.075)).abs() < 1e-6);
        // 40/20 with 32 moves to go: base t/33.
        let (soft, _) = limits(parse_tc("40/20").unwrap(), 20.0, 8);
        assert!((soft - 19.99 / 33.0).abs() < 1e-6);
    }
}
