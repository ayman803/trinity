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
  has, cut it to weekly. REVIEWED 27 October (Ayman): KEEP DAILY. Passed:
  no-qsfut (+60, Scout's removal campaign) and no-razor (+58, from Scout's
  ablation-audit idea, tested directly by the Session after the audit
  proved too noisy). Cut it to every 2-3 days earlier if it starts
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
  9% of it (13 October). Pause Fable at 80% of the Fable limit (both coders together, from 15 October). Trinity's
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
| 2026-10-15 | wide network: 2048 neurons, same recipe as the 1024 run (3.65B Leela positions, 600 superbatches; Coder 2) | self-play +15 to +45, CCRL +5 to +25, 70% to pass | -29.76 +/- 4.16 at 8+0.08 (9000 games, SPRT #23); -19.71 +/- 6.61 at 30+0.3 (3000 games, SPRT #24); both FAILED | - | held-out loss only 1.3% lower than the 1024's; 33% fewer nodes per second. At this amount of data, width does not pay for its speed. Path closed |
| 2026-10-13 | tm-plan (part A, Fable gate change 1): the soft limit is scaled by plan/spend so far in the game (closed loop, prior 0.6, clamp 1-2.5x), and the whole increment is spent; hard limit at least 1.6x soft. Bench unchanged | 8+0.08 vs main: +4 to +10; gate test vs main of that moment (40/10 since 13 October, decided before any result; same prediction): +2 to +6, about a 50% chance to pass the capped test; CCRL-scale (40/20 calibration): +1 to +6 | pending | gate test at 40/10 vs main with tm-control: +0.71 +/- 3.40 after 10800 games, undecided at the cap: gate loss, not merged | Prediction +2 to +6 missed. On top of tm-control's 12%, the closed loop adds nothing measurable: once the 12% is in, most of the leak's value is taken. Safety check SPRT #18 vs main at 8+0.08: +4.29 +/- 3.77 after 9000 games (LLR 2.11, undecided, 0 time losses): no sign of harm; not continued (the 40/10 gate test decides). Below tm-control's +6.8 vs the same main at this clock. Validation (16 games at 10+0.1): first version overshot in sudden death, clock down to 0.09 s and 158 of 812 moves started under 1 s (main: 2); fixed by capping the loop at 1.25x and keeping 3/4 of the increment in sudden death (repeating controls keep 2.5x). PC data: Trinity spends 0.67 (10+0.1) and 0.48 (40/20) of its own plan; its rated opponents 0.84 and 0.63. Fixed-ratio alternative rejected: the ratio differs by clock, so the loop measures it |
| 2026-10-13 | nnue-simd: the network's output layer written with AVX2 instructions (same arithmetic, bench node count unchanged 780879); about 10% more nodes per second measured here (1.06M vs 0.97M) | self-play +3 to +6 (measured on 60 positions from real games at 300 ms: +7.3% nodes per second, less than the 10% on bench); CCRL +2 to +4 | SPRT #16 (+0.15 +/- 3.71, 9000 games) INVALID: the branch's only commit held the speed probe, not the AVX2 code, so both sides were the same engine (found by Scout, 14 October). Code committed 14 October (961b44b; median of 10 paired bench runs +7.7% nodes per second, ruler: about +4.4); re-test: PASSED +13.25 +/- 3.80 after 9000 games (SPRT #17, LLR 8.39, 0 time losses). Gate change 2 = gate WIN. #16 does not count for the gate. Side result: 9000 games of identical engines gave +0.15, so the GitHub setup favours neither side. The workflow now stops when both branches build the same engine file | - | v2 build (no AVX2) checked: falls back to the old code, bench 780879, no AVX2 instructions. Fable gate change 2 (owner: counts whether it passes or fails) |
| 2026-10-20 | speed-picker (speed lane): quiet moves scored only when the first is reached; quiets removed from the list once pruning skips them (bench 828267; the order changes a little because the history is fresher) | 11% fewer instructions per node; +5.6% nodes per second (median of 15 paired bench runs). Ruler: about +3. Self-play +1 to +5, about 45% to pass [0, 5]; CCRL about 60% of that | PASSED +21.45 +/- 3.83 after 9000 games (SPRT #38, LLR 14.1, 0 time losses); merged 20 October, bench 828267 | runner, 20-21 October night, 10+0.1, 4000 games each: main 3499, 1.0.0 3473 the same night: gap +26 +/- 9; the main before it had +14 +/- 8.5 (reference from 18 October), so about +12 +/- 12 from this change, inside the advisor's +5 to +15; 40/60 night queued | far above the prediction (+1 to +5). The speed part is worth about +3 by the ruler, so about +18 comes from the changed order: quiet moves are now ordered by history that already includes what the searches of the hash move, captures and killers at this node just learned. Fresher ordering information was undervalued. Expect shrinkage against other engines (the calibration decides). Advisor's prediction for the rated engines (before the calibration): +5 to +15 (self-play has shrunk 1.3-6x before). It changed the search order, not only the speed, so it also gets a 40/60 night |
| 2026-10-20 | unsure-spread (Scout 12 October, bucket spread): a node is also "unsure" (LMR one ply less) when a neighbouring output bucket's eval differs by >= 61 cp (the 80th percentile, fixed before the test). Offline gate passed: rank correlation with the eval's error 0.23-0.30 among positions within 150 cp, beyond what eval size explains (2908 positions). Costs 1.7% nodes per second. Bench 786552 | self-play -2 to +5, about 30% to pass (corr-unsure gave +10.8 on the same mechanism; this signal overlaps with it and pays 1.7% speed); CCRL about half | FAILED -4.44 +/- 3.92 after 9000 games (SPRT #39, LLR -3.96, 0 time losses); not merged | - | below the predicted range (-2 to +5). The offline correlation was real, but reducing less on 20% more nodes costs more depth than the better-placed effort earns, plus 1.7% speed. Lesson: an offline signal check is necessary, not sufficient. First Scout idea to reach a strength test |
| 2026-10-20 | lazy-noisy (lesson of speed-picker): with a hash move, every other move (captures, killers, counter) is scored only after the hash move's search; fresher capture history, and nodes that cut off on the hash move skip SEE and scoring. Instructions per node -0.8%, timing +2.3%. Bench 879832 | self-play -2 to +4, about 30% to pass (capture order is mostly decided by SEE and the victim, which do not change; the history part is small) | UNDECIDED +1.89 +/- 3.80 after 9000 games (SPRT #40, LLR 0.40, 0 time losses); stopped, not merged (a continuation to 18000 would most likely end undecided; the GitHub slot goes to new candidates) | - | inside the prediction (-2 to +4); parked: may be retried combined with a later ordering change |
| 2026-10-20 | rescore-quiets (lesson of speed-picker): at depth >= 5, the quiet moves still waiting are re-scored after the 2nd, 4th, 8th... quiet searched. Bench 804532 (2.9% fewer nodes, 0.2% fewer instructions in total) | self-play 0 to +6, about 40% to pass (the freshness effect was large once; a second dose should be smaller). Advisor, before any result: 0 to +3, since it overlaps with speed-picker's re-sorting; both kept, scored separately | FAILED -6.37 +/- 3.87 after 9000 games (SPRT #41, LLR -5.34, 0 time losses); not merged | - | below both predictions (Session 0 to +6, advisor 0 to +3). More re-sorting by fresher history hurts, so "fresher is better" does not explain speed-picker's +18 on its own; the cause of that gain is not understood yet |
| 2026-10-21 | speed-pick2 (speed lane, from the new profile): the move picker keeps the best score in hand while scanning; moves generated straight into the sorted list (no copy). Same search, bench 828267 unchanged; 4.8% fewer instructions, +4.6% nodes per second (median of 15 paired runs) | no-regression test [-4, 1] (advisor's rule for identical-bench speed-ups): about 85% to pass; true value by the ruler about +2.6 (self-play +1 to +4) | PASSED [-4, 1]: +5.91 +/- 3.68 after 9000 games (SPRT #43 + #50 + #52, LLR 4.85, 0 time losses), against the main before no-razor. Re-check on the new main PASSED: +5.27 +/- 4.46 after 6000 games (#56 + #60, LLR 2.95). Merged 22 October | - | predicted before any game |
| 2026-10-21 | tm-stable-floor (forensics: search mistakes were played with 0.75x the game's average move time): a best move that stays the same no longer shortens the think below plan (stability scale 0.9/0.8 -> 1.0/1.0). Bench unchanged (time only) | self-play -2 to +4, about 30% to pass; spends more time on "easy" moves, so later moves get less in sudden death; 40/15 may gain more than 8+0.08. Advisor (21 October, before any result): also test at 40/10 (repeating, as CCRL), as for tm-plan; the 0.75x clue may be confident-and-wrong positions that more time does not fix. Decision rule, fixed now: merge only if the 40/10 test passes ([0, 5], cap 12000 games) and the 8+0.08 test does not fail. 40/10 prediction: -3 to +4 (the old main's +14 at 10+0.1 shrank to +4 +/- 13 at 40/60, so time gains have not carried to repeating controls) | 8+0.08: +2.20 +/- 3.76 after 9000 games (SPRT #44, LLR 0.64), undecided, not failing; 40/10: +5.06 +/- 4.25 after 7000 games (#45 + #51, LLR 2.09; it fell back from 2.93); FINAL at the 12000 cap: +3.13 +/- 3.23 (#57, LLR 1.75), undecided. Not merged (rule fixed before: the 40/10 test had to pass). The 0.75x clue did not turn into a clear gain | - | predicted before any game |
| 2026-10-22 | no-razor (ablation audit, 21 October): razoring removed. Audit net +42 at equal nodes, but only about +21 +/- 28 above the no-change baseline. Bench 892727 | self-play -4 to +4 (weak evidence; razoring was never tested on its own in Trinity). Advisor (21 October, before any game): test it as a simplification, no-regression bounds [-4, 1]; keep razoring removed if not worse (one rule fewer to tune). About 65% to pass that test | PASSED, far beyond any expectation: +58.33 +/- 6.91 after 3000 games (SPRT #49, [-4, 1], LLR 13.48; pairs 9 / 192 / 642 / 605 / 52; 0 time losses) | runner, 21 October by day (idle PC), 10+0.1, 4000 games: no-razor 3538 (akimbo 59%, Patricia 56%, Bread 56%, Prune 47%). Vs main of the night before (3499): +39 +/- 9 (above the advisor's +15 to +35). Gap over 1.0.0: +65 +/- 9, reference from 21 October (the same-day 1.0.0 run was disturbed at 5:02 pm and reruns). Same-night rerun (check-in 22 Oct): no-razor 3542 (2026-10-21-03, 10:43 pm, idle) next to 1.0.0 3473 (2026-10-21-02, 8:21 pm, idle): gap +69 +/- 9. 40/60 (main = speed-picker + no-razor, 22 Oct 1-5 am): +46 +/- 22 over 1.0.0, so the gain shrinks at the slower clock but stays large | prediction badly wrong (-4 to +4). Likely mechanism: with eval 500-1050 cp below alpha at depth 1-3, razoring gave up unless a capture saved the position, so Trinity could not see quiet attacking moves after a sacrifice (or defences against one), the tactics that decide self-play games. The audit hinted (net +42) but its noise hid the size. Lesson: never-tested inherited rules are worth testing one by one. Merged 21 October 12:35 pm (e0da35a, bench 892727; the running GitHub tests had all built the earlier main at 12:12). Advisor's prediction for the rated engines: +15 to +35 | - | predicted before any game; started 21 October, counted as 22 October's first |
| 2026-10-22 | no-iir (Scout's removal campaign: each rule from the first commit, never tested alone, removed one at a time): no internal iterative reduction. Audit net +11 (noise level +20). Bench 1001780 | self-play -6 to +4; no-regression test [-4, 1]: about 40% to pass (keep removed if not worse) | -2.72 +/- 4.57 after 6000 games (SPRT #54 + #58, LLR -0.93), undecided and leaning negative; stopped 22 October: IIR stays (a removal that looks slightly harmful is not worth more games) | - | predicted before any game; on main with no-razor |
| 2026-10-22 | no-fut (removal campaign): no futility pruning of quiet moves. Audit net -6 (noise). Bench 1159611 (+30% nodes) | self-play -10 to +3; [-4, 1]: about 25% to pass | FAILED -7.07 +/- 4.60 after 6000 games (SPRT #55 + #59, LLR -2.98): futility pruning stays | - | predicted before any game; on main with no-razor |
| 2026-10-22 | no-qsfut (removal campaign): no futility pruning in quiescence. Audit net -7 (noise). Bench 916512 | self-play -6 to +3; [-4, 1]: about 35% to pass | PASSED, far beyond the prediction: +59.76 +/- 6.76 after 3000 games (SPRT #61, LLR 14.13; W 981, L 470; 0 time losses). Merged 22 October (bench 916512, with speed-pick2). Second +60 removal: razoring and QS futility both gave up when the static eval was far below alpha, cutting tactics the eval cannot see (quiet futility, the same shape at depth, held: -7 when removed). Checked for a shared fault (piece values, en passant, eval scale): none found. Clear gain, so a 40/60 night (advisor's guard); calibration queued tonight | - | predicted before any game |
| 2026-10-22 | no-negext (removal campaign): singular extension keeps its +1 and multi-cut but no longer searches the hash move a ply shallower when another move also beats the singular bound. Bench 869069 | self-play -5 to +4; [-4, 1]: about 45% to pass | 3000 games: +0.93 +/- 6.47 (SPRT #62, LLR 0.41) on the earlier main; on the main with no-qsfut -3.24 +/- 6.29 after 3000 (SPRT #63, LLR -0.61). Stopped 23 October, leaning negative: the negative extension stays | - | predicted before any game |
| 2026-10-23 | rfp-wide (advisor's eval-margin idea, 22 October): reverse futility margin doubled, 85 -> 170 (prunes less, not removed; the audit put RFP off at -61). Bench 1408587 (+54% nodes) | self-play -6 to +12, about 35% to pass [0, 5]; both SPSA tunes moved this margin up (75 -> 81 -> 85) | FAILED -10.16 +/- 3.75 after 9000 games (SPRT #65, LLR -8.19): RFP's margin was about right; the "ahead, stop" cutoff is not over-pruning | - | predicted before any game; started 22 October as 23 October's pair; not merged before the 1.1 freeze |
| 2026-10-23 | fut-wide (same idea): main-search futility margins doubled, 90 + 104 x depth -> 180 + 208 x depth (removing it lost -7). Bench 1012002 | self-play -5 to +8, about 30% to pass (SPSA barely moved these: 90 -> 91 -> 90, 100 -> 102 -> 104) | PASSED +5.75 +/- 3.75 after 9000 games (SPRT #66, LLR 3.18). Left for 1.2 (owner); merged into main after the freeze | - | predicted before any game; same pairing |
| 2026-10-23 | lmp-wide (for 1.2; pattern: early give-up cutoffs over-prune): late-move pruning base doubled, 359 -> 718 (about 3.6 more quiet moves at every depth). Bench 1129860 | self-play -6 to +8, about 30% to pass [0, 5]; SPSA moved it up 300 -> 349 -> 359, and the audit found LMP off clearly harmful (-36) | FAILED -5.37 +/- 3.74 after 9000 games (SPRT #67, LLR -4.77): the late-move pruning count is about right | - | predicted before any game |
| 2026-10-23 | fut-wide2 (for 1.2): futility margins doubled again, 360 + 416 x depth (the first doubling gave +5.8). Bench 987234 | self-play -4 to +5, about 25% to pass (a second doubling should give less than the first) | -2.12 +/- 3.71 after 9000 games (SPRT #68, LLR -2.47), undecided and leaning negative; stopped 24 October: futility stays at the first doubling | - | predicted before any game |
| 2026-10-26 | lmr-behind (Scout, 26 October): late moves reduced one ply less when the static eval is 300+ below alpha (Scout's offline check: 200 touched 28% of reduced moves, over its 20% limit; 300 touches 19.7%). Bench 1056549 | Scout: self-play +2 to +8, about 40%. Session: -3 to +6, about 35% (overlaps with the removals, which already search "behind" nodes more) | FAILED -3.98 +/- 3.61 after 9000 games (SPRT #69, LLR -3.88): below both ranges; the "behind" blind spot paid off through the two removals, not through reductions | - | predicted before any game; if it passes, also 40/10 |
| 2026-10-26 | rfp-tight (eval-bias table, the "ahead" side): reverse futility margin 85 -> 65, prunes more when ahead (widening it lost -10). Bench 924219 | self-play -4 to +5, about 30% to pass | FAILED -4.01 +/- 3.68 after 9000 games (SPRT #70, LLR -3.85): RFP's margin of 85 is right on both sides (wider -10, tighter -4) | - | predicted before any game |
| 2026-10-27 | hist-prune2 (Scout, 27 October: re-test old near-zero ideas on today's engine): history pruning as on 3 October (depth <= 4, history below -2500 x depth). Then +0.3 +/- 4.1 on a far more pruned engine. Bench 1001215 | self-play -3 to +5, about 25% to pass; the engine now searches 30% more nodes per depth, so a cheap prune has more to save, but this week every "prune more" change lost (rfp-tight -4) | -1.00 +/- 3.69 after 9000 games (SPRT #72, LLR -1.66), undecided, leaning negative; stopped: still about zero on the current engine | - | predicted before any game |
| 2026-10-27 | split38-drop (research, the other half of the #38 split): speed-picker with lazy quiet scoring switched off (only dropping pruned quiets), vs the main before #38. Lazy-only half: +3.62 +/- 4.28 after 7200 games (#46 + #71). Bench 775818 | +2 to +18: the pure speed part predicts about +2; if the two halves add up, +21.5 - 3.6 = +18. A result near +2 would mean #38's +21.5 was partly luck or the halves interact | PASSED +18.59 +/- 3.88 after 9000 games (SPRT #73): dropping carries nearly all of #38. Mechanism checked 28 October: the search is the same (bench nodes 780879 vs 775818, tie order only), but on 60 positions from real games at 300 ms this build is 18% faster (median), while the bench showed only 5.6% for all of #38. It is speed, not extra pruning (Scout read it as pruning more). LESSON: measure speed on game positions, not the bench; the move loop costs far more in real games | - | predicted before any game; research, not a candidate |
| 2026-10-27 | nonpawn-corr2 (Scout, 27 October re-test): correction history also keyed by each side's non-pawn pieces (4 October: +1.0 +/- 2.7 after 21k games on the old engine). Bench 982007 | self-play -2 to +5, about 25% to pass; corr-unsure later showed the correction signal is worth a lot (+10.8), and a second key adds detail, but it also makes more nodes "unsure" (less reduction), whose cost is unknown | +3.24 +/- 3.73 after 9000 games (SPRT #74, LLR 1.39), undecided; continued to 18000 (28 October) | - | predicted before any game; the day's second candidate (the #38 split is research) |
| 2026-10-28 | lmp-early (Scout, 28 October): late move pruning starts sooner, LMP_BASE 359 -> 260 (about one quiet move fewer before pruning when improving, half a move when not); the opposite of lmp-wide (-5.4). Bench 895637 (-11.5% nodes) | self-play -5 to +4, about 12% to pass [0, 5] (Scout: -2 to +6). Against it: SPSA left LMP_BASE where it is (chains C/D 355 and 368), and this week every "prune more" change lost (rfp-tight -4); for it: fewer nodes is speed, and #38 showed speed is worth more than the bench says | pending | - | predicted before any game; ranges widened 1.5x and pass chance scaled by 0.6 (forecast lesson of 27 October) |
| 2026-10-28 | quiet-skip (speed lane, from the game-position profile): late move and futility pruning are checked before the quiet moves are scored; neither rule depends on which quiet move comes next, so when one applies at the first quiet move, all of them are skipped unscored. Same search, bench 1012002 unchanged. On 17 game positions: 1.1% fewer instructions (quiet scoring -18%); wall clock on 60 game positions +0.3% median, +0.6% mean | self-play -1 to +2; [-4, 1]: about 75% to pass (a same-search change passes about 95% of the time if it is not slower; scaled down for my record) | pending | - | predicted before any game; pure speed, so the short no-regression test (advisor, 20 October) |

## Test queue

Lanes: search and network, both the Session's since 16 October (Coder 2
folded back after the 2048 network failed). GitHub: at most 2 search tests and 1 network test
at once. PC: one heavy job at a time, in the order Ayman chooses.

Pace rules (owner with the advisor, 20 October; tests #28-#37 were all
network, and the second week merged nothing):
- Network lane: about 20% of GitHub test time at most, one experiment at
  a time. Current one: KB16-Seed2 (stopping rule: no third 16-bucket run
  unless Seed2 passes against main). Main changed after Seed2 started
  (speed-picker, +21.5), and Seed2's published branch carries the older
  search: before its screen, merge main into that branch (checked: merges
  cleanly), so the test compares networks only.
- Search/speed lane: at least 2 new candidates tested per day, from the
  forensics, Scout's list and the speed profile below. Predictions in the
  Changes table before any game.
- Pure speed-ups with an identical bench get a short no-regression test
  (SPRT bounds [-4, 1], 600 games per job, continued if undecided)
  instead of a full [0, 5] test (advisor, 20 October). A changed bench
  gets the full test.
- Calibration nights for candidates that passed on GitHub come before
  runner tuning nights (docs/spsa-runner-design.md).
- Runner tuning (planned work, built 20 October, commit ff3d4a6;
  approved by Ayman 20 October, commit 58841ba): on a night with no
  candidate calibration and no 40/60 pair queued, queue
  `scripts/calibrate.ps1 | -Tune -Chain A -Minutes 270` and then the same
  with `-Chain B`. Stop after 6 nights per chain, or at night 3 if chains
  A and B move fewer than 3 of the 4 settings the same way. Prediction:
  CCRL scale 0 to +6, about 35% to pass Scout's rule; the unseen-opponent
  check decides.
- Removal campaign (Scout, 21 October): every pruning or extension rule
  came with the first commit (30 September) and none was tested alone;
  razoring's removal gave +58. Each remaining rule is removed one at a
  time and tested with [-4, 1] (kept removed if not worse), two a day,
  predictions first. Order: IIR, futility (22 October), then QS
  futility, singular extension's parts (negative extension, multi-cut),
  SEE quiet pruning, null move, SEE capture pruning, QS SEE, LMP, RFP.
  Guards (advisor, 21 October): a removal with a clear gain also gets a
  40/60 night; when the campaign ends, the engine with every removal
  plays the pre-campaign main (branch pre-campaign = e0da35a, main with
  no-razor) at 40/10, to catch small "not worse" losses adding up.
- Scout's pointer (22 October): the removed razoring had no mate-score
  guard. Checked the remaining rules: RFP skips mate-range evals; null
  move returns beta instead of an unproven mate; LMP, futility and SEE
  pruning wait until one move avoids being mated (best_score >
  -TB_BOUND); singular extension skips mate-range hash scores. Futility
  and QS futility do not check whether alpha is already a mate score (they
  could skip a quiet move that mates faster); harmless for the result,
  noted as a possible later candidate. Never bring razoring back without
  a mate guard.
- Tests measured against the main before no-razor (tm-stable-floor 40/10
  #45/#51, speed-pick2 #43/#50/#52, Seed2 #42/#47/#53): any that passes
  gets a short no-regression test ([-4, 1]) on the new main before it is
  merged (advisor, 21 October).
- Same-night gap (advisor, 21 October evening): no-razor is calibrated
  again tonight (2026-10-21-03) next to 1.0.0's rerun (2026-10-21-02),
  so the gap over 1.0.0 at 10+0.1 is a true same-night measurement; if
  the night is too short it finishes in the next idle hours.
  Result (check-in 22 October): 1.0.0 3473 (8:21 pm), no-razor 3542
  (10:43 pm), both idle throughout: gap +69 +/- 9 at 10+0.1.
- Seed2 stopped (21 October 5 pm): +1.24 +/- 3.46 after 12000 games (SPRT
  #42 + #47 + #53, LLR -0.13): a pass by the 18000 cap was unlikely
  (under 10%), and the network lane is capped at about 20% of GitHub
  time. Not merged; by the stopping rule (owner, 20 October) no third
  16-bucket run and no more random starts. The next network test fixes
  the data and the seed so only one change differs (to be designed).
- Runner tuning paused while the removal campaign changes the search
  (tuning a search that changes daily wastes the nights); it restarts
  with chains C and D once the campaign settles.
- Release 1.1 plan (owner, 22 October: "freeze Friday"; advisor's
  additions). Correction first: only one same-night 10+0.1 gap so far is
  at +50 or more (+69, 21-22 October); the +65 used another night's 1.0.0.
  1. Friday 23 October, evening (when Ayman sends "freeze now"): merges
     stop; branch release-1.1 = main at that moment.
  2. Friday night on the runner, in this order (about 6.5 hours, fits
     before 6:55): calibration of release-1.1 and of release-1.0.0 at
     10+0.1 the same night; threads check (threads.ps1 -Ref release-1.1
     -Rounds 100: 4 threads vs 1, 200 games); v2 build check (v2check.ps1
     -Ref release-1.1). Move similarity in the cloud.
  3. Saturday night: 40/60 pair of release-1.1 and release-1.0.0.
  4. The numbers published with v1.1.0 come from these runs of the frozen
     build only. TAG ONLY (owner, 23 October): the README promises testers
     that "the next release won't come before 1.0.0 has a CCRL rating", so
     there is no release page, no downloads and no README change. The
     cloud session cannot create tags (git and the API both refuse), so
     the frozen build is marked by the branch release-1.1 (47a89f9, never
     to be moved). When 1.0.0 is rated: run the release workflow (on main)
     with version 1.1.0, commit 47a89f927e56876a90687479c85ba857546ff968
     and bench 916512; it builds that commit, not a branch, and
     stops if any build's bench differs
     (advisor, 23 October), otherwise it creates the v1.1.0 tag and a
     draft release page on that commit.
  Gate added (Scout and advisor, 22 October): before release, the frozen
  build needs a 40/10 time-loss check (at least 1000 games, 0 losses)
  and tonight's 40/60 result; if either is missing, release the version
  measured at +44 at 40/60 (the no-razor main) instead. 40/10 check of
  the current main started 22 October (main vs pre-campaign, 1200 games;
  it also measures no-qsfut + speed-pick2 at a repeating control).
  The 1.1 candidate is the current main (bench 916512); tests that pass
  after this go into 1.2, so the checks describe exactly what is released.
  Forecasts for tonight, logged before the runs (advisor's correction:
  no-qsfut's +60 was measured on top of no-razor): same-night 10+0.1 gap
  over 1.0.0 +90 to +115 (advisor; no-razor kept about 70% against rated
  engines); under +80 would mean the two removals overlap against rated
  engines. Session: +85 to +115, central +100. 40/60 of main tonight:
  +55 to +85 over 1.0.0 (reference from 22 October, 3504).
  RESULTS (check-in 23 October; PC idle throughout, 0 time losses):
  10+0.1 same night: main 3574 (2026-10-22-03, 5:58 pm), 1.0.0 3472
  (2026-10-22-04, 8:21 pm): gap +102 +/- 9, inside both forecasts and
  at the Session's centre (+100); the two removals did not overlap
  against rated engines. 40/60: main 3570 (2026-10-22-05, 10:42 pm):
  +66 +/- 21 over 1.0.0, reference from 22 October (3504), inside +55
  to +85. Gate: the 40/10 time-loss check (SPRT #64, main vs
  pre-campaign, 1200 games) had 0 time losses, 0 crashes, and passed
  [0, 5] at +51.0 +/- 10.0; with tonight's 40/60 result, both gate
  conditions are met for the current main (bench 916512). Skeptic: main
  now scores about 64% against akimbo and Patricia at both controls and
  is above Prune too; all four are still inside 25-75%, but the next
  gains will push akimbo out, so a stronger opponent will be needed
  soon (rule 2).
  FROZEN 23 October 6:30 am (Ayman: "freeze now. Leave fut-wide for
  1.2"): branch release-1.1 = 47a89f927e, version 1.1.0, bench 916512 (the
  same engine as the build measured at +102 / +66; only the version name
  changed). Gates met: 40/10 check #64, 1200 games vs pre-campaign, 0 time
  losses, +51.0 +/- 10.0; 40/60 of this engine +66 +/- 21 (reference from
  22 October). Queued (2026-10-23-01..06): calibration of release-1.1 and
  1.0.0 at 10+0.1, v2 check, the 40/60 pair, the threads check (its 8-hour
  limit is its only length estimate, so it runs when that fits).
  RESULTS (23 October, by day on the idle PC, so they count, rule 13):
  - 10+0.1, same day, 4000 games each: release-1.1 3570 (akimbo +87,
    Patricia +80, Bread +70, Prune +16), release-1.0.0 3480: gap +90 +/- 9
    (the previous night's main, the same engine: +102 +/- 9; the two
    agree within their noise). All opponents still between 25% and 75%.
  - v2 build check: PASSED (same bench 916512, 20 games, no time losses,
    illegal moves or crashes; the v2 build is 32% slower, as expected).
  - 40/60 pair, same day (check-in 24 October): release-1.1 3573
    (2026-10-23-04, counted run 5:05-7:11 pm after two DISTURBED starts;
    akimbo +101, Patricia +83, Bread +73, Prune +9), release-1.0.0 3510
    (2026-10-23-05, 7:12-9:10 pm): gap +63 +/- 21. Inside the +55 to +85
    forecast for this engine and close to the previous night's +66 (reference
    from 22 October). 0 time losses for either. Pooled with that night:
    about +64 +/- 15 at 40/60.
  - Threads check (2026-10-23-06, 9:10-9:53 pm): 4 threads vs 1, 200 games
    at 10+0.1: +79.5 +/- 19.3 (46 wins, 1 loss, 153 draws), 0 time losses,
    illegal moves, crashes or hangs. PASSED (prediction +60 to +100, as for
    1.0.0's +99.6 +/- 10.2 in 1000 games). Skeptic: the lower figure fits
    200 games' noise together with more draws as the engine gets stronger;
    no sign of a threading fault.
  Release notes (advisor, 23 October): headline the same-day 40/60 gap
  over 1.0.0 (the clock closest to CCRL's), with the 10+0.1 gap second;
  word every number as our own measurement ("about +N over 1.0.0 in our
  tests at 40 moves/60 s against four rated engines"), never as a CCRL
  rating. No mention of CCRL testing, submissions, testers or any queue
  (advisor, 23 October): nothing that could read as pressure on the
  tester. Only our own measurements.
  5. Settled 23 October: 1.1 is not offered to the CCRL tester (see the
     CCRL decision above).
- CCRL DECISION (Ayman, 23 October; settled, do not raise again): 1.0.0
  stays with the CCRL tester as agreed; no change is requested and no
  message goes to the tester. Keeping the tester's goodwill matters more
  than an earlier, higher listing. Newer versions wait until 1.0.0 is
  rated and Ayman decides to offer one (by then main will be stronger
  than 1.1). 1.1 is still published on GitHub as planned. The check-in,
  Scout and the phone summary do not bring up 1.1-for-CCRL.
- Process rules adopted 23 October (owner with the advisor; the
  Session's verdicts): (1) small candidates (central prediction under +4)
  are bundled 3-4 to one SPRT; a passed bundle gets one quick split test
  per member before it counts (a harmful member must not ride along);
  big or uncertain ideas still go alone. (2) When 3 or more candidates
  are ready, run the one with the highest expected value per test-hour
  first (pass chance x central gain / expected games), from the logged
  predictions. (3) Release robustness suite (release/README.md) on every
  release and on any version a CCRL tester uses. (4) Forecast scoring:
  for the 27 October review, how often each forecaster's result fell
  inside its predicted range (Session, Scout, advisor; small samples for
  the latter two).
- Runner plan from 24 October (owner with Scout): (a) same-night
  calibration of main (1.1 + fut-wide) and 1.0.0 at 10+0.1, then the
  40/60 pair; (b) tuning chains C and D resume (tuning last each night).
  Predictions: 10+0.1 gap over 1.0.0 +88 to +108, central +99 (1.1
  measured +102 and +90; fut-wide's +5.8 self-play should be about +3
  here, which one night's +/- 9 cannot resolve: these runs track main,
  they do not settle fut-wide alone); 40/60 gap +55 to +80 (1.1: +66,
  +63). Tuning list changed 24 October: QS_FUTILITY (removed from the
  search) -> FUT_MUL; needs Ayman's approval, otherwise the chains run
  the approved list with one dead setting (harmless: it does not affect
  the games). Tuning prediction unchanged: rated scale 0 to +6.
  RESULTS (check-in 25 October; PC idle in every counted run, 0 time
  losses): 10+0.1, same day: main 3581 (2026-10-24-01, 10:30 am; akimbo
  +105, Patricia +77, Bread +82, Prune +29), 1.0.0 3470 (2026-10-24-02,
  counted run 6:09-8:29 pm after one DISTURBED start): gap +111 +/- 9,
  3 above the +88 to +108 range, inside its noise (1.1's +102 and +90
  plus fut-wide's expected ~+3). 40/60, same night: main 3564
  (2026-10-24-03), 1.0.0 3501 (2026-10-24-04): gap +63 +/- 21, inside
  +55 to +80 (1.1: +66, +63; pooled over the three 40/60 nights of this
  engine family about +64 +/- 12). Skeptic: main now scores about 65%
  against akimbo at 10+0.1 (+105) and is above all four opponents; still
  inside 25-75%, but one more step like no-razor would push akimbo out
  (rule 2). Tuning: chain C ran 267 min (96 steps, 1344 of 11000 pairs);
  chain D started 5:03 am (91 minutes, cut to end by 6:55). Note: both
  ran under approved commit 189246b, so they still tune QS_FUTILITY, not
  FUT_MUL: the 24 October approval has not reached the runner (its log
  still names 189246b). Harmless for the games (the setting is dead), but
  FUT_MUL is not being tuned until Approve-Scripts is run on the PC.
  Approval took effect 25 October 6:40 am (commit 272a0f3; the 24
  October one had not registered: chain C's first night, 5376 games,
  still tuned the dead QS_FUTILITY). From 25 October the chains tune
  FUT_MUL; C and D continue from their saved states (FUT_MUL starts at
  its default). Queued 25 October: C, then D.
  RESULTS (check-in 26 October): chain C 2026-10-25-01 (one DISTURBED
  start at 1:49 pm, counted run 6:16-10:43 pm, 267 min) and chain D
  2026-10-25-02 (10:43 pm-3:12 am, 269 min), both under 272a0f3 (FUT_MUL
  tuned). Totals: C 2926 of 11000 pairs (209 steps, its 2nd night), D 1820
  (130 steps; night 1 was only 90 min). Values vs main's defaults:
  FUT_MUL 208 -> C 206.6 / D 209.0; LMP_BASE 359 -> C 355.0 / D 367.9;
  LMR_HIST_DIV 7860 -> C 7799 / D 7638; RFP_MARGIN 85 -> C 85.9 / D 85.0.
  Only LMR_HIST_DIV moves the same way (down) in both chains so far;
  the others are within a step or two of default or split. If that holds
  after night 3 (tonight), the stop rule (fewer than 3 of 4 the same way)
  ends the chains. Caveat: FUT_MUL still ran on the chain's late step
  schedule; Scout's fix (ec90c7e, own schedule per setting) is not yet
  approved on the runner, so FUT_MUL's small moves say little yet.
  Scout (25 October): the chains kept their pair count when the list
  changed, so FUT_MUL started late in the step schedule (test nudges
  about half size, learning rate about 60%, 88% of the plan left).
  Fixed in tune-opponents.ps1: each setting now counts its schedule from
  when it joined the chain ("since" in the saved state; dry-run checked).
  Needs one approval (26 October); until then FUT_MUL runs on the late
  schedule, which is slower but not wrong.
  Status 26 October (Session), after 2 nights (C: 11704 games, D: 7280):
  moves from the defaults in step sizes: FUT_MUL C -0.2 / D +0.1, LMP_BASE
  C -0.1 / D +0.2, LMR_HIST_DIV C -0.1 / D -0.4 (the only agreement),
  RFP_MARGIN C +0.1 / D 0.0. Every move is under half a step: the chains
  are still at noise level, as the design review expected (a game against
  a rated opponent carries about a quarter of the information of a
  self-play game). The stop rule (fewer than 3 of 4 agreeing after night 3)
  is applied on 27 October as agreed; FUT_MUL joined a night late, but
  its moves are as small as the others', so that does not change the
  verdict. The per-setting schedule fix is not worth an approval before
  then: if the rule stops tuning, it is moot; if not, approve it then.
  NIGHT 3 AND STOP RULE (check-in 27 October): chain C 2026-10-26-01
  (7:46 am-12:14 pm, 268 min) and chain D 2026-10-26-02 (one DISTURBED
  start, counted run 7:10-11:38 pm, 269 min); 0 time losses in either
  chain. Totals: C 4284 pairs (17136 games), D 4480 pairs (17920 games).
  Moves from the defaults, in step sizes: FUT_MUL C -0.4 / D +0.2;
  LMP_BASE C -0.3 / D +0.3; LMR_HIST_DIV C -0.1 / D -0.6; RFP_MARGIN
  C +0.1 / D +0.2. Same direction in both chains: LMR_HIST_DIV (down) and
  RFP_MARGIN (up, both under a quarter step), so 2 of 4, fewer than 3: by
  the stop rule fixed in advance, chains C and D STOP. Nothing is applied
  to main (no setting moved even one step). Prediction (rated scale 0 to
  +6, about 35% to pass) not tested: stopped before a result. Lesson:
  against rated opponents, three nights (about 17000 games per chain) are
  not enough for SPSA to move settings already tuned by self-play; the
  per-setting schedule fix (ec90c7e) is moot and needs no approval. By
  the plan of 26 October, runner nights now go to calibrating candidates
  that pass.
- Search lane idle 23 October 5:43 pm to 26 October (release checks and
  tuning took the attention; the pace rule was missed for two and a half
  days). Restarted 26 October: lmr-behind, rfp-tight, and the #38 split
  continued (research). Pace alarm added to CLAUDE.md (the check-in
  reports "Decision needed: search lane idle" when no candidate test
  started in the last 24 hours, unless a deliberate pause is recorded
  here). If the tuning stop rule fires on 27 October, the runner nights go
  to calibrating candidates that pass (owner with the advisor).
- PC and GPU plan from 27 October (owner with the advisor; tuning stopped):
  (1) PC as a second test farm: scripts/sprt.ps1 -Rounds 1000 plays one
  chunk of 2000 games (about 1 hour at 8+0.08, 14 at a time) and publishes
  its counts to trinity-results sprt/<test>/; the Session adds a test's
  chunks up with scripts/sprt_combine.py (cloud dry run 27 October: chunk
  published and combined). A disturbed chunk loses at most an hour.
  Capacity: about 9 chunks a night, 18000 games, about two full tests.
  Skeptic note: GitHub runs up to 4 tests at once around the clock, so the
  PC adds about 10-25% capacity, not double; the lane's bottleneck has
  been candidates and attention, not machines. Its real value: busy days,
  and slower clocks (30+0.3, 40/10) that are slow on GitHub.
  (2) Weekly same-night calibration of main next to 1.0.0 (10+0.1), queued
  by the check-in on Fridays (CLAUDE.md).
  (3) Next network experiment, one change: main's recipe trained 900
  instead of 600 superbatches, bullet's default seed and the same four
  data files. Main's network was trained with exactly that seed and data
  (the drift check of 20 October showed today's trainer reproduces it), so
  the comparison with main differs only in length; no control run needed.
  Prediction (widened per the forecast scores): self-play -4 to +12, about
  20% to pass. GPU about 7.5 hours as a runner job (tools/net/kb16.ps1
  -KingBuckets 8 -Superbatches 900 -Tag long900), after a 5-minute dry-run
  job checks that training works under the runner's account (CUDA in a
  background session is untested). The network goes to trinity-results;
  the Session makes its branch. Training stays subject to the
  keyboard/mouse rule; the 6:55 deadline keeps it to nights.
  Needs: one runner reinstall (runner.ps1 and Approve-Scripts.ps1: the two
  scripts added to the runner's list and their time limits, 2 and 9
  hours; it also brings the opponent-copy fix of 21 October) and one
  approval (scripts/sprt.ps1, tools/net/kb16.ps1).
- Weekly count (for the phone summary): week of 19 October: tested 4
  (speed-picker, unsure-spread, lazy-noisy, rescore-quiets; the last two
  started 20 October evening as 21 October's pair: +1.9 undecided,
  -6.4 failed), merged 1 (speed-picker, +21.5; calibration gap over 1.0.0
  +26 +/- 9, 21 October). Updated by the check-in 22 October: tested 9 search
  candidates (adding speed-pick2, tm-stable-floor, no-razor, no-iir,
  no-fut; plus Seed2 in the network lane), merged 2 (speed-picker;
  no-razor, +58 self-play, same-night gap over 1.0.0 +69 +/- 9 at
  10+0.1 and +46 +/- 22 at 40/60). Updated by the check-in 23 October:
  tested 13 (adding no-qsfut, no-negext, rfp-wide, fut-wide), merged 4
  (adding speed-pick2 and no-qsfut, +60). fut-wide passed (#66, +5.75
  +/- 3.75) but is not merged; it was marked for after the 1.1 freeze.
  rfp-wide failed (#65, -10.2). Same-night gap of main over 1.0.0:
  +102 +/- 9 at 10+0.1.
  Week of 26 October (check-in 27 October): tested 2 (lmr-behind
  -4.0 failed, rfp-tight -4.0 failed; plus the #38 split, research,
  undecided), merged 0. Updated 28 October: tested 4 candidates
  (adding hist-prune2 -1.0 and nonpawn-corr2 +3.2, both undecided after
  9000 games; plus split38-drop, research, passed +18.6 against the
  base before #38), merged 0.

### Speed profile on game positions (28 October; owner with the advisor)

After #38's lesson (dropping pruned quiets: 18% faster on game positions,
5.6% on the bench), the engine was profiled on 17 positions from
calibration games (150000 nodes each, callgrind, AVX2 build). Where the
instructions go: accumulator updates about 18% (inlined in the search),
search logic 12%, SEE 6.2% (bench 3.9%), move scoring 6.3%, quiet
history scoring 4.9%, evaluate 7.4%, move generation 6.5%, king
refreshes 3.3%, make_move 3.2%, memcpy 1.7%.

Three speed ideas from it, measured before any game (wall clock: 60 game
positions, 400000 nodes, both builds alternated, ratio new/old):
1. lazy-see: test a capture's SEE only when it comes up for search.
   Only 18% of the SEE calls are saved (most captures that are generated
   are reached), and the re-scan after a failed test costs more:
   instructions +0.5%. Dropped, no games.
2. quiet-later: generate quiet moves only after the hash move and the
   winning captures (38% of nodes cut off before any quiet move; only 6%
   have a quiet hash move). Instructions -3.7%, but wall clock -0.7% and
   -1.0% in two runs: fewer instructions, not faster. Dropped, no games.
   (It is also a textbook method, close to the deferred accumulator
   update dropped under the originality rule; that question is moot.)
3. quiet-skip: check late move and futility pruning before scoring the
   quiets. Same search; instructions -1.1%, wall clock +0.3%. Tested
   (Changes table).
Re-measured on game positions: lazy-noisy (20 October; bench timing
+2.3%, SPRT undecided +1.9) is 0.998 on game positions: no speed-up.
LESSON (skeptic check of "the bench understates speed about 3x"): it did
for #38's dropping, but not for lazy-noisy or quiet-later. The bench is
not reliably low or high; instruction counts are not time either. Judge
speed only by wall clock on game positions. What is left in the profile
is mostly the network (accumulator, evaluate, refreshes: about 30%) and
the search itself; no further large, original move-loop saving is in
sight, so the speed lane has less in it than the advisor's estimate
assumed.

### Speed profile (20 October; callgrind on bench, AVX2 build)

Instructions: search logic 20%, accumulator updates 13% (inlined in
negamax), move picking (selection scan) about 10%, quiet history scoring
8%, evaluate 8%, move generation 4%, SEE 4%, king refreshes 3.5%,
make_move 3%. Top 3 speed-ups:
1. Lazy quiet scoring: score quiets only when reached (most nodes cut off
   on the hash move or a capture). In speed-picker.
2. Drop pruned quiets from the list: after late-move or futility pruning
   the picker still scanned every remaining quiet for each capture left.
   In speed-picker. Together with 1: 11% fewer instructions per node,
   +5.6% nodes per second.
3. SEE computed twice for captures in quiescence (once to order, once to
   prune). Checked 20 October: reusing the ordering answer where it
   decides the pruning one keeps the bench identical (780879) but saves
   nothing (instructions +0.1%; timing +1.3%, within noise): the history
   lookup needed to reuse it costs as much as the SEE calls it avoids.
   Dropped before any games.
Not proposed: deferring accumulator updates until a node is evaluated (a
known technique elsewhere; dropped under the originality rule, as on 13
October).

| # | Test | Lane | Where | Status (15 October) |
|---|---|---|---|---|
| 1 | Merge combo (tm-control + nnue-simd) | search | - | DONE 15 October: v2 check passed (same bench 780879, 20 games, no errors); merged |
| 2 | Part A (tm-plan) gate test vs the new main, repeating 40/10, cap 12000 games | search | GitHub | FINAL: +0.71 +/- 3.40 after 10800 games at the cap (#21 + #22; one GitHub job lost; undecided, 0 time losses). Gate change 1 = gate LOSS by the rule fixed in advance; not merged, main kept. Gate change 2 (nnue-simd) won, so the Fable gate does not trigger. #21 alone: +2.72 +/- 4.55 after 6000 games. Started 15 October (SPRT #21). Version tested, fixed before any result: tm-plan with the new main merged in, so it keeps tm-control's 12% as well (part A added to main, nothing removed). Local check: 20 games at 40/10, 0 time losses |
| 3 | Calibration (5-Calibrate.bat) | search | PC CPU | owner, 15 October: now, on the main with combo (replaces the 14 October decision to wait for part A); part A needs its own calibration later |
| 4 | Quantisation check (Scout 14 October, idea 2): float vs integer eval gap | network | PC CPU or GitHub | Coder 2 to design |
| 5 | T1 (Coder 2's request): SPRT #23, net-wide2048-20261015-0709 vs main at 8+0.08: FAILED -29.76 +/- 4.16 after 9000 games (LLR -20.2, 0 time losses). Prediction self-play +15 to +45 (70% pass): badly wrong. By the time-odds ruler, 33% fewer nodes per second costs about -20 to -25 on its own, so the bigger network's better evaluation is worth only about 0 to +5 here; its held-out loss was only 1.3% lower. Eval-scale check (advisor, 16 October; 1505 positions from calibration games): 2048/1024 scale 0.99, mean |eval| 247 vs 240 cp, correlation 0.97: same scale, so scale mismatch does not explain the result. Verdict test at 30+0.3 (owner's rule: -5 to +5 there means one SPSA re-tune first; clearly negative closes the path) SPRT #24 at 30+0.3: FAILED -19.71 +/- 6.61 after 3000 games (LLR -4.93, 0 time losses): clearly negative, path closed (owner's rule). The deficit shrank from -30 to -20 at the slower clock, as a speed cost does, but not near zero. Review rule (owner, 15 October): Coder 2's first trained network did not pass, so the network work folds back into the Session. Started 15 October 6 PM. Local check: bench 781681, plays; 33% fewer nodes per second than main here. Loss check (Net-Loss.bat): held-out loss 0.0817 vs 1024's 0.0828, training loss 2.5% lower. Wide network (2048 neurons, same recipe as the 1024 run: 3.65B Leela positions, 600 superbatches, LR 0.001 cosine, WDL 0.5) | network | PC GPU, then GitHub SPRT | planned. Coder 2's prediction (logged before training): self-play +15 to +45, CCRL +5 to +25, about 70% chance to pass; speed cost -28% (AVX2 build). Owner's rule, fixed before training: if 2048 lands between -5 and +5 at 30+0.3, run one SPSA re-tune before the verdict; clearly negative closes the path. Session's review: test it at 8+0.08 and at 30+0.3 (a slower, bigger network is penalised more at a fast clock than at CCRL's 40/15, as with the 1024 net: +80 and +82); about 16 epochs over 3.65B positions for twice the weights, so under-training is the main risk |

## PC queue

One heavy job at a time on the PC; Ayman decides the order. Coder 2
requests PC time in NETWORK.md; the Session adds it here.

| # | Job | Owner | Hours | Status (15 October) |
|---|---|---|---|---|
| 1 | Quantisation check, step 1 (tools/net/Quant-Check.bat; reviewed by the Session 15 October: deletes nothing, writes only sprt\coder2) | Coder 2 | under 1 | next; Coder 2's prediction: mean gap 4-8 cp, 95th percentile 10-16 cp |
| 2 | Calibration of the new main, combo merged (5-Calibrate.bat) | Session | about 2.5 | DONE 14 October: about 3482 (+16) |
| 3 | Wide network, 2048 neurons (tools/net/Wide-Network.bat; reviewed by the Session 15 October: deletes only its own unshuffled .raw parts in sprt\coder2\data, peak disk about 150 GB, touches nothing of the Session's) | Coder 2 | about 20 (GPU) + 2-3 (CPU) | after 2; owner said go. No calibration or SPRT on the PC meanwhile. Coder 2 compares the training loss at superbatch 300 with the 1024 run and reports; if 2048 is not clearly ahead, Ayman may stop it. Loss comparison script tools/net/Net-Loss.bat (reviewed: deletes nothing), run after training |

| 5 | Control run: main's recipe unchanged, new random start, same data files as 4 (tools/net/Control-Network.bat; same script as 4; advisor and owner, 18 October) | Session | about 5 (GPU) | next GPU job after 4, whatever 4 gives. Measures run-to-run noise; prediction: self-play vs main within +/- 10 (nobody knows; this run is the measurement) |
| 4 (result) | 16 king buckets: FAILED, SPRT #28/#29/#31, -2.10 +/- 2.85 after 18000 games (screen +18 +/- 19; held-out loss -0.27%). Not merged. Head-to-head vs the control follows the control's extended test (NETWORK.md 6) | Session | - | done 19 October |
| 4 | 16 king buckets (tools/net/Dry-Run-KB16.bat, then KB16-Network.bat; owner's decision 18 October; reviewed by the Session: deletes only its own dry-run folder, converts nothing, writes only sprt\coder2) | Session | 5-8 (GPU) | ready. Runner jobs wait while it trains (the runner starts nothing while a trinity* program runs). Prediction and gates: NETWORK.md section 6 (self-play -8 to +10, central +2; about 30% to pass) |

Gates for the new-data network (owner, 16 October, before the run):
1. Dry run of every stage first (tools/net/Dry-Run-New-Data.bat, about 2
   minutes).
2. After training: held-out loss (Net-Loss.bat) at least 0.3% better than
   main's network (0.0828 -> 0.0825 or lower; owner chose 0.3% over 1% on
   16 October, before the run); then 400 quick games on
   GitHub at 8+0.08, stop if the estimate is -10 or worse; only then the
   full SPRT.
3. Training length: the 1024 run trained in about 5 hours (checkpoint
   times); the 2048 run took 9.6 hours. Expect about 5-6 hours plus 1 hour
   of conversion.
Result (17 October): trained 5.0 h on 5.6B positions (branch
net-data1024-20261016-1554). Gate 2a PASSED: held-out loss 0.08232 vs
main's 0.08276 (-0.54%, inside the predicted 0.3-0.8%; the 2048 net was
-1.3% but 33% slower). Training loss on the first file's positions is
slightly worse (0.02589 vs 0.02564), as expected with fewer passes over
them. Gate 2b: 400 games at 8+0.08 on GitHub, SPRT #25.
Note on the gate (advisor, 17 October): the first proposal was 1% (about
0.0819); the owner set 0.3% (0.0825) on 16 October, before training
started (commit b4003df, which the run itself built from). So the network
passes the gate in force, not the 1% first proposed; at 1% it would have
stopped here. Rule from now on (owner, 17 October): held-out loss at least
1% better than main's for networks slower than main, at least 0.3% for
networks of the same speed.
Full SPRT cap for this network (owner, 17 October, before #25 finished):
at most 18000 games at 8+0.08 (two GitHub runs); undecided at the cap =
not merged.
Gate 2b PASSED: -2.61 +/- 19.05 after 400 games (SPRT #25, 0 time
losses). Full SPRT started 17 October 9:06 AM as #26.
RESULT: SPRT #26 FAILED, -10.85 +/- 4.08 after 9000 games (LLR -8.05,
0 time losses). Not merged. Eval scale checked (1504 positions from
calibration games): new/main 1.00, so not a scale problem. Prediction
(self-play +3 to +12) wrong in sign.
Lesson: held-out loss on Trinity's games did not predict playing strength
twice in a row (2048 net: -1.3% loss, -30 Elo; this one: -0.54% loss,
-11 Elo). The loss gate can stop bad runs early but cannot pass good ones;
only games decide. Every network attempt since the 1024 run (9 October)
has failed: refinements, averaging, 2048, more data.
4. The 117 GB in sprt\coder2\data (big-1..4.data) are the first Leela
   file's 3.65B positions, the same ones main's network trained on; this
   run reuses them, so they stay.

## Time losses before 1.1 (17 October 2026)

Calibration of 14 October: Trinity lost 9 games on time in 4000 (1.0.0: 3).
All 9 were long games (53-96 moves), 8 of them already lost or drawn on
the board, ending with overruns of 2 to 103 ms. Rebuilding the clocks from
the PGN: in every one Trinity had played for many moves with only a few
tenths of a second left. Cause: in sudden death the plan spends t/25 plus
3/4 of the increment, so at 10+0.1 the clock settles near 0.6 s; the 12%
from tm-control lowers that to about 0.35 s. At 14 games at once on the
PC, a delay of 0.1 s then forfeits. Move Overhead was only 10 ms.
Fix, branch tm-reserve: with the clock under 10 increments, spend half
the increment and skip the 12% (the clock settles near 1 s); Move
Overhead default 30 ms. Bench unchanged.
Local stress test (3+0.05, 8 games at once on 4 cores, 400 games): 0 time
losses for either side, so the failure did not reproduce here; Elo
-21.7 +/- 19.9 (noise or a real cost, to be settled by #27).
PC calibration of tm-reserve (17 October, 4000 games): 0 time losses
(14 October main: 9), so rule (b) is met. Rating about 3471 against
main's 3482 (-11; the difference of two calibrations has about +/- 10 of
noise, but all four opponents were lower by 6 to 16). #27 decided the
strength question: -0.85 +/- 3.71 after 9000 games (undecided, 0 time
losses). Both rules met (Elo estimate -2 or better; at most 5 time
losses): MERGED 17 October. The time-loss condition for 1.1 is met. The
calibration's -11 against main stays a watch item for the next
calibration.
The sensitive check (advisor), all 4000 games each, book moves skipped:
moves started with under 0.3 s on the clock fell from 2.03% (14 October)
to 0.32% (tm-reserve), six times fewer; under 1 s 8.74% -> 7.89%. Time
use otherwise unchanged (12% unused at game end; opponents 7-8%).
Decision rule, fixed before results: merge tm-reserve only if (a) SPRT #27
(9000 games at 8+0.08) gives an Elo estimate of -2 or better, and (b) a PC
calibration of the branch shows at most 5 Trinity time losses in 4000
games (changed from 3 on 17 October, before any result: with a true rate
of 3, "at most 3" fails a working fix about 35% of the time; "at most 5"
passes it about 92% of the time and still catches an unchanged rate of 9
about 88% of the time). Target before 1.1: no more than 1.0.0's rate.
More sensitive check (advisor): share of Trinity's moves started with
under 0.3 s on the clock, before (calibration of 14 October) and after
(18-Time-Use.bat now prints it). Baseline from the 949 lost games of 14
October: 4.07% of Trinity's moves started under 0.3 s (opponents 2.00%),
13.95% under 1 s (opponents 12.29%). The tool now skips book moves (they
were counted as moves that earn the increment, which overstated clocks;
earlier time-use figures carry that small bias).

## Parked ideas

**PARKED (not started; Ayman's decision, 17 October 2026): idea 6,
teacher-student.** Use the existing 2048 network (held-out loss 1.3% below
main's 1024, but 33% slower) to re-score the training positions, then
train a 1024 network on those scores: the better judgement without the
speed cost. Distillation is standard machine learning, not taken from any
engine. Replaces the self-play idea parked earlier the same day.
Why not self-play data: it was already tried. Networks #1 to #4 (30
September - 3 October) were trained on Trinity's own self-play games
(3-Generate-Training-Data.bat: 5000 nodes per move, about 120M
positions; README: "Trained first on Trinity's own games"). Switching to
Leela data gave +218 in self-play and +169 on the CCRL scale (network #5),
so self-generated data at this strength is far worse than Leela's.

Skeptic review before un-parking (Session, 18 October, owner said "go"):
(1) the training labels are already deep engine scores (the Leela data's
scores), far stronger than any static network; re-scoring with the 2048
network's static eval would replace good labels with weaker ones, so the
only gain left is distillation's "easier target", typically a fraction
of the teacher's 1.3% held-out edge; (2) held-out loss did not predict
Elo twice (2048: -1.3% and -30; more data: -0.54% and -11), so even a
student that inherits part of the edge is a coin flip; (3) disk: a
re-scored copy of the 3.65B positions needs another 117 GB (125 GB free).
Estimate if built: self-play -10 to +8, about 15-20% to pass. Put back to
the owner with the alternative below before any work.

**PARKED with a trigger (19 October): best of several random starts**
(Scout, scout/2026-10-19.md). Train the best recipe with 3-4 weight seeds,
keep the best on about 2000 games each, confirm with a fresh SPRT.
Taken up if the control's extended test (about 3000 games) shows -8 or
worse against main; dropped if within +/- 4 (NETWORK.md section 6). Taken up 19 October (control -9.7); with
three runs (0, -9.7, -0.2) the spread is about +/- 5.5, so its value is
small: it ends with KB16-Seed2 unless Seed2 passes against main.

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
| 2026-10-14 | main with combo merged: tm-control + nnue-simd (bench 780879) | 49.05% -> 3467 | 47.60% -> 3471 | 48.95% -> 3515 | 40.45% -> 3476 | - | about 3482 (+16 over 3466; predicted +5 to +12, lowered to +2 to +8 after #19). All four opponents up 13-22. 9 games lost on time by Trinity in 4000 (1.0.0: 3), worth watching |

### First runner calibrations (queued 17 October 2026, predictions written first)

Run by the PC job runner, overnight or after 30 idle minutes (CLAUDE.md,
runner rules 11-12). Same PC, opponents, book, Syzygy and 10+0.1 as before.

| Job | What | Prediction | Pass rule | Result |
|---|---|---|---|---|
| 2026-10-18-11 | release 1.0.0, 200 games (plumbing check) | about 3466 | within +/- 35 (200 games: 1 sigma about 16) | 3472 (+/- 27): PASS. 9 minutes, all four opponents played |
| 2026-10-18-12 | release 1.0.0, 4000 games (known answer) | 3466 | within +/- 20 (two 4000-game runs differ by about 10 at 1 sigma, so the advisor's +/- 10 would fail a correct runner about 1 time in 3) | 3478 (+/- 6): PASS, +12 over 3466. Per opponent vs 12 Oct: akimbo 3454 (3453), Patricia 3469 (3458), Bread 3510 (3493), Prune 3481 (3461): all four up, so part of the +12 may be the conditions (overnight, PC otherwise idle) rather than noise. Compare runner results with runner results |
| 2026-10-18-13 | main (= tm-reserve merged), 4000 games | about 3476 (3471 and 3482 measured this week) | for information; a result under 3456 would repeat the watch item (-11 on 17 Oct) | 3492 (+/- 6). Same-night gap over 1.0.0 (3478): **+14 +/- 8.5** (rule 13). The absolute prediction (3476) was 16 low, the same conditions shift as 1.0.0's; the gap matches the daytime chain (+16 combo, then -11 tm-reserve, i.e. +5 to +16). Watch item closed: tm-reserve did not lose strength |

17 October, 5:46-5:49 pm: all three FAILED within a minute, before any game: the approved scripts' housekeeping tried to tidy `data\prepared` through the read-only link and could not even list it. No measurement; queued again as 11-13 after the approval of dabb545 (18 October, about 12:50 am). Also, job 07 started while Ayman was at the PC: the idle check took "No User exists" (what quser tells the background account) as idle. Both fixed (scripts: no housekeeping of linked folders under the runner; runner: unknown idle time means wait for 11 pm). The read-only wall did its job: the scripts could not touch Ayman's data.

### Runner analysis jobs (queued 18 October, predictions written first)

| Job | What | Prediction | Result |
|---|---|---|---|
| 2026-10-18-14 | time use, the three runner calibrations of 18 Oct | main: under 0.3 s about 0.3-0.6% of moves (tm-reserve by day: 0.32%), at most 5 time losses in 4000; 1.0.0: about 2% (by day 2.03%) | main (03:29 run): 0.06% of moves under 0.3 s (better than predicted 0.3-0.6%), 6.29% under 1 s; 0 time losses in 4000. 1.0.0 (01:07 run): 0.51%, not the predicted ~2% (by day 2.03%): the idle overnight PC clearly helps 1.0.0's clock too; 0 time losses in 4000. Opponents 7.2-7.6% under 0.3 s. Prediction for main: PASS (better); for 1.0.0: MISS (4x fewer) |
| 2026-10-18-15 | loss forensics, main's runner calibration (800 losses, 4x the usual sample) | close to 1.0.0's mix of 13 Oct: gradual 60-75%, search 15-25%, evaluation 4-10%, time 1% or less (fewer than 1.0.0 after tm-reserve). Stockfish referee only if copied into the runner's clone; otherwise Trinity judges itself and the mix is less reliable | 800 of 876 losses, Stockfish referee, 13 minutes (not the expected ~2 h). Gradual 75% (top of 60-75%), search 16% (low end of 15-25%), evaluation 6.5%, time 0% (predicted 1% or less), lost from the book 2%. Inside the prediction on every line. Search mistakes: median time ratio 0.77 (128 cases), again the base rate, so not under-thought. Gradual losses' worst moves at 10x time: search 568 / evaluation 487 |

### Slower time control on the runner (owner's decision, 18 October)

40 moves in 60 s, repeating (about 4x the time of 10+0.1; CCRL plays 40/15
on its own hardware). 480 games per version per night (60 rounds x 4
opponents, about 2 h each), main and 1.0.0 the same night; nights are
pooled until the gap's error is under about +/- 10.

| Job | What | Prediction | Result |
|---|---|---|---|
| 2026-10-18-16/17 | main and 1.0.0 at 40/60, 480 games each | gap over 1.0.0 +5 to +15 (+14 at 10+0.1; time-management gains may shrink with more time); one night is about +/- 21, so no verdict before 3 nights; 0-2 time losses per 480 games | Night 1 (18-19 Oct): main 3511, 1.0.0 3490 (each +/- 17): gap +21 +/- 24, above the +5 to +15 range but well inside one night's noise; no verdict (1 of 3 nights). Per opponent main minus 1.0.0: akimbo +41, Patricia -3, Bread +32, Prune +17, so akimbo drives most of it. 0 time losses for either (480 games each). Night 2 NOT queued 19 Oct: scripts/calibrate.ps1 changed since approved commit dabb545 (new -Affinity option, 1a03ff6), so the check-in may not queue it until a newer commit is approved Night 2 (19-20 Oct, jobs 2026-10-19-01/02): main 3514, 1.0.0 3506 (+/- 15-16): gap +8 +/- 22, inside +5 to +15. Main's first try was DISTURBED at 23:42 (keyboard used) and re-ran idle from 00:28, so both counted runs were idle throughout. Per opponent main minus 1.0.0: akimbo -15, Patricia +14, Bread +9, Prune +13. 0 time losses. Pooled over 2 nights (weighted): about +14 +/- 16 (2 of 3 nights; still no verdict). Night 3 queued 20 Oct (2026-10-20-01/02) Night 3 (20 Oct by day, idle PC, jobs 2026-10-20-01/02, still the old main before speed-picker): main 3503, 1.0.0 3517 (+/- 15-16): gap -14 +/- 22. Per opponent main minus 1.0.0: akimbo -30, Patricia -32, Bread +15, Prune 0. 0 time losses. RESULT over 3 nights (weighted): +4 +/- 13 (nights +21, +8, -14; their spread fits noise). Prediction +5 to +15: at its bottom edge, not distinguishable from 0. Reading: the +14 of the old main at 10+0.1 mostly does not carry over to 40/60, as the prediction warned for time-management gains. Series closed for the old main (main has changed); the new main's 40/60 pair is jobs 2026-10-20-09/10 |

Method from 18 October (CLAUDE.md runner rule 13): report each version as
its gap over 1.0.0 calibrated the same night on the runner. Lesson: the
earlier daytime gains (+17 for 1.0.0, +16 for combo) were measured under
changing conditions and carry more noise than their +/- 7 suggests.

## PC use: usefulness, not hours (owner and advisor, 19 October)

There is no hours target (the 80-90% use mentioned in chat is withdrawn;
it was never written here). Flag it only when useful work is waiting
and the PC is not doing it; an idle PC with nothing worth testing is
fine.

## Dated checks

- **26 October 2026: PC link decision** (owner and advisor, 19 October).
  Count the week's PC visits Ayman had to make for Trinity (clicks,
  pastes at the PC). Two or fewer: keep the current setup, no link. Three
  or more: set up a Claude session that can work in C:\Trinity, with the
  rule that it reads files and starts only approved scripts, never edits
  the runner, never trains without Ayman's "go". Correction to the
  advice: this cloud session cannot be linked to the PC; the option is a
  session running on the PC itself (desktop app or `claude
  remote-control` in C:\Trinity), which runs as Ayman, so the rule is a
  promise, not a Windows wall. Brought forward if Ayman will be away for
  days while the work needs his clicks. Before it: the runner's clean
  week ends about 24 October, after which training can become a runner
  job (fewer visits).
  Visits so far (week of 19 October): 19 Oct, 4 pm (git pull, reinstall,
  Control2); 19 Oct, about 11:45 pm (Drift-Check, run twice); 20 Oct
  (KB16-Seed2 start, planned). Training is the main reason for visits;
  after the clean week (about 24 October) it can become a runner job.

## Runner check-ins

One line per morning check-in (CLAUDE.md rule 5 exception): date, what
was read in trinity-results (last runner log line covered), what was queued.

- 2026-10-18 (set up by the Session): read up to "03:29 STARTED 2026-10-18-13". Nothing queued.
- 2026-10-18 6:20 am: the scheduled check-in ran without the repository (it could not attach it) and wrote nothing; the Session logged job 13 by hand: read up to "05:50 DONE 2026-10-18-13". Nothing queued. The check-in moves to a persistent session with the repository attached (as Scout).
- 2026-10-19 6:27 am: read up to "03:17 DONE 2026-10-18-17" (jobs 14-17, all done, 0 time losses). Nothing queued: scripts/calibrate.ps1 differs from the approved commit dabb545 (1a03ff6, -Affinity), so 40/60 night 2 waits for a new approval.
- 2026-10-19 7:10 am (Session): 40/60 night 2 queued (2026-10-19-01/02). The approved calibrate.ps1 (dabb545) runs these arguments as they are; the newer -Affinity option is only needed on the laptop, so no new approval is needed for the PC.
- 2026-10-20 6:21 am: read up to "04:31 DONE 2026-10-19-02" (40/60 night 2; one DISTURBED restart, then done). Queued 40/60 night 3: 2026-10-20-01 main, 2026-10-20-02 release-1.0.0 (approved dabb545 accepts these arguments; CLAUDE.md rule 5).
- 2026-10-20 4:30 pm (Session): 40/60 night 3 already ran by day (8:10 am - 12:18 pm, idle PC); results for the next check-in. Queue was empty, so tuning night 1 queued: 2026-10-20-03 chain A, 2026-10-20-04 chain B (approved 58841ba). They start after Seed2's training ends.
- 2026-10-20 4:50 pm (Session): speed-picker passed (+21.5) and is merged. Calibration of the new main comes before tuning (pace rules): tuning jobs 03/04 replaced by 05 main and 06 release-1.0.0 (10+0.1, 500 rounds each, the same night, rule 13), then 07/08 tuning chains A and B (they now tune the new main).
- 2026-10-20 5:00 pm (Session, advisor's check): #40 and #41 are built on and tested against the new main (both contain the merge 22cdaf1; base at dispatch c44a99a). Added a 40/60 pair for the new main (09 main, 10 release-1.0.0), after tuning chain A: order 05, 06, 07, 09, 10, 08. Chain A takes the early-morning slot because a stopped tuning job loses at most one step, while a stopped 40/60 game is lost.
- 2026-10-21 6:40 am (Session, Scout's points): (1) the 40/60 nights are treated as the CCRL-relevant number (old main: +14 at 10+0.1, +4 +/- 13 at 40/60). Prediction for the new main's 40/60 pair (jobs 09/10), written before it runs: gap over 1.0.0 +4 to +20, central +12 (Scout: +8 to +12); ordering gains should carry to slower clocks better than the time-management gains did, the speed part less. (2) The ablation audit gets a no-change baseline (the same engine with a different hash size, or 4% more or fewer nodes); a rule counts only if its net effect clearly beats that. RESULT (check-in 22 October, jobs 2026-10-20-09/10, 1:06-5:06 am, idle): main 3545, 1.0.0 3499 (+/- 15-16): gap +46 +/- 22. Above +4 to +20, but by then main also contained no-razor (merged 21 October 12:35 pm, +39 over the previous main at 10+0.1), which the prediction did not cover; +12 plus a part of no-razor's +39 fits. Per opponent main minus 1.0.0: akimbo +38, Patricia +50, Bread +20, Prune +83. 0 time losses. One night only.
- 2026-10-21 7:00 am (Session): Scout's split of #38 started on GitHub: split38-lazy (lazy quiet scoring only, no dropping) vs base-before-38 (df7be94), 4000 games. Scout's description had the halves reversed: dropping pruned quiets is pure speed (the old code already skipped them); lazy scoring is the ordering change. Prediction: +14 to +22 (if the ordering is the cause; dropping is worth about +1-2 by the ruler).
- 2026-10-21 (advisor): when GitHub is full, new candidates go ahead of research tests such as the #38 split (#46 had already started, so it runs on).
- 2026-10-21 9:30 am (Session): #38 split result (SPRT #46, 4 of 5 jobs; job 3 lost to a GitHub machine shutdown, exit 143, not a Trinity crash): lazy scoring alone vs the main before #38 +4.45 +/- 6.46 after 3200 games, undecided. Below the prediction (+14 to +22). Either the ordering half is smaller than thought (the whole of #38 was +21.5 +/- 3.8), the two halves are not additive, or one of the two tests drew lucky or unlucky (the gap of 17 +/- 7.5 is about 2 sd). To be continued with more games when GitHub has room (candidates first); a dropping-only test would complete the split.
- 2026-10-21 12:20 pm (Session): no-razor passed (+58). Tuning paused: chains A/B tuned RAZOR_MUL, which no longer exists once razoring is removed (jobs 07/08 removed from the queue; chain A had about 3 hours done). The tuning script now tunes LMR_HIST_DIV instead (fifth most moved; RAZOR_BASE, fourth, is razoring too) and has fresh chains C and D for the new engine; needs Ayman's approval. Queue now: no-razor calibration (2026-10-21-01) and 1.0.0 (02) the same night, then the 40/60 pair of main (09/10, which will include no-razor once merged). Running on GitHub against the current main, to finish before the merge: tm-stable-floor 40/10 (+1000 games), speed-pick2 (+3000), Seed2 (+6000).
- 2026-10-22 2:30 pm (Session): 40/60 night 2 ran by day (idle PC, 7:46 am - 11:58 am): main (no-razor) 3546, 1.0.0 3504: gap +42 +/- 21; with night 1 (+46 +/- 22, 22 October 1-5 am): pooled +44 +/- 15. Same-night 10+0.1 (21-22 October night): no-razor 3542, 1.0.0 3473: +69 +/- 9. Merged speed-pick2 and no-qsfut (+60). Queued tonight: main (with no-qsfut) and 1.0.0 at 10+0.1, then main at 40/60 (2026-10-22-03/04/05; about 7 hours, fits before 6:55 if the PC is idle by 10 pm). Tuning stays paused; QS_FUTILITY (in its list) is gone too, so the list changes again before tuning resumes (one approval).
- 2026-10-21 6:22 am: read up to "05:55 DISTURBED 2026-10-20-07" (40/60 night 3 logged: old main +4 +/- 13 over 3 nights; 05/06 already logged by the Session; tuning chain A stopped at 05:55 by keyboard use, reruns by itself). Nothing queued: 07 (rerun), 09, 10 and 08 are still waiting.
- 2026-10-22 6:23 am: read up to "05:06 DONE 2026-10-20-10" (tuning chain A finished 12:45 pm; no-razor and 1.0.0 same-night pair at 10+0.1: gap +69 +/- 9; main 40/60: +46 +/- 22; three DISTURBED restarts of 1.0.0, all re-run idle; 0 time losses). Two nights now show a gap of +50 or more at 10+0.1 (+65 reference from 21 October, +69 same night), so Ayman is told about the 1.1 question. Queued a second 40/60 pair for main (2026-10-22-01 main, 2026-10-22-02 release-1.0.0) to firm up the slower-clock gap for that decision.
- 2026-10-23 6:26 am: read up to "00:42 DONE 2026-10-22-05" (main and 1.0.0 the same night at 10+0.1: gap +102 +/- 9; main at 40/60: +66 +/- 21, reference from 22 October; 0 time losses, no disturbances). Nothing queued: tonight's release-1.1 jobs need the branch that Ayman's "freeze now" creates, so the Session queues them.
- 2026-10-24 6:22 am (run by Ayman's "run"): read up to "21:53 DONE 2026-10-23-06" (all six release-1.1 jobs done; 40/60 same day +63 +/- 21; threads +79.5 +/- 19.3, no errors; 0 time losses). The Session had already logged 01-03. Nothing queued: no further runner work is planned (tuning stays paused, and the 1.1 measurements are complete).
- 2026-10-25 6:24 am: read up to "05:03 STARTED 2026-10-24-06" (tuning chain D still running, due about 6:34). Main vs 1.0.0 same day: +111 +/- 9 at 10+0.1, +63 +/- 21 at 40/60; tuning chain C done. The runner still uses approved commit 189246b (QS_FUTILITY, not FUT_MUL). Queued tonight's tuning (planned: chains C and D each night, tuning last): 2026-10-25-01 chain C, 2026-10-25-02 chain D (270 minutes each; the runner cuts them to end by 6:55).
- 2026-10-26 6:25 am: read up to "03:12 DONE 2026-10-25-02" (tuning chains C and D, one DISTURBED restart; approval 272a0f3 in effect). APPROVAL PENDING: ec90c7e (scripts/tune-opponents.ps1, per-setting step schedule) is newer than the approved 272a0f3. Queued tonight's tuning (night 3, planned): 2026-10-26-01 chain C, 2026-10-26-02 chain D.
- 2026-10-27 5:36 am (run by Ayman's "run"): read up to "23:38 DONE 2026-10-26-02" (tuning night 3; one DISTURBED restart; 0 time losses). Tuning stop rule fired: only 2 of 4 settings move the same way in C and D, so the chains stop and nothing is queued (no candidate is waiting for a calibration). The pending approval (ec90c7e) is moot. Search lane: no SPRT running; the last candidates (#69, #70) started about 7 am on 26 October and both failed, so the 24-hour pace alarm trips about 7 am today.
- 2026-10-28 6:37 am: read up to "23:38 DONE 2026-10-26-02" (no runner job since; queue empty). Nothing queued: no candidate to calibrate and no other planned job runs on the approved scripts (Friday brings the weekly calibration). APPROVAL PENDING: 7ad796a (scripts/sprt.ps1, tools/net/kb16.ps1; also needs a runner reinstall) is newer than the approved 272a0f3. Search lane idle: the last candidate tests (#72-#74) started 27 October 12:49 UTC, over 24 hours ago, and none is running.

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
Under game-like load (advisor's check, 14 October: 3 bench copies at once,
30 runs per machine): -2.9% (AMD EPYC 9V45), +8.6% (Xeon Platinum 8573C),
+9.9% (Xeon 6973P-C). Machines differ a lot (the AMD one may already get
wide vector code from the compiler for the plain loop), but no machine
shows a gain near the +20% that +13 Elo would need. Load does not explain
it either.
The combined test #19 is an independent repeat: if nnue-simd's +13 is real,
combo should score about +17 to +20 (+13 plus most of tm-control's +6.8).
Combined test started 14 October: branch combo = main + tm-control +
nnue-simd, SPRT #19 at 8+0.08 and #20 at 40/20 (1200 games, the time-loss
check), both against main.
Prediction for combo, logged before #19 and #20 finished (Scout and
advisor, owner's paste of 14 October): self-play +10 to +18; calibration
(CCRL scale) +5 to +12, i.e. 3466 -> about 3471-3478.
Result: #19 PASSED +5.64 +/- 3.74 (9000 games, LLR 3.10, 0 time losses);
#20 at 40/20: +4.05 +/- 9.29 (1200 games, 0 time losses, 0 crashes).
Below the prediction (+10 to +18) and far below the +17 to +20 that
nnue-simd's +13 would imply. Reading: the parts do not add up as measured
alone; most likely #17's +13 was largely luck (a test that stops when it
passes catches upswings), and the speed-up is worth about what the ruler
says (+3) or less. The gate rule was fixed in advance, so #17 still counts
as the gate win, but future estimates should treat a single early pass as
an overstatement. Self-play gain of combo vs main: about +6.
Revised calibration prediction for combo (owner, 15 October, after #19 and
before any calibration): +2 to +8 on the CCRL scale (was +5 to +12). The
combo miss is logged: predicted self-play +10 to +18, got +5.6. Gate count
unchanged: #17 stays the gate win, as decided in advance.
Calibration of combo (14 October, 4000 games at 10+0.1): about 3482, +16
over 1.0.0's 3466, above even the first prediction (+5 to +12) and far
above the lowered one (+2 to +8); self-play said only +5.6. All four
opponents agree (+13 to +22). Reading: time-management gains may carry
over to rated engines more than 1:1, unlike network refinements (the
opponents spend their time better, so Trinity's leak cost more against
them). Statistical uncertainty of a difference of two calibrations is
about +/- 10, so the true gain is likely +6 to +26. Lowering the
prediction after #19 was the wrong correction for this kind of change.
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

## Ablation audit (21 October 2026; Scout's idea of 13 October)

2000 positions from Trinity's lost calibration games (Trinity to move,
move 9 on, not in check, 12+ pieces, |eval| <= 300). Each version
searches 250,000 nodes (equal nodes, not equal depth: advisor); its move
is compared with main's own search at 16x the nodes. Main agrees on 1228.
One rule switched off at a time (net = positions fixed - broken; sd about
21 from the counts alone):
- razoring off: net +42 (fixed 250, broken 208), the only clear gain;
- IIR off +11; history term in LMR off -2; singular extension off -3;
  futility off -6; QS futility off -7 (all within noise);
- null move off -17, SEE quiet -18, LMR off -20, SEE capture -24, QS SEE
  -27, unsure term -31, LMP off -36, RFP off -61 (these rules help).
Caveats: the reference is Trinity's own deeper search, so it shares
Trinity's blind spots; 14 rules were tested, so one 2-sd result can be
chance.
No-change baseline (Scout): the same engine with only the hash size
changed: 8 MB net +20 (fixed 190, broken 170), 64 MB net +22 (184 /
162); with 4% fewer or more nodes almost nothing moves (+0 from 12 flips,
+2 from 26), because the move comes from the last finished iteration.
So a change that touches nothing still flips about 350 of 2000 moves and
nets about +20, and most rules sit below that (mean of the 14 about
-13). Razoring off (+42) is only about +21 +/- 28 above the no-change
level: weak evidence, not a finding. RFP (-61) and LMP (-36) are the
rules most clearly worth keeping. Lesson: at equal nodes the move choice
is chaotic; an audit like this ranks rules only coarsely, and needs at
least 4x the positions to separate effects of +20.

## Eval-bias measurement (23 October 2026; Scout's idea of the same day)

3939 positions from Trinity's lost calibration games (every 3rd ply from
move 7, not in check, 8+ pieces). Static network eval (no correction)
against a depth-10 search, side to move's view. "Behind": how far the
search lands ABOVE the static eval; "ahead": how far it lands BELOW.

| static eval | behind: mean / p90 / p95 | ahead: mean / p90 / p95 |
|---|---|---|
| 50-150 cp   | +23 / 174 / 254   | -5 / 120 / 156   |
| 150-300 cp  | +38 / 254 / 331   | -31 / 156 / 230  |
| 300-600 cp  | +85 / 462 / 556   | -43 / 206 / 315  |
| 600+ cp     | +341 / 1175 / 1424 | -102 / 152 / 195 |

(Near equal, |static| < 50: the search lands 18 above the static eval on
average: having the move is worth something.) Lopsided, as Scout
suspected: when behind, the search finds far more than the static eval
saw (p95 1.6x to 7x the "ahead" error); when ahead, the search usually
finds even MORE than the static eval (negative means), so "ahead" cutoffs
are safe, matching rfp-wide's -10. Caveat: some "behind" positions are
mid-exchange (a recapture pending), which the static eval cannot see; a
cleaner version compares the quiescence score with the search. Next:
measure the error at the depths where futility actually prunes (1-8) and
set its margin from the p95 there (a candidate for 1.2).

## Forecast scores (27 October review; advisor's idea of 23 October)

Every prediction with a numeric range logged before its result (Changes
table, calibration forecasts, Scout reports). Point results; a miss is
the distance outside the range (0 if inside).

| Forecaster | Inside the range | Above / below | Mean miss | Median miss |
|---|---|---|---|---|
| Session (45, from 3 October; includes Coder 2's 2048 network) | 22 (49%) | 10 / 13 | 8.4 | 0 |
| Scout (5 tested: bucket spread, weight averaging, runner tuning, "+10 to +25 on top" for no-qsfut, lmr-behind) | 1 (20%) | 1 / 3 | 3.9 | 3.0 |
| Advisor (4: rescore-quiets, #38 rated, no-razor rated, 22 October gap) | 2 (50%) | 1 / 1 | 2.6 | 2.0 |

Reading:
- The Session's ranges are too narrow: a "likely" range should catch
  about 70-80%, it caught 49%. Small search ideas mostly land inside or a
  little below; the big misses are all upward surprises (no-razor +58,
  no-qsfut +60, the 1024 network +80, speed-picker +21) or network
  failures (2048 -30, drill -92). Lesson: widen ranges about 1.5x, and
  give untested inherited rules and first-of-a-kind changes a fat upper
  tail instead of "about zero".
- Scout's ideas were predicted positive and came out negative 3 times in
  5; its strength is choosing what to test (the removal campaign), not
  the size of the effect.
- The advisor and Scout have too few scored forecasts (4 and 5) to
  separate skill from luck; re-score at the next review.
- Not scored (no numeric range or no clean result): the tuning's
  "0 to +6" (stopped, no change), research predictions without a range.
- Pass chances (advisor, 27 October; the queue ordering depends on
  them), 21 tests with a logged chance: expected 8.3 passes, actual 5.
  About 30%: 10 tests, 2.8 expected, 1 passed; 32-50%: 7 tests, 2.8
  expected, 2 passed; 50% and up: 4 tests, 2.7 expected, 2 passed. Brier
  score 0.171, barely better than always guessing the overall pass rate
  (0.181). So the pass chances run about 1.6x too high and barely rank
  the tests: until re-scored, scale them by about 0.6, and do not lean
  hard on expected-value ordering. Odd but consistent: the gains that did
  come were bigger than predicted, while passes were rarer.
- Next review: re-score range coverage after the 1.5x widening (target
  70-80% inside) and the pass chances (target: expected passes close to
  actual).

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
- Release 1.1 (cloud, 23 October; `trinity similarity 1000 100`, Linux,
  Stockfish 18 downloaded as a binary only; the other engines' release
  pages are not reachable from the cloud):

  |               | 1.1   | 1.1 again | 1.0.0 | Stockfish 18 |
  |---|---|---|---|---|
  | 1.1           | -     | 85.0% | 84.6% | 77.4% |
  | 1.1 again     | 85.0% | -     | 83.5% | 78.3% |
  | 1.0.0         | 84.6% | 83.5% | -     | 78.4% |
  | Stockfish 18  | 77.4% | 78.3% | 78.4% | -     |

  1.1 is no closer to Stockfish than 1.0.0 was (77.4-78.3% vs 78.4%),
  and 1.1 against 1.0.0 (84.6%) is about as close as 1.1 against itself
  (85.0%): same network, search changed only by removals and margins.
  The full six-engine PC run (13-Similarity-Test.bat) waits until a
  version is offered to CCRL (Ayman's decision, 23 October: not 1.1).
- Robustness suite (release/README.md), 23 October, cloud, 1.1 and 1.0.0:
  UCI checks 18 of 18 passed for both. Games (1.1 vs 1.0.0, 196 games):
  Hash 1 MB at 40/4 (80 games), Hash 1024 MB at 40/4 (20), 2 threads each
  at 40/8 (16): no crash, no illegal move, no disconnect, no time loss.
  Sudden death 5 s with no increment (80 games): 1.0.0 lost 6 games on
  time (overruns of 1-3 ms), 1.1 none. Cause known (17 October): 1.0.0's
  Move Overhead default is 10 ms and its clock runs very low in sudden
  death; 1.1 has tm-reserve (30 ms, keeps about 1 s). Not a CCRL 40/15
  risk: a repeating control refills the clock every 40 moves, and the
  movestogo checks passed (1.0.0 used 0.4 s of 2 s on the last move
  before the control). Verdict: 1.1 passes; 1.0.0 passes for repeating
  controls, with a known weakness in no-increment sudden death.
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
