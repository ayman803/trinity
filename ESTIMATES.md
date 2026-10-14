# Predictions vs results

Self-play = SPRT against the previous `main` (8+0.08, UHO book).
CCRL = calibration estimate (10+0.1, balanced book, 1 thread, rated
opponents); real uncertainty about +/- 50-100.

## Where things stand (11 October 2026)

- Strength of release 1.0.0: about 3466 at 10+0.1 (1 CPU, calibration of
  12 October, same script and opponents as before). The previous build
  (bench 626889) measured about 3449 at 10+0.1 and about 3480 at
  repeating 40/20. CCRL 40/15 list of 9 October 2026: #100 is 3455.
  Compare 3466 with CCRL's own rating of 1.0.0 when it appears: a gap
  above 50-100 means the calibration method needs fixing.
- Scout (daily idea task, from 13 October 2026): review on 27 October. If
  at least one Scout idea has passed SPRT by then, keep it daily; if none
  has, cut it to weekly. Cut it to every 2-3 days earlier if it starts
  repeating itself or reporting "No new idea".
- Goal line: the CCRL 40/15 list as published ("best versions only"), where
  a 4-CPU entry counts. The 1-CPU estimate is always reported next to it.
- Release 1.0.0: PUBLISHED 13 October 2026 (github.com/ayman803/trinity/releases/tag/v1.0.0, bench 780879). Before publishing it waited for
  the SPRT of `spsa-1024` (goes in if it passes) and the threads test
  (4 threads vs 1 on the PC: passed, +99.6, no errors, plus a 200-game
  check of the 1.0.0 build: no errors). The laptop (mixed fast and slow
  cores) also passed a 200-game check of the 1.0.0 build: no errors.
- `unsure-1024` passed (+10.8, SPRT #8) and the SPSA re-tune passed on top of it (+10.8, SPRT #10); both merged, bench 780879. This is the 1.0.0 build. The SPSA part is not counted toward the +50 or the stop rule (tuned at 8+0.08).
- Next attempt (+50 branch, separate from 1.0.0): ideas 8-11 below.
  Fable 5.1 at maximum effort designs each change; Opus does the routine
  steps. Paid from the plan's included allowance first. Cap on extra
  credit bought beyond the plan (any model): $200. Fable has its own weekly
  limit, separate from the regular one; one Fable design turn used about
  9% of it (13 October). Pause Fable at 80% of the Fable limit. Trinity's
  share of the regular weekly limit stays capped at 50% (13 October:
  about 12-18%). The owner checks usage; the Session cannot see it. Gate: if
  the first two Fable-designed changes don't beat `main`, stop paying.
  Stop rule: less than +10 on calibration over 4 weeks of steady testing
  means park the project.

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
| 2026-10-05 | corr-unsure (owner's "unsure" idea, cheap form: reduce LMR by 1 where the correction shifts eval >= 12 cp) | self-play -3 to +5 | +2.9 +/- 2.4 after 29344 games (passed, laptop, against the older main) | - | inside the prediction. Re-tested on the 1024 network as branch unsure-1024 |
| 2026-10-06 | king buckets (8 buckets, mirrored; starts from main's network, 80 superbatches on 1B unseen Leela positions, LR 0.0005) | self-play +10 to +40; CCRL +5 to +20 (about 6% slower search) | +19.1 +/- 8.5 after 2622 games (passed, merged; bench 550859) | +17 (3379 -> 3396; all four opponents up) | self-play held up almost fully (~1.1x), the best ratio so far |
| 2026-10-07 | king buckets, longer (main's KB network + 160 superbatches on the next 1B unseen Leela positions, LR 0.0005) | self-play +3 to +15; CCRL +2 to +12 | -13.8 +/- 7.9 after 2888 games (FAILED, not merged) | - | wrong. Suspected cause: restarting at LR 0.0005 undid the tuned network (refinement on fresh data at 0.0002 gave +6). Being tested: same data, LR 0.0002 sliding to near zero |
| 2026-10-07 | king buckets, gentle refinement (same 160 superbatches and the same positions as the failed run, LR 0.0002 sliding to 0.000002) | self-play -3 to +8 | -3.8 +/- 4.6 after 8016 games (FAILED, not merged) | - | the gentle rate hurt much less (-3.8 vs -13.8), so the rate explained most of the loss, but more Leela data still gives nothing on this network |
| 2026-10-07 | SPSA tune of 20 search settings (never tuned before; 5 chains x ~3 GitHub runs, 5+0.05) | self-play +10 to +30; CCRL +5 to +20 | +11.2 +/- 3.9 at 8+0.08 (9000 games, passed) and +8.7 +/- 6.3 at 30+0.3 (3000 games, LLR 1.83); merged, bench 637984 | about +3 (3396 -> 3399; within noise) | self-play overstated ~4x; most of the gain did not carry over |
| 2026-10-09 | big network: 1024 neurons, king buckets, from scratch, 600 superbatches on up to 4B Leela positions, LR 0.001 cosine, WDL 0.5 (about 28% slower search) | self-play +10 to +40; CCRL +5 to +25. Kill test: no pass by 20 October = network path used up | +80.0 +/- 4.3 at 8+0.08 (9000 games, 3380 W / 1343 L, 0 time losses; merged, bench 626889). +82.0 +/- 6.7 at 30+0.3 (3000 games, passed). Trained on 3.65B positions (whole file) | +50 (3399 -> 3449; all four opponents up 5-9 points) | far above the prediction: the earlier networks were starved of capacity and training time |
| 2026-10-11 | SPSA re-tune for the 1024 network (2 runs, 35280 games; all moves small, branch spsa-1024, bench 627481) | self-play 0 to +5; CCRL 0 to +3 | +5.9 +/- 2.7 after 18000 games (SPRT #7 + #9, passed against the main before unsure-1024) | - | the settings tuned on the 512 network mostly still fit |
| 2026-10-11 | corr-unsure on the 1024 network (branch unsure-1024, bench 805153) | self-play 0 to +4; CCRL 0 to +1 | +10.8 +/- 3.8 after 9000 games (SPRT #8, passed; merged, bench 805153) | - | far above the prediction and the laptop's +2.9 on the older main: the 1024 network's corrections may be more informative. Expect it to shrink against other engines |
| 2026-10-12 | threads: 4 threads vs 1 thread, same clock (PC, 1000 games, 10+0.1; release check) | +60 to +100 if threading works | +99.6 +/- 10.2; 0 time losses, 0 illegal moves, 0 crashes (test-results/threads-pc.txt) | not measured (only matters on the 4-CPU list) | threading works. Self-play overstates; against other engines expect perhaps +60 to +90 on 4 CPUs |
| 2026-10-13 | weight averaging (Scout idea 2): mean of checkpoints 550+600 of the 1024 run (branch net-avg-550-600-20261012-2235, bench 830311) | self-play 0 to +8 (Scout); CCRL about 70% of that | +0.4 +/- 3.9 after 9000 games (SPRT #11, LLR -0.64; stopped, not merged) | - | no gain: the late checkpoints of a run that ends at a tiny learning rate are already at the centre |
| 2026-10-13 | weight averaging, bolder: mean of checkpoints 450, 500, 550, 600 (branch net-avg-450-600-20261012-2235, bench 859329) | self-play -5 to +8 | -1.3 +/- 4.3 after 7200 games (SPRT #12, one GitHub job lost; stopped, not merged) | - | no gain |
| 2026-10-13 | tm-unsure: think up to 30% longer when the root's eval correction is large (Fable design B; bench unchanged) | self-play -2 to +6; CCRL 0 to +3 | +3.9 +/- 3.8 after 9000 games (SPRT #13, LLR 1.83; stopped, not merged) | - | no sign that the unsure signal helps beyond the extra time (+3.9 and +5.3 are statistically the same; both spent the same extra time on average, mean multiplier 1.12). Dropped; does not count for the Fable gate |
| 2026-10-13 | tm-control: 12% more soft time on every move (control for tm-unsure: its measured mean multiplier is 1.12) | self-play -2 to +4 | +5.3 +/- 3.7 after 9000 games (SPRT #14, LLR 2.82); PASSED +6.76 +/- 2.65 after 18000 games (SPRT #15, continues #14; 0 time losses) | - | simply thinking 12% longer on every move gains about +5: confirms the unspent-time leak. Passed. Time-odds ruler (Scout, 14 October): 12% more time = about +6.8 here, so a speed-up of x% is worth about 0.57x Elo in self-play. 40/20 time-loss check done 14 October (40 local games vs main: 0 time losses, 19.5/40). Merge after testing it together with nnue-simd (owner) |
| 2026-10-13 | tm-plan (part A, Fable gate change 1): the soft limit is scaled by plan/spend so far in the game (closed loop, prior 0.6, clamp 1-2.5x), and the whole increment is spent; hard limit at least 1.6x soft. Bench unchanged | 8+0.08 vs main: +4 to +10; gate test vs main of that moment (40/10 since 13 October, decided before any result; same prediction): +2 to +6, about a 50% chance to pass the capped test; CCRL-scale (40/20 calibration): +1 to +6 | pending | - | Safety check SPRT #18 vs main at 8+0.08: +4.29 +/- 3.77 after 9000 games (LLR 2.11, undecided, 0 time losses): no sign of harm; not continued (the 40/10 gate test decides). Below tm-control's +6.8 vs the same main at this clock. Validation (16 games at 10+0.1): first version overshot in sudden death, clock down to 0.09 s and 158 of 812 moves started under 1 s (main: 2); fixed by capping the loop at 1.25x and keeping 3/4 of the increment in sudden death (repeating controls keep 2.5x). PC data: Trinity spends 0.67 (10+0.1) and 0.48 (40/20) of its own plan; its rated opponents 0.84 and 0.63. Fixed-ratio alternative rejected: the ratio differs by clock, so the loop measures it |
| 2026-10-13 | nnue-simd: the network's output layer written with AVX2 instructions (same arithmetic, bench node count unchanged 780879); about 10% more nodes per second measured here (1.06M vs 0.97M) | self-play +3 to +6 (measured on 60 positions from real games at 300 ms: +7.3% nodes per second, less than the 10% on bench); CCRL +2 to +4 | SPRT #16 (+0.15 +/- 3.71, 9000 games) INVALID: the branch's only commit held the speed probe, not the AVX2 code, so both sides were the same engine (found by Scout, 14 October). Code committed 14 October (961b44b; median of 10 paired bench runs +7.7% nodes per second, ruler: about +4.4); re-test: PASSED +13.25 +/- 3.80 after 9000 games (SPRT #17, LLR 8.39, 0 time losses). Gate change 2 = gate WIN. #16 does not count for the gate. Side result: 9000 games of identical engines gave +0.15, so the GitHub setup favours neither side. The workflow now stops when both branches build the same engine file | - | v2 build (no AVX2) checked: falls back to the old code, bench 780879, no AVX2 instructions. Fable gate change 2 (owner: counts whether it passes or fails) |

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
| 8. Multi-threading: test 4 threads vs 1, fix scaling if weak | n/a (1 thread) | +60 to +100 on the 4-CPU list only | none; PC games at 4 threads |
| 9. Time management for repeating 40/15 | small at 8+0.08 | +5 to +20 | none; 40/20 calibration |
| 10. Second Leela file + longer 1024 (or wider) training | +15 to +40 | +10 to +25 | 3-4 days; needs ~200 GB free (PC has 93) |
| 11. Batch of small search ideas, GitHub SPRT | +2 to +5 each | +10 to +15 for ten | none |

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
| 2026-10-12 | release 1.0.0: + unsure-1024 + SPSA re-tune (bench 780879) | 47.00% -> 3453 | 45.75% -> 3458 | 45.80% -> 3493 | 38.45% -> 3461 | - | about 3466 (+17 over 3449; predicted +1 to +13). 3 games lost on time by Trinity in 4000 |

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

## Time management design (Fable, 13 October 2026)

Facts first, from Trinity's own games (20 self-play games at 10+0.1 here;
the calibration games on the PC are being measured with 18-Time-Use.bat):

- Baselines kill two forensics leads. Over ALL of Trinity's moves the
  median time ratio (move time / the game's average) is 0.77, the same as
  the 0.75 found for its mistakes: Trinity does not under-think its
  mistakes in particular. 80% of all moves follow a quiet opponent move,
  the same as the 77% among search mistakes. Both leads are base rates.
- Real leak: at 10+0.1 Trinity ends the game with a median 17% of its
  available time unused (its opponent copy too). The cause is the budget
  rule t/25 + 0.75 x increment: it treats every position as "25 moves to
  go" and never spends the last quarter of the increment, so time piles up
  towards the end of a game that the rule cannot see coming. Only 1% of
  moves reach the hard limit; 22% stop very early (under 35% of the soft
  limit). Moves after which Trinity's own eval then fell by 100+ cp had a
  median time ratio of 0.48, twice as often "quick" as an average move:
  the stop rule's confidence is sometimes misplaced (noted, not yet used).
- Repeating control (CCRL's kind), 7 self-play games at 40/20 here: a
  median 34% of the available time is unused at the end of the game, and
  about 7 s of each 20 s period is carried over; 40% of moves stop under
  35% of the soft limit, 1% reach the hard limit. The stop rule saves time
  readily but the savings are never spent: they only raise later budgets
  by 1/(moves to go) per move, so they pile up. Fix in A: plan the spend
  (soft = (t/H + inc) / c, with c the measured mean of move time / soft
  limit, about 0.5) so that the realised spend matches the plan.
- PC data (18-Time-Use.bat, 4000 games at 10+0.1 and 1200 at 40/20 against
  rated engines, test-results/timeuse-*.txt) confirms the picture: Trinity
  spends a mean 0.67 of its planned soft limit at 10+0.1 and 0.48 at 40/20;
  its opponents, under the same clock, 0.84 and 0.63. Unused at game end:
  19% vs the opponents' 12% (10+0.1); 43% vs 37% (40/20, inflated by
  adjudicated endings for both). Carried over per 40-move period: 6.5 s vs
  5.1 s. Per move at 11-20: Trinity 0.27 s, opponents 0.40 s, which is
  exactly Trinity's planned base: the formula is fine, the stop rule
  under-spends it. Moves after which the eval fell by 100+ cp: ratio 0.73
  vs 0.75 overall, no signal (the 0.48 seen in 20 self-play games was
  noise). The remaining-moves table (median/80th percentile by move
  number: 58/83, 48/73, 39/64, 30/55, 22/49, 22/55) is kept for a possible
  A2 (horizon model); A itself changes only the spend ratio and the
  increment, to keep the test readable.
- Trinity's games are long: median own moves still to come is about 61 at
  moves 1-10, 51 at 11-20, 43 at 21-30, 37 at 31-40, 27 at 41-60 (80th
  percentile about 17 higher). "25 to go" is far from this at every stage.

Design (three parts, each its own branch and test; from Trinity's data and
first principles, nothing taken from other engines):

- A. Horizon model (branch to come after the PC data): base time =
  time left / H + increment, where H is the expected number of own moves
  still to come, read from a small table by move number and piece count
  fitted to Trinity's own calibration games (a quantile above the median,
  so that running short stays rare). In repeating controls (CCRL's 40/15)
  H = min(moves to the control, expected remaining) + 1, so the last
  period of a game that will end early is not hoarded. Aim: the unused
  share falls from ~17% to under 8% without more time losses. Prediction:
  self-play +3 to +12 at 8+0.08 if the leak is confirmed on the PC data
  (0 to +4 if the calibration games show under 8% unused); at a repeating
  control +2 to +8; CCRL-scale (40/20 calibration) +2 to +8. Ceiling note:
  time management moves time between moves (at most about 2x on one
  move); the 60% of mistakes that 10x time fixes are a search-depth
  finding, not a time-management mandate.
- B. Unsure root (branch tm-unsure, pushed): the owner's "unsure" signal
  (|corrected - raw eval| at the root, which gave +10.8 in LMR) scales the
  soft time by up to 1.3x at a gap of 40 cp. Hypothesis: where the
  network's opinion needs the largest learned correction, evaluation is
  least reliable and depth must make up for it. Prediction: self-play -2
  to +6 at 8+0.08; CCRL-scale 0 to +3.
- C. Stop-rule calibration (later, only if A or B pass): expose the stop
  rule's constants (stability table, node-fraction and score-drop
  coefficients, the 0.6 factor, hard multiplier) to the SPSA tuner and
  tune at a repeating control such as 20/15, or measure P(best move
  changes with 2x time | signals) directly on Trinity's games and fit the
  table. These constants have never been tuned.

Gate (fixed 13 October, before any part A result): tm-control is banked
first (SPRT #15; merged if it passes). Part A is the first Fable-designed
change for the gate and must beat the main of that moment (including
tm-control if merged). Deciding test: repeating clock 40/10 (owner, 13
October, before any part A result; was 40/20), bounds [0, 5]. Game cap
written down before the test: 12000 games = 2 GitHub runs of 1200 games
per job (5 jobs); if a run's time limit cuts it short, the cap is still 2
runs. Passed = gate win; anything else at the cap = gate loss, keep main
(simpler wins a tie). An 8+0.08 run of part A is a safety check only.
Gate change 2 is nnue-simd (owner, 13 October): it counts whether it
passes or fails. If both changes fail, stop paying for Fable.
Owner decisions of 14 October, before any #17 or #18 result: if #17
(nnue-simd re-test) is undecided at 9000 games, continue it once to 18000;
that result is final for gate change 2 (passed = gate win; still undecided
or failed = gate loss, as for part A). The combined test of nnue-simd +
tm-control also gets a 40/20 time-loss check. Part A is not merged on #18
alone (#18 is against a main without tm-control; only the 40/10 gate test
decides part A).
Result note (14 October): nnue-simd gained +13.25 against a prediction of
+3 to +6 (ruler: +4.4 from +7.7% nodes per second measured here). Twice
the ruler; possible reasons: the GitHub runners' processors gain more from
the AVX2 code than this machine, or luck (the 95% range is +9.5 to +17).
Lesson for estimates: measure speed-ups on the test machines, not only here.
Checked 14 October (Speed workflow, 3 GitHub machines, AMD EPYC 7763, 10
bench runs each): nnue-simd +5.6%, +1.6%, +7.1% nodes per second, i.e. no
more than here. So the "faster machines" reason is ruled out: by the ruler,
+5% speed is worth about +3, and +13.25 is far above that. Unexplained.
The combined test #19 is an independent repeat: if nnue-simd's +13 is real,
combo should score about +17 to +20 (+13 plus most of tm-control's +6.8).
Combined test started 14 October: branch combo = main + tm-control +
nnue-simd, SPRT #19 at 8+0.08 and #20 at 40/20 (1200 games, the time-loss
check), both against main.
Prediction for combo, logged before #19 and #20 finished (Scout and
advisor, owner's paste of 14 October): self-play +10 to +18; calibration
(CCRL scale) +5 to +12, i.e. 3466 -> about 3471-3478.
Merge conditions for combo (owner, 14 October): #19 passes; #20 shows no
time losses; the x86-64-v2 build (no AVX2) gives the same bench as the
AVX2 build and plays 20 games on the PC with no crash (19-V2-Check.bat).
The v2 check logged on 13 October was on the branch without the AVX2 code,
so it does not count.
Other owner decisions (13 October): if tm-control (#15) and nnue-simd
(#16) both pass, test them together against main before merging either;
hold release 1.1 while CCRL tests 1.0.0.
Test plan: SPRT on GitHub at 8+0.08 for B and its control now; A at 8+0.08 and at 40/60
(the workflow now accepts repeating controls); CCRL-scale check of the
winners with 12-Calibrate-Repeating.bat (two runs, +/- 11 each).

## Where the time per node goes (Fable, 13 October 2026)

Measured on the AVX2 build (callgrind on `bench 8`, instruction counts) and
with a timing probe (`cargo test --release speed_probe -- --ignored
--nocapture`, kept in src/nnue.rs):
- search logic (negamax, move scoring, movegen, SEE, make_move): about
  50%; the network: evaluate 19%, accumulator updates about 10% (inlined
  in negamax), king-bucket refreshes 3%; bench's own per-position table
  clearing about 18% (memset), which games do not pay.
- evaluate cost 169 ns per call with the compiler's own vectorisation,
  83 ns with explicit AVX2 (branch nnue-simd); the accumulator update
  about 135-150 ns; a make_move 16-18 ns.
- Not pursued: lazy or deferred accumulator updates (a known technique
  elsewhere, dropped under the originality rule); caching the quiet
  history score between move ordering and LMR (under 2%).
- Conclusion for "deeper or faster": the cheap, certain gain is speed in
  Trinity's own code (about 10% now, perhaps 5% more from the update loops
  and the refresh cache). Where depth is lost in the search itself is not
  known yet; Scout's ablation audit (switch off one pruning rule at a time
  on the forensics mistake positions, 8 minutes of PC time) is the right
  next measurement before designing any search change, given that search
  tweaks have transferred poorly so far.

## Loss forensics

- 13 October 2026, release 1.0.0's calibration (200 of 1040 losses, Stockfish
  as referee at 100 ms, Trinity re-searched at 10x its time;
  test-results/forensics-2026-10-12_19-36.txt): gradual 68% (no single move
  lost a pawn by the referee's judgment), search 22% (one clear mistake
  that 10x time fixes), evaluation 6% (one clear mistake that 10x time
  does not fix), lost from the book 3%, time 1%. Search mistakes mostly
  in the middlegame and after a quiet opponent move.
- Re-run with the gradual losses tested too (same 200 losses): Trinity's 3
  worst moves in each gradual loss (drop >= 30 cp) were retried at 10x
  time: 138 fixed (search) / 107 not (evaluation). With the clear mistakes
  (43 / 8): about 60% of tested mistakes are fixed by more time, about 40%
  are not. Search mistakes were played with 0.75x Trinity's average move
  time (median); only 2 of 43 fell in the first 5 moves out of the book, so
  the "too little time just after the book" lead is not supported.
- Reading: search depth and time use matter at least as much as the
  network; earlier "mostly the network" was withdrawn. Caveats: no noise
  control yet (how often a retry at the SAME time also "fixes" a move,
  given referee noise of +/- 30-50 cp and search randomness); the margin
  for small mistakes (half the drop, at least 15 cp) is lenient.

## Release checks

- Move similarity (`trinity similarity`, 1000 positions from short random
  games, 100 ms per move, 10 October 2026). Share of positions with the
  same move:

  |          | Trinity | Reckless | akimbo | Patricia |
  |---|---|---|---|---|
  | Trinity  | -       | 78.3%    | 78.3%  | 77.4%    |
  | Reckless | 78.3%   | -        | 77.0%  | 74.9%    |
  | akimbo   | 78.3%   | 77.0%    | -      | 76.1%    |
  | Patricia | 77.4%   | 74.9%    | 76.1%  | -        |

  Trinity against itself (two runs, timing noise only): 86%. Trinity is no
  closer to any engine than the unrelated pairs are to each other (75-77%);
  all values are high because many test positions have an obvious best
  move. Stockfish to be added from the PC run (13-Similarity-Test.bat).
- PC run (13-Similarity-Test.bat, 1000 positions, 100 ms, 10 October 2026;
  Stockfish download failed, fixed afterwards):

  |          | Trinity | Reckless | akimbo | Patricia | Bread | Prune |
  |---|---|---|---|---|---|---|
  | Trinity  | -     | 77.7% | 80.5% | 77.4% | 81.2% | 77.2% |
  | Reckless | 77.7% | -     | 81.4% | 76.3% | 79.8% | 80.8% |
  | akimbo   | 80.5% | 81.4% | -     | 77.7% | 82.1% | 79.9% |
  | Patricia | 77.4% | 76.3% | 77.7% | -     | 76.6% | 76.3% |
  | Bread    | 81.2% | 79.8% | 82.1% | 76.6% | -     | 80.2% |
  | Prune    | 77.2% | 80.8% | 79.9% | 76.3% | 80.2% | -     |

  Trinity's highest match (Bread, 81.2%) is below the highest match between
  two unrelated engines (akimbo-Bread, 82.1%). No outlier.
- With Stockfish 18 (PC, 11 October 2026): Trinity-Stockfish 81.0%;
  Stockfish-Reckless 82.0%, Stockfish-Bread 82.3%, Reckless-Bread 82.3%.
  Trinity against the others in the same run: Bread 80.1%, akimbo 79.6%,
  Reckless 78.9%, Prune 77.1%, Patricia 76.2%. All within the 73-82% range
  of the unrelated pairs. No outlier.
- Caveat: this test separates engines poorly. Unrelated engines match up to
  82%, and Trinity matches itself 86%, so only about 4 points separate
  "unrelated" from "identical". The positions are too easy (many have one
  obvious move). It would catch a near-copy, not a partial one.
- Source comparison (release checklist item 1, 11 October 2026). Every Rust
  or C++ file, comments and strings removed, cut into 25-token windows;
  share of Trinity's windows also found in the other engine's source:

  | Trinity vs | exact | names and numbers renamed |
  |---|---|---|
  | akimbo     | 0.1% | 0.5% |
  | Viridithas | 0.1% | 0.6% |
  | Reckless   | 0.2% | 0.7% |
  | Patricia   | 0.0% | 0.0% |

  For comparison, Reckless vs Viridithas: 2.7% / 3.3%; Patricia vs Reckless:
  4.1% / 3.6%. Trinity's longest shared runs are boilerplate: a list of
  bench positions (5 of its 44 positions are the usual public test
  positions), a zeroed heap allocation, a prefetch call, and a row of
  println! lines. Svart could not be checked (its repository now holds only
  a README). Limits: this finds copied text, even renamed. It cannot find
  ideas or structure re-expressed in new code.
- Repeating time control 40/20: 0 games lost on time in 1200.
- Release 1.0.0 download check (PC, 12 October 2026): the GitHub-built
  `windows-x86-64-v3` file gives bench 780879 at 1.62M nps, against 1.69M
  nps for the PC's own build (4% slower, normal for a generic build); `uci`
  shows `id name Trinity 1.0.0` and `id author Ayman Mikhail`. The
  `x86-64-v2` Windows file also gives bench 780879 on the PC, at 1.28M nps
  (about 21% slower than v3, as expected without AVX2).
- CCRL 40/15 list of 9 October 2026 (best versions only): #100 is Houdini 6
  4CPU at 3455 (#99 3456, #101 3451, #102 3449). The goal line of about
  3455 stands. Trinity's 3449 (1 CPU, 10+0.1, our own calibration) would sit
  near #102; the difference is inside our measurement error (about +/- 15).
  Opponent ratings unchanged except Patricia 5.1: 3487 -> 3488.
