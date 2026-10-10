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
| 2026-10-04 | network #6 (Leela stage 120 instead of 40 superbatches) | self-play +10 to +30; CCRL +5 to +20 | +29.1 +/- 10.3 | about +3 (akimbo 34.4% -> 35.1%; same opponent, same settings) | self-play overstated ~10x |
| 2026-10-04 | drill network (owner's idea: main's net + 5 superbatches on its biggest quiet disagreements with Leela, 50/50 with ordinary positions) | -5 to +15 | -92 +/- 41 after 124 games (stopped; clearly worse) | - | not merged. Differences from net #6's Leela stage: position mix, LR 0.0001 (vs 0.0005), 5 superbatches. WDL was 0.5, the same as the Leela stage (0.4 is only used for stage 1 on our own data). Retry later with ONE change: drill positions ~5% of a normal Leela mix, everything else as net #6's stage |
| 2026-10-05 | fresh Leela data (main's net + 40 superbatches on 1B never-seen Leela positions, LR 0.0002) | self-play +5 to +20 | +6.0 +/- 4.1 after 8930 games (passed, merged; bench 735643) | -13 together with tablebases (3390 -> 3377; akimbo, which has no tablebase support: -0.85% = about -6 +/- 10) | no CCRL gain; self-play overstated again. TB-off calibration pending to separate the two |
| 2026-10-05 | corr-unsure (owner's "unsure" idea, cheap form: reduce LMR by 1 where the correction shifts eval >= 12 cp) | self-play -3 to +5 | pending (laptop) | - | |
| 2026-10-06 | king buckets (8 buckets, mirrored; starts from main's network, 80 superbatches on 1B unseen Leela positions, LR 0.0005) | self-play +10 to +40; CCRL +5 to +20 (about 6% slower search) | +19.1 +/- 8.5 after 2622 games (passed, merged; bench 550859) | +17 (3379 -> 3396; all four opponents up) | self-play held up almost fully (~1.1x), the best ratio so far |
| 2026-10-07 | king buckets, longer (main's KB network + 160 superbatches on the next 1B unseen Leela positions, LR 0.0005) | self-play +3 to +15; CCRL +2 to +12 | -13.8 +/- 7.9 after 2888 games (FAILED, not merged) | - | wrong. Suspected cause: restarting at LR 0.0005 undid the tuned network (refinement on fresh data at 0.0002 gave +6). Being tested: same data, LR 0.0002 sliding to near zero |
| 2026-10-07 | king buckets, gentle refinement (same 160 superbatches and the same positions as the failed run, LR 0.0002 sliding to 0.000002) | self-play -3 to +8 | -3.8 +/- 4.6 after 8016 games (FAILED, not merged) | - | the gentle rate hurt much less (-3.8 vs -13.8), so the rate explained most of the loss, but more Leela data still gives nothing on this network |
| 2026-10-07 | SPSA tune of 20 search settings (never tuned before; 5 chains x ~3 GitHub runs, 5+0.05) | self-play +10 to +30; CCRL +5 to +20 | +11.2 +/- 3.9 at 8+0.08 (9000 games, passed) and +8.7 +/- 6.3 at 30+0.3 (3000 games, LLR 1.83); merged, bench 637984 | about +3 (3396 -> 3399; within noise) | self-play overstated ~4x; most of the gain did not carry over |
| 2026-10-09 | big network: 1024 neurons, king buckets, from scratch, 600 superbatches on up to 4B Leela positions, LR 0.001 cosine, WDL 0.5 (about 28% slower search) | self-play +10 to +40; CCRL +5 to +25. Kill test: no pass by 20 October = network path used up | +80.0 +/- 4.3 at 8+0.08 (9000 games, 3380 W / 1343 L, 0 time losses; merged, bench 626889). +82.0 +/- 6.7 at 30+0.3 (3000 games, passed). Trained on 3.65B positions (whole file) | +50 (3399 -> 3449; all four opponents up 5-9 points) | far above the prediction: the earlier networks were starved of capacity and training time |

## Proposed (not yet run)

| Idea | Predicted self-play | Predicted CCRL | GPU time |
|---|---|---|---|
| 1. Fresh Leela positions (next 1B, not re-used) | +10 to +25 | +5 to +20 | ~35 min (+25 min CPU conversion) |
| 2. King input buckets + horizontal mirroring | +20 to +40 | +10 to +30 | ~45-60 min |
| 3. Cosine LR to a low final value + WDL ramp in stage 2 | +5 to +15 | +0 to +10 | ~26 min |
| 4. Drill Trinity's biggest disagreements with Leela (quiet positions only: no check, quiet best move, qsearch = static eval; Leela's score agrees with the game result; top ~20% by abs(Trinity - Leela)) | -5 to +15 | -5 to +10 | ~30 min + CPU selection pass |
| 5. Drop contradictory labels (Leela > +3 pawns but lost, or < -3 but won) | +0 to +10 | +0 to +5 | ~35 min + conversion |
| 6. Teacher-student distillation (large GPU net re-scores positions for the fast net) | +5 to +25 | +0 to +15 | several hours + re-scoring |
| 7. Syzygy tablebases (3-4-5 piece) | small in self-play | +5 to +15 | engine work; library decision pending |

## Calibrations

| Date | main | akimbo 3474 | Simbelmyne 3193 | Inanis 3046 | Estimate |
|---|---|---|---|---|---|
| 2026-10-02 | net #3 (bench 920942) | 10.95% | 61.95% | 79.00% | 3238 |
| 2026-10-03 | net #4 (bench 945619) | 10.95% | 63.65% | 79.25% | 3242 |
| 2026-10-03 | net #5 (bench 705171) | 34.40% | 85.90% | 93.15% | 3411 |

From 4 October 2026 the opponents changed (rule 2: keep scores within 25-75%).

| Date | main | akimbo 3474 | Patricia 3487 | Bread 3522 | Prune 3543 | Simbelmyne 3193 (dropped) | Estimate (in-range opponents only) |
|---|---|---|---|---|---|---|---|
| 2026-10-04 | net #6 (bench 689648) | 35.10% -> 3367 | 36.55% -> 3391 | 36.15% -> 3423 | 28.55% -> 3384 | 87.25% (out of range) | about 3390 (script, all five: 3402) |
| 2026-10-05 | fresh net + Syzygy 3-4-5 for all engines (bench 735643) | 34.25% -> 3361 | 32.90% -> 3363 | 33.80% -> 3405 | 27.45% -> 3374 | - | about 3377 |
| 2026-10-05 | same + faster tablebase probing (TT storage, no FEN) | 34.95% -> 3366 | 34.25% -> 3374 | 33.95% -> 3406 | 26.75% -> 3368 | - | about 3379 (no measurable change: the slowdown cost little) |
| 2026-10-06 | king buckets (bench 550859) | 39.00% -> 3396 | 36.00% -> 3387 | 36.70% -> 3427 | 27.20% -> 3372 | - | about 3396 |
| 2026-10-09 | SPSA-tuned search (bench 637984) | 38.65% -> 3394 | 36.85% -> 3393 | 36.20% -> 3424 | 28.65% -> 3384 | - | about 3399 |
| 2026-10-10 | 1024-neuron network (bench 626889) | 44.90% -> 3438 | 42.95% -> 3438 | 44.50% -> 3484 | 34.80% -> 3434 | - | about 3449 (6 games lost on time in 4000, worth watching) |
| 2026-10-10 | same, at repeating 40/20 (300 games per opponent) | 48.17% -> 3461 | 47.33% -> 3468 | 48.67% -> 3513 | 40.67% -> 3477 | - | about 3480 (+/- 11); 0 games lost on time in 1200. Higher than at 10+0.1, not yet explained |

## Lessons so far

- Self-play gains shrink against other engines by a varying factor
  (1.3x to 6x). Treat self-play as "better or not", not "how much".
- Scores outside 25-75% give unreliable ratings; akimbo alone said 3362
  for net #5 while the weaker opponents said ~3500.
- Longer training on the same Leela positions (net #6) gained +29 in
  self-play but only about +3 against other engines: re-using data helps
  little. Fresh data (idea 1) gave +6 in self-play and nothing measurable
  against other engines: refining on more Leela data has stopped paying.
- More network capacity (king buckets) transferred almost fully to other
  engines (+19 self-play, +17 CCRL), unlike more data on the same network.
- More training on Leela data after the network has converged gives
  nothing (fresh data +6 self-play / ~0 CCRL; king buckets refined: -3.8).
- GitHub Actions SPRT matches the PC: king buckets +16.3 +/- 4.4 on GitHub
  (9000 games, 0 time losses) vs +19.1 +/- 8.5 on the PC.
- A big network trained properly (1024 neurons, from scratch, 600
  superbatches on 3.65B positions) gave +80 self-play: far more than any
  refinement. Long, single training runs beat patching.
- Search tweaks borrowed from stronger engines gave ~0 here; data and
  networks gave almost all the gains.
