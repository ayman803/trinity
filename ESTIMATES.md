# Predictions vs results

Self-play = SPRT against the previous `main` (8+0.08, UHO book).
CCRL = calibration estimate (10+0.1, balanced book, 1 thread, rated
opponents); real uncertainty about +/- 50-100.

## Changes

| Date | Change | Predicted | Self-play result | CCRL-scale result | Notes |
|---|---|---|---|---|---|
| 2026-10-02 | time management | not recorded | +4.8 (passed) | not measured | |
| 2026-10-02 | pawn correction history | not recorded | +4.0 (passed) | not measured | |
| 2026-10-03 | network #4 (512 hidden, 8 output buckets, same 120M positions) | CCRL +15 to +20 (from self-play) | +25.9 +/- 10.1 | +4 (3238 -> 3242) | self-play overstated ~6x; too little data for the bigger net |
| 2026-10-03 | hist-prune (history pruning) | a few Elo | +0.3 +/- 4.1 after 11k games (stopped) | - | not merged |
| 2026-10-04 | nonpawn-corr (non-pawn correction history) | more than hist-prune | +1.0 +/- 2.7 after 21k games (stopped) | - | not merged |
| 2026-10-03 | network #5 (+40 superbatches on 1B Lc0-derived positions) | "a big jump" (no number) | +217.8 +/- 28.4 | +169 (3242 -> 3411) | self-play overstated ~1.3x |
| 2026-10-04 | network #6 (Leela stage 120 instead of 40 superbatches) | self-play +10 to +30; CCRL +5 to +20 | +29.1 +/- 10.3 | pending | |

## Proposed (not yet run)

| Idea | Predicted self-play | Predicted CCRL | GPU time |
|---|---|---|---|
| 1. Fresh Leela positions (next 1B, not re-used) | +10 to +25 | +5 to +20 | ~35 min (+25 min CPU conversion) |
| 2. King input buckets + horizontal mirroring | +20 to +40 | +10 to +30 | ~45-60 min |
| 3. Cosine LR to a low final value + WDL ramp in stage 2 | +5 to +15 | +0 to +10 | ~26 min |
| 4. Drill Trinity's biggest disagreements with Leela (quiet positions only: no check, quiet best move, qsearch = static eval; Leela's score agrees with the game result; top ~20% by abs(Trinity - Leela)) | -5 to +15 | -5 to +10 | ~30 min + CPU selection pass |

## Calibrations

| Date | main | akimbo 3474 | Simbelmyne 3193 | Inanis 3046 | Estimate |
|---|---|---|---|---|---|
| 2026-10-02 | net #3 (bench 920942) | 10.95% | 61.95% | 79.00% | 3238 |
| 2026-10-03 | net #4 (bench 945619) | 10.95% | 63.65% | 79.25% | 3242 |
| 2026-10-03 | net #5 (bench 705171) | 34.40% | 85.90% | 93.15% | 3411 |

## Lessons so far

- Self-play gains shrink against other engines by a varying factor
  (1.3x to 6x). Treat self-play as "better or not", not "how much".
- Scores outside 25-75% give unreliable ratings; akimbo alone said 3362
  for net #5 while the weaker opponents said ~3500.
- Search tweaks borrowed from stronger engines gave ~0 here; data and
  networks gave almost all the gains.
