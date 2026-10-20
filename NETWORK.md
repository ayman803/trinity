# Network notes

**16 October 2026: Coder 2 folded back into the Session** (owner's review
rule; the 2048 network failed SPRT). The Session maintains this file and
tools/net/ from now on. Everything below is kept as history.

Coder 2 owns this file from 15 October 2026 (see CLAUDE.md, "Coder 2").
It lives on `main` (owner's rule change of 15 October; the branch
`net-notes` is no longer used). The first section is the Session's
handover; everything after it is Coder 2's. PC time and tests are asked
for at the end of this file; the Session queues and runs them (ESTIMATES.md,
"Test queue", lane "network").

## Coder 2's rules (owner, 15 October 2026)

1. Coder 2 may push to `main`, but only NETWORK.md and its scripts in
   `tools/net/` (so the PC can `git pull` them and the others can read the
   notes). Engine code and `.nnue` files stay on `net-...` branches; only
   the Session merges them.
2. PC work goes through double-click scripts in `tools/net/`, like the
   Session's. Coder 2 asks for PC time here; the Session schedules it, and
   Ayman gets one instruction at a time.
3. Coder 2 updates its branch from `main` before every test request.
4. Fable for design, Opus for routine steps; the Fable limit (80% pause) is
   shared with the Session.
5. Coder 2's PC scripts may only write inside their own output folder
   (`C:\Trinity\sprt\coder2\`); any script that deletes or moves files
   is reviewed by the Session before Ayman runs it. (Added later on 15
   October. The PC queue rule that goes with it: no calibration or SPRT
   on the PC while training runs.)
6. Unchanged: never edit ESTIMATES.md; the originality rule (CLAUDE.md)
   applies; write a prediction before every test.

## Handover from the Session (15 October 2026)

### The network in main
- Architecture: (768 x 8 king buckets, mirrored by king file) -> 1024
  hidden, x2 (both sides) -> 8 output buckets (by piece count). SCReLU.
  Quantised QA = 255, QB = 64, SCALE = 400 (src/nnue.rs). Mirroring by
  king file makes the eval mirror-symmetric by construction.
- File: nets/default.nnue (padded to a multiple of 64 bytes, 12617792).
- Trained 9 October: from scratch, 600 superbatches (batch 16384, 6104
  batches per superbatch), AdamW, LR 0.001 with cosine decay, WDL 0.5
  constant, on the whole Leela file converted: 3.65 billion positions.
  Result: +80 self-play, +50 on the CCRL scale (3399 -> 3449): the biggest
  single gain the project has had. Search is about 28% slower than the 512
  network.
- Since then only search changes; release 1.0.0 (bench 780879) calibrates
  at about 3466 (10+0.1, 1 CPU).

### What the history says (ESTIMATES.md, "Changes" and "Lessons")
- Self-play overstates network gains against other engines by 1.1x to
  10x. Capacity (king buckets, 1024 neurons) carried over almost fully;
  more data or longer refinement on a converged network did not (net #6:
  +29 self-play, +3 CCRL; fresh Leela data: +6 / about 0; king buckets
  refined longer: -13.8 and -3.8; weight averaging of late checkpoints:
  +0.4 and -1.3; drill on Trinity-Leela disagreements: -92).
- One long run from scratch beat every patch. A restart at a high learning
  rate on a tuned network undid it.

### Training tools
- Trainer: trainer/src/main.rs (bullet). Modes:
  `scratch <superbatches> <name> <start lr> <wdl> <data files...>`,
  `finetune <checkpoint dir> <superbatches> <name> <data> [start lr]`,
  `convert`, `convert-chunks <binpack> <prefix> <chunks> <per chunk>`,
  `average <out> <checkpoint dirs...>`, `kbcheck`.
- PC launchers (each calls a script in scripts/):
  - 11-Big-Network.bat -> big.ps1: the run that made main's network
    (convert up to 4 x 1 billion Leela positions, shuffle, train from
    scratch, publish a net-big-... branch).
  - 4-Train-Network.bat -> train.ps1; 8-Fresh-Leela.bat and
    10-King-Buckets.bat -> fresh.ps1; 7-Drill-Network.bat -> drill.ps1;
    17-Weight-Average.bat -> average.ps1; 3-Generate-Training-Data.bat ->
    datagen.ps1 (Trinity self-play data, about 3-5 million positions an
    hour on the PC).
- Checkpoints: C:\Trinity\checkpoints\<name>-<superbatch>\, saved every 50
  superbatches. quantised.bin is the engine file; the float weights are in
  optimiser_state\weights.bin (what the quantisation check needs). Main's
  network is the 600-superbatch checkpoint of the big 1024 run (the
  "big1024" folders). Housekeeping in training scripts keeps main's
  checkpoint and the 4 newest and deletes older ones
  (Remove-OldCheckpoints in scripts/common.ps1): copy any checkpoint you
  need elsewhere before a training script runs.
- Disk: 32 GB per billion converted positions, plus about 52 GB for
  shuffling and spare. Budget about 200 GB (C: had 265 GB free on 15
  October).

### What was deleted to free space (13 October)
- data\prepared\big-*.data (the 4 converted, shuffled parts of the Leela
  file used for main's network, about 117 GB), shuffled.data, and the old
  self-play data gen_*. To train on the same positions again, the Leela
  file must be converted again (about 20 minutes per billion positions).
- Kept: the Leela binpack in data\leela (13.4 GB, the only source file;
  about 3.65 billion positions) and the checkpoints.

### Findings from games that bear on the network
- Loss forensics of 1.0.0's calibration (test-results/forensics-*.txt;
  Stockfish only as a referee): 68% of losses are gradual (no single move
  loses a pawn), 22% one search mistake, 6% one evaluation mistake. Of
  Trinity's worst moves in the gradual losses, about 60% are fixed by 10x
  more time and about 40% are not (138 / 107): the 40% point at the
  evaluation. Caveat: no noise control yet.
- Time use (test-results/timeuse-*.txt): Trinity spent only about 0.5-0.67
  of its planned time; the search lane is fixing that (tm-control merged
  with nnue-simd, part A in test). This matters for reading network tests:
  a slower network costs less when time is better spent.
- Speed: the output layer is AVX2 code since 14 October (nnue-simd).
  Network size and speed trade off: the 1024 network is about 28% slower
  than the 512 one and still gained +50 on the CCRL scale.

### Open items for Coder 2
1. Quantisation check (Scout, scout/2026-10-14-2.md, idea 2): compare the
   float network's eval with the quantised one on positions from
   Trinity's games; act only if the gap is meaningful.
2. Design of the next training run: data (the one Leela file, a second
   Leela file, Trinity's own games), size, length, learning rate. Disk
   and PC time are the limits; Ayman decides when the PC trains.

## Coder 2's log

### 1. Quantisation check, step 1 (Scout, 14 October, idea 2)

Question: how far does the integer network in the engine (nets/default.nnue,
rounded to whole units of 1/255 and 1/64) drift from the float network it
was rounded from (checkpoints\...-big1024-600\optimiser_state\weights.bin)?

How (15 October, done, waiting for the PC):
- Engine command `trinity quantcheck <weights.bin> <max positions> <pgn
  files...>` on branch `net-quantcheck` (src/quantcheck.rs; the engine
  itself is unchanged, bench 780879). It first checks number by number
  that rounding the float weights the trainer's way gives exactly the
  integer file (so the two files belong together and the layout is right),
  then evaluates positions from Trinity's own calibration games (from ply
  16 on, side to move not in check, 20,000 spread evenly over all games)
  with both networks and reports the gap integer - float in cp: bias, mean
  absolute, median, 95th and 99th percentile, share over 5 and 10 cp, by
  output bucket and by eval size, plus the sign-flip count. It also reports
  the gap with only the first layer rounded and with only the output layer
  rounded, so we know where the error comes from.
- PC script: tools\net\Quant-Check.bat (a few CPU minutes; builds the
  branch inside sprt\coder2, finds the checkpoint whose quantised.bin
  equals main's network, reads sprt\results\*_calibration_*.pgn, saves
  sprt\coder2\results\quantcheck-<date>.txt; deletes nothing). It also
  prints when each checkpoint of the 1024 run was written, which gives the
  training speed for the plan below.
- Checked here with the dequantised network as a stand-in for the float
  weights (gap 0.6 cp = the final rounding to whole cp alone) and with
  weights perturbed by up to 0.45 of a rounding step, which imitates real
  rounding error: mean gap about 6 cp, of which about 6 cp came from the
  output layer and under 2 cp from the first layer.

Prediction (written before the PC run): mean |gap| 4 to 8 cp, 95th
percentile 10 to 16 cp, bias within +/- 1 cp, no bucket clearly worse;
almost all of it from the output layer (1/64 steps on weights of size up
to 2 are coarse; 1/255 steps on the first layer are fine, and the SCReLU
clamp hides much of them). Yardstick: the referee noise in the forensics
is +/- 30-50 cp and a "mistake" there is a 30 cp drop, so a 6 cp random
error adds under 1% to the evaluation's own error. Decision rule: act
(step 2: choose each output weight's rounding direction to cancel the
error, or a finer output quantisation) only if the mean gap is above 8 cp
or one bucket is far worse than the others, or if the layout check shows
the files do not match. Expected Elo from step 2 even then: 0 to +2.

Result (PC, 14 October 18:05 Pacific, sprt\coder2\results\quantcheck-2026-10-14_18-05.txt;
20,000 positions from 39,200 calibration games, 42 PGN files):
- Layout check: 100.000% of all 6.3 million numbers equal. The files
  belong together and the layout is right.
- Gap integer - float: mean |gap| 6.4 cp, median 5.2, 95th percentile
  16.6, 99th 23.2, max 55; 21% of positions off by 10 cp or more; bias
  +0.6 overall; 229 of 20,000 evals change sign (1.1%, all near zero).
- Source: first layer alone 3.4 cp, output layer alone 5.4 cp (they add
  in quadrature: sqrt(3.4^2 + 5.4^2) = 6.4, so the two errors are
  independent). The output layer is the larger part, as predicted, but
  the first layer is not negligible (predicted under 2).
- By output bucket the mean |gap| is even (5.4 to 7.4), but the BIAS is
  not: +5.2 cp at 6-9 pieces, +3.1 at 26-29, +2.9 at 2-5, -4.1 at 30-32,
  -1.8 at 18-21. These are systematic shifts per piece-count range, so a
  capture that crosses from 30-32 to 26-29 pieces looks about 7 cp better
  to the integer network than to the float one.
- Large evals are rounded worse: 7.3 cp at |eval| >= 200 vs 5.7 under 50.
- Prediction vs actual: mean 4-8 (actual 6.4, right); 95th percentile
  10-16 (16.6, just outside); bias within +/- 1 (overall +0.6, right, but
  per bucket up to +/- 5, not foreseen); "almost all from the output
  layer" (two thirds, not almost all).
- Output weights in the file: mean |w| is 17 units of 1/64, 3.4% of them
  are above 64 units (|w| > 1.0), max 127. A finer output scale (1/128)
  would halve the output error but needs the weights clipped at 1.0 in
  training (3.4% of them are larger) or a slower integer path; a
  training-recipe question for a later run, not a patch.

Verdict against the decision rule (act only above 8 cp mean or a bucket
far worse): the random part is below the bar and costs under 1% extra
evaluation error; the per-bucket bias is the one finding worth a cheap
fix. Options for the owner:
- (a) Nothing now. Cost 0.
- (b) Step 2 light: re-round the output layer of main's network to match
  the float output on positions (per bucket: the bias, and for each of
  the 2048 output weights the rounding direction chosen to cancel the
  error; fitted on half of the positions, checked on the other half).
  No training, no speed cost, engine unchanged. Needs: an engine command
  on a net-... branch, a tools\net script on the PC (the float weights
  are only there; a minute of CPU), a branch with the re-rounded network,
  and one GitHub SPRT in the network lane, which is idle while the PC
  trains. Prediction: self-play 0 to +3, most likely about +1; a [0, 5]
  SPRT will most likely end undecided. The procedure then also serves the
  2048 network, whose output layer has twice the terms and so about 1.4x
  the output error. Recommendation: (b) only if the Session has the
  GitHub slot to spare; otherwise fold the procedure into the 2048
  network's quantisation later, where it is tested for free.

Training speed from the checkpoint times: superbatch 50 at 19:25,
600 at 23:59 on 9 October = 0.50 minutes per superbatch, 5.0 hours for the
whole 1024 run (not "under 10"). The 2048 run should take about 8-10
hours of GPU, plus 2-3 hours of CPU for the data: started at 8:30 pm it
is done by about 7-9 am.

### 2. Plan for the next training run (15 October)

What the history says (ESTIMATES.md): capacity transferred to other
engines (king buckets +19 self-play / +17 CCRL; 512 -> 1024 from scratch
+80 / +50), while more or fresh Leela data on a converged network gave
nothing (+6 / ~0; -3.8; weight averaging ~0). So the lever is capacity,
and the honest unknown is how much of the +80 was the width rather than
the from-scratch 600-superbatch run on the whole file.

Measured here (cloud machine with AVX2, main's code of 15 October with
the AVX2 output layer, builds for the machine's own processor, the 1024
network padded with zero neurons so the search is identical: bench 780879
in all three):

| Hidden | nodes/s (mean of 4) | vs 1024 | Elo by the time-odds ruler (0.57 per 1%) |
|---|---|---|---|
| 1024 | 1104k | - | - |
| 1536 | 899k | -19% | about -11 self-play |
| 2048 | 794k | -28% | about -16 self-play |

(Without the AVX2 output layer the costs were 25% and 39%. 512 -> 1024
cost 28% on the PC and still gained +50 CCRL. The accumulator updates,
which also double, are untouched by the AVX2 code; measure the cost on
the PC and the GitHub machines before reading the test.)

Training time: the 1024 run (600 superbatches, 3.65 billion positions,
batch 16384) took 5.0 hours (checkpoint times from the quantisation
check: 0.50 minutes per superbatch). Expect about 1.6-2x for 2048 (8-10
hours) and 1.3-1.5x for 1536, plus 2-3 hours of CPU to convert and
shuffle the Leela file again (it was deleted on 13 October).

Disk: 4 parts x 32 GB = 117 GB of data (the file yields 3.65 billion
positions) plus 32 GB while shuffling: within the 200 GB budget. All of
it lives in sprt\coder2 (data\, checkpoints\, results\); the parts stay
on disk afterwards (a second run reuses them) until Ayman deletes the
folder. The script deletes only its own raw parts after shuffling them
(rule 5: it needs the Session's review before it runs; the deletion is
marked at the top of tools\net\wide.ps1).

The run (ONE change, as the drill and refinement failures advise):
`tools\net\Wide-Network.bat` = big.ps1's recipe exactly (whole Leela
file, from scratch, 600 superbatches, LR 0.001 cosine to 1%, WDL 0.5,
batch 16384, 8 king buckets mirrored, 8 output buckets) with the hidden
layer 2048 instead of 1024. The trainer and the published branch
(`net-wide2048-<date>`) come from branch `net-wide`, which is main plus
engine support for 1536 and 2048 neurons (build.rs and src/nnue.rs; the
1024 build is byte-for-byte the same, bench 780879; both sizes build and
pass the tests here). The trainer's loss per superbatch is saved in
sprt\results\train-<name>.log this time (the 1024 run's log was not
kept), so overfitting of the wider network on 16 passes over the same
3.65 billion positions can be seen rather than guessed.

Prediction (written before the run):
- 2048: self-play vs main at 8+0.08 +15 to +45 (central +30: a raw gain
  of about half of the 1024 run's +95-before-speed, minus 16 for speed);
  CCRL-scale +5 to +25 (central +15). Chance to pass SPRT [0, 5]: about
  70%. If the 1024 gain was mostly the recipe rather than the width, the
  result is -10 to +5 and the width path is closed; that is worth knowing
  either way, and 2048 gives a clearer answer than 1536.
- 1536 (fallback if the PC time is too long or the risk unwanted):
  self-play +5 to +30 (central +15); CCRL +3 to +15 (central +9).

Skeptic's review of these numbers:
- The +80 of the 1024 run bundled width, from-scratch training, 600
  superbatches and 3.65B positions; nobody knows the width's share.
- The speed cost was measured on one cloud CPU; the PC and the GitHub
  test machines may pay more or less (nnue-simd itself ranged from -3% to
  +10% across machines). The ruler (0.57 Elo per 1%) was measured at +12%
  time; a 28% change may not scale linearly.
- A test that stops when it passes over-reads (SPRT #17: +13 that became
  +6 in the repeat). Calibration decides, not the SPRT number.
- 16 passes over the same data: a wider network can memorise more; the
  loss log will show it (training loss falling while the SPRT result is
  flat would be the sign).
- Everything downstream of the SPRT (calibration +/- 15 real uncertainty)
  can hide a +10 as easily as show it.

Decision (owner, 15 October, evening): GO 2048. PC order: Quant-Check.bat,
then Wide-Network.bat overnight (the calibration of the new main was
already done on 14 October: about 3482, +16 over 3466; the 2048 network
will be measured against that). Two conditions:
1. Compare the training loss with the 1024 run at superbatch 300 and
   report. The 1024 run's loss log was never saved (big.ps1 showed it on
   screen only), so, as the owner allows, both runs' checkpoints are scored
   on the same positions instead: tools\net\Net-Loss.bat runs the engine
   command `trinity netloss` (branch net-quantcheck) on every saved
   checkpoint of both runs. Two kinds of positions, both reported (advisor
   and Session, 14 October evening: a wider network can win on training
   positions by memorising them, and no held-out Leela positions exist
   because both runs train on the whole file):
   - held-out: 1,000,000 positions from Trinity's own calibration games
     (39,200 games; ply 16 on, not in check), label = the game's result,
     loss = (sigmoid(output) - result)^2. Neither network ever saw them.
     This is the fair number.
   - training: the first 1,000,000 positions of the shuffled part big-1,
     the trainer's own loss (wdl 0.5) plus the pure-result and pure-score
     losses and the mean cp error. Fit only; the gap between the two kinds
     is the memorisation.
   All 12 checkpoints of the 1024 run (50 to 600) still exist (listed by
   the quantisation check), so the comparison is at every superbatch,
   including 300. Prediction: held-out result loss of the 2048 run 0.5-1.5%
   lower (relative) than the 1024 run's at the same superbatch; training
   loss 1-3% lower at the same superbatch, 2-4% at 600. Held-out not lower
   at all: the width is unused, expect no Elo. Training much lower but
   held-out not: memorisation.
   Same run, same positions (Scout, 15 October, idea 1, adopted): the
   nine networks that were ever in main, read from the repository's
   history (the engine reads their integer files; the two averaged
   networks are gone with their branches). Their Elo steps, from
   ESTIMATES.md, give the exchange rate between held-out loss and Elo:

   | Network (commit) | Self-play step | CCRL step | Note |
   |---|---|---|---|
   | net3 -> net4 (39b8bd1 -> e0c0fee) | +25.9 | +4 | 256 -> 512 neurons, output buckets |
   | net4 -> net5 (32dde25) | +217.8 | +169 | first Leela data |
   | net5 -> net6 (3870ee6) | +29.1 | +3 | 120 instead of 40 Leela superbatches |
   | net6 -> fresh (3aca207) | +6.0 | about 0 | fresh Leela data |
   | fresh -> king buckets (b2fbbb4) | +19.1 | +17 | 8 king buckets, mirrored |
   | king buckets -> big1024 (f3d46ea) | +80 (about +96 before the 28% speed cost) | +50 | 1024 from scratch |

   Prediction for the exchange rate: the held-out result loss falls along
   the whole sequence; the two steps that gained little against rated
   engines (net6: +3, fresh: about 0) differ from their predecessors by
   under 0.3% relative, and king buckets -> big1024 (+50 CCRL) by 2-4%.
   If that holds, about 15-25 CCRL-scale Elo per 1% of relative held-out
   loss, and the 2048 network's loss at 600 prices it before any test.
   Checkpoint safety (Session, 14 October evening): Remove-OldCheckpoints
   in scripts/common.ps1 runs only when 4-Train, 7-Drill, 8-Fresh-Leela or
   11-Big-Network starts; it now never deletes big1024 folders and keeps
   main's whole run. Coder 2's scripts never call it, and Coder 2's
   checkpoints live in sprt\coder2\checkpoints, outside its reach.
2. If 2048 lands between -5 and +5 at 30+0.3, one SPSA re-tune of the
   search settings before the verdict.
3. (Scout, 15 October, idea 2, adopted as the contingency.) If the 2048
   run's held-out loss is not below the 1024 run's at superbatch 300 and
   600, or its curve stalls, the width is not judged used up: the next run
   is 2048 with the learning rate halved (0.0005), everything else the
   same, judged on held-out loss at 300 and 600 before any SPRT. The
   reasoning from first principles: with an Adam-type optimiser every
   weight moves by about the learning rate, so a layer twice as wide moves
   its output about twice as far per step; the rate tuned at 1024 is too
   hot at 2048. Not started before tonight's curves are read.

After a pass (for the Session's queue): v2 check (19-V2-Check.bat),
calibration (5-Calibrate.bat, rule 2), then an SPSA re-tune of the search
settings (owner's rule of 15 October; the last one was worth +5.9).

Parked alternatives (not for the first run): 16 king buckets (capacity
without per-node cost, but a smaller expected gain than width); a second
Leela file (the disk budget caps data at about 4.5 billion positions, so
it adds at most 25%); positions from Trinity's own games labelled by its
deeper search (Scout, 13 October: on-policy data; datagen gives about 100
million positions a day, 3% of the data; a fine-tune experiment only once
the width run is banked); step 2 of the quantisation idea (after step 1).

### 3. Loss comparison and exchange rate: result (PC, 15 October 17:56 Pacific, sprt\coder2\results\netloss-2026-10-15_17-56.txt)

Held-out result loss (1,000,000 positions from Trinity's calibration games;
lower is better; the 1024 run's checkpoint rows are still to be pasted):

| Network | Held-out loss | Training loss (wdl 0.5) |
|---|---|---|
| 2048 run, superbatch 300 | 0.083505 | 0.025476 |
| 2048 run, superbatch 600 | 0.081687 | 0.024974 |
| 1024 run, superbatch 300 | 0.084320 | 0.025975 |
| 1024 run, superbatch 600 (= main's network) | 0.082764 | 0.025627 |
| king buckets (5 Oct) | 0.085218 | 0.027080 |
| fresh (4 Oct) | 0.086187 | 0.027323 |
| net6 (4 Oct) | 0.086201 | 0.027209 |
| net5 (3 Oct) | 0.086343 | 0.027335 |
| net4 (3 Oct) | 0.091305 | 0.030139 |
| net3, net2, net1 | 0.092918, 0.094674, 0.097991 | 0.0306, 0.0313, 0.0335 |

- Full table received (sprt\coder2\results\netloss-2026-10-15_17-56.txt).
  At 300 (the owner's condition 1): held-out 0.083505 vs 0.084320, the
  2048 run 1.0% below; training 0.025476 vs 0.025975, 1.9% below. At 600:
  held-out 1.3% below (predicted 0.5-1.5%: right), training 2.5% below
  (predicted 2-4%: right). The 2048 run is below the 1024 run at every
  one of the 12 checkpoints, and reaches the 1024 run's final held-out
  loss at about superbatch 400-450. Half of the training advantage does
  not generalise (memorisation), but the held-out gain is real.
- Two side findings from the same table. (1) The quantisation costs
  nothing measurable: main's integer network scores 0.082764 against
  0.082759 for the float checkpoint it came from (0.006%), which settles
  the quantisation question (section 1): no step 2. (2) A floor for the
  ruler: the "king buckets, longer" checkpoints of 7 October (kb-130 to
  kb-160) score 0.3% BETTER on held-out loss than the network they
  started from (kb-80: 0.084898 vs 0.085186) yet lost -13.8 and -3.8 in
  self-play. So differences under about 0.4% (about 7 Elo by the ruler)
  say nothing, and the result loss misses something the search needs
  (those runs restarted at a high learning rate). The ruler is for
  pricing steps of 1% and more.
- Exchange rate (Scout idea 1), CCRL-scale Elo per 1% of relative
  held-out loss, from the steps with a calibration: net5 -> net6 -0.16%
  for +3 and net6 -> fresh -0.02% for about 0 (both at the noise floor, as
  predicted); fresh -> king buckets -1.1% for +17 (15 per 1%); king
  buckets -> 1024 -2.9% for +50 (17 per 1%, speed cost included);
  net4 -> net5 -5.4% for +169 (31 per 1%); net3 -> net4 -1.7% for +4 (2
  per 1%: the outlier, measured against the old, out-of-range opponents
  and with a 512 network starved of data). Working figure: about 15-20
  CCRL Elo per 1% (predicted 15-25).
- Priced by the ruler: 2048's -1.3% is worth about +20-25 on the CCRL
  scale before its speed cost (31% fewer nodes per second here; by the
  time-odds ruler about -18 self-play, perhaps -10 to -15 CCRL), so net
  about +5 to +15 CCRL and, by the self-play steps (17-33 per 1%), about
  +5 to +25 self-play. Lower than the logged prediction (+15 to +45
  self-play); the SPRT decides. The logged prediction stands as written.
- Scout idea 2 (halved learning rate) is not triggered: the 2048 run is
  below the 1024 run on held-out loss at 600 and its curve kept falling
  (0.0837 at 250 to 0.0817 at 600, every checkpoint lower).

### 4. Post-mortem of the 2048 network (16 October)

Result: -29.8 +/- 4.2 in self-play (SPRT #23, 9000 games) against a
prediction of +15 to +45 and a 70% chance to pass. Both wrong.

Where the -30 comes from, by the measurements at hand:
- Speed. 31% fewer nodes per second alone here, 34% with three copies
  running at once (the PC runs 14 games and the GitHub machines 5 jobs;
  the 25 MB network shares their caches worse than the 12.6 MB one). By
  the time-odds ruler read as "Elo per doubling of time" (+6.8 for +12%
  gives about 42 per doubling) a 31-34% slowdown costs 22-25 Elo. The
  linear reading I used (0.57 per 1%, -18) understated it, and I never
  measured the cost on the test machines, which ESTIMATES.md's own lesson
  from nnue-simd told me to do.
- Evaluation. The held-out loss gain of 1.3% was priced at +20-25 by the
  new ruler. The result says the evaluation was worth about 0 to -8 after
  the speed cost of -22 to -25. Either the ruler overprices small gains
  (its floor finding in section 3 already said that differences under
  0.4% can go either way; 1.3% was treated as safely above it), or part
  of the gain is lost in the search. The evaluation scale is NOT it: the
  Session measured both networks on 1,505 positions from the latest
  calibration games (ratio 0.99, mean size 247 vs 240 cp); my own 15
  bench positions had suggested 10-15% larger, too few to trust. So an
  SPSA re-tune (owner's condition 2) has little to correct.
- Not the cause, as far as can be checked from here: the engine uses the
  new network (different scores and lines from main on the same
  position), passes its tests at 2048 (incremental updates, refresh
  cache, AVX2 output layer within its overflow bound), lost no games on
  time, and quantisation cost the 1024 network 0.006% of held-out loss.
  Scout's 16 October checks on the integer file: Quant-Check.bat can run
  on the 2048 network with one command line (N4 below), a few minutes.

Lessons (for ESTIMATES.md's calibration of future estimates):
1. A network that costs speed must be priced with the speed cost measured
   under test conditions, and on the log scale; a 30% slowdown is -22 to
   -25, not -18.
2. The held-out loss ruler, on its first real use, over-priced a 1.3%
   gain by 20 Elo or more. Until it has more points, treat 1% as "worth
   perhaps +5 to +15 before speed", not +15-20.
3. Width at 2048 is used by training (every checkpoint better) but not
   worth its per-node cost on this hardware. Capacity that costs no
   per-node speed is what is left: training length, king buckets, data.
4. Scout's "quick look" (16 October): a clearly bad network shows within
   a few hundred games; the SPRT's first report can be read as that, so
   no separate test is needed, but the next prediction should name a
   stop-early threshold.

Proposal for the next run (after #24 closes the width path; for the
owner's decision, not started): the same 1024 recipe from scratch with
1200 superbatches instead of 600 (one change; the cosine schedule
stretched over the longer run). Why: zero speed cost; the 1024 run's
held-out loss was still falling at the end (0.082862 at 550, 0.082759 at
600), and the 2048 run needed only about 150 more superbatches' worth of
capacity to pass it, which suggests training time, not width, is the
cheaper lever. GPU: about 10 hours; disk: the data is already in
sprt\coder2\data (nothing to convert). Prediction: held-out loss 0.5-1.0%
below the 1024 run's; self-play +5 to +15 (with the corrected ruler: 1%
is worth +5-15 and there is no speed cost); CCRL-scale +3 to +10; 55% to
pass [0, 5]. Stop-early threshold: if the SPRT's first 1000 games show
-10 or worse, stop. Alternative of the same cost: 16 king buckets at
1024 (memory only, no per-node cost), predicted +3 to +10 self-play.

### 5. Next run, decided (owner, 16 October): the 1024 recipe plus a second Leela file

One change at a time: main's 1024 recipe exactly as it was (from scratch,
600 superbatches, LR 0.001 cosine, WDL 0.5, 8 king buckets, 1024 neurons)
with more data. Reason (owner and advisor): the 2048 run showed the
networks memorise the file (2.5% gain on training positions, 1.3% on
held-out games); fresh positions attack that directly, at zero speed
cost. King buckets at 1024 (16 instead of 8) is the run after this one.

Reuse and disk (checked 16 October):
- The 4 parts in sprt\coder2\data (117 GB) are the same 3.65 billion
  positions the 1024 run used (the conversion is deterministic; only the
  shuffle differs) and are reused as they are: nothing to convert for
  the first file.
- A second file of the same kind holds about 3.65 billion usable
  positions, but the whole of it does not fit: 162 GB free on 16
  October. Per extra part of a billion positions: 32 GB, plus 32 GB
  while shuffling, 13 GB for the download and 20 GB spare.

  | Parts from the second file | Needs free | Data on disk after | Positions | Passes in 600 superbatches |
  |---|---|---|---|---|
  | 1 | 97 GB | 149 GB | 4.65 billion | 12.9 |
  | 2 (default) | 129 GB | 181 GB | 5.65 billion | 10.6 |
  | 3 | 161 GB | 213 GB | 6.65 billion | 9.0 (over the 200 GB budget) |

  Default 2 parts (`New-Data-Network.bat`); 3 with `-SecondParts 3` if
  the owner accepts 213 GB on disk. The 1024 run made 16.4 passes over
  its data; with 2 extra parts every position is seen 10.6 times.
- What Ayman must do first (the Session found the source, 16 October):
  the first file is test80-2024-02-feb-2tb7p.min-v2.v6.binpack from the
  Hugging Face dataset linrock/test80-2024; download a neighbouring
  month's file of the same name pattern (about 13 GB) into a new folder
  C:\Trinity\data\leela2 (the Session's scripts expect exactly one file
  in data\leela, so the second one must not go there).
- Reviewed by the Session (16 October): safe to run. Its suggestion is
  in: the files are given to the trainer interleaved (big-1, second-1,
  big-2, second-2, big-3, big-4), so the final low-learning-rate stretch
  is not mostly one month.

Script: tools\net\New-Data-Network.bat (newdata.ps1: wide.ps1 with the
width back at 1024, the trainer and the published branch from main since
the engine is unchanged, the second file converted into second-<n>.data,
all parts given to the trainer). It deletes its own second-<n>.raw files
after shuffling (rule 5: the Session's review before it runs). Time: about
1 hour of CPU for 2 parts, then about 5-6 hours of GPU (the 1024 run took
5.0 hours; 600 superbatches is the same work whatever the data size).

Prediction (written before the run): held-out result loss 0.3-0.8% below
the 1024 run's at 600 (less memorisation: the gap between training and
held-out loss narrows); self-play +3 to +12 (central +6; the corrected
ruler, no speed cost); CCRL-scale +2 to +8; about 50% to pass [0, 5].
Stop-early threshold: -10 or worse after the first 1000 games. If the
held-out loss is not lower at all, more of the same data is not the
lever either, and king buckets come next.

### 6. 16 king buckets (owner, 18 October; the Session)
Decided after the skeptic review set teacher-student aside (ESTIMATES.md,
parked ideas). One change to main's 1024 recipe: 16 king buckets instead
of 8, from scratch on the same 3.65 billion positions (sprt\coder2\data,
nothing to convert), 600 superbatches, LR 0.001 cosine, WDL 0.5.
Layout (own king, mirrored onto files a-d): rank 1 and rank 2 one bucket
per square; ranks 3 and 4 one bucket per pair of files; ranks 5-6 and
7-8 one bucket per half-board. Every new bucket lies inside one old
bucket, so the shared "factoriser" part still carries what all buckets
have in common, and main's network copied into the new layout
(tools/net/expand_kb16.py) gives exactly main's bench (780879): the
engine side is checked without any training.
Speed: no measurable cost here (bench of the copied network, 4 runs and
3 copies at once, within the machine's +/- 10% noise; the input weights
double to 25 MB, but a position only touches its own bucket's 1.5 MB).
GitHub's machines (speed check of `kb16-speed`, main's network copied,
against main, 3 copies at once, 30 runs each, 18 October): -1.8%, -0.9%
and -1.9% on the three machines, about -1.5%, i.e. about -1 Elo by the
time-odds ruler (42 per doubling). Counted as a same-speed network for
the loss gate (0.3%).
Branches: `net-kb16` (engine + trainer support; main's network, bench
780879), `kb16-speed` (speed check only, not for merging).
Script: tools\net\KB16-Network.bat (kb16.ps1), dry run
Dry-Run-KB16.bat (1 superbatch, save, load in the engine, bench). Writes
only in sprt\coder2; deletes only its own dry-run folder.
Prediction (written before the run):
- Training time 5-8 hours (the optimiser updates twice the input weights).
- Held-out loss: 0.1-0.6% below main's 0.08276.
- Self-play vs main at 8+0.08: -8 to +10, central +2; CCRL-scale -5 to +7.
  About 30% to pass [0, 5] (lower than the 40% first quoted to the owner:
  the more-data network had a better loss and still lost 11, and nobody
  knows how much two from-scratch runs of the same recipe differ).
- Stop-early threshold: -10 or worse after 400 games.
Gates (the owner's rules of 16-17 October): held-out loss at least 0.3%
better than main's (same speed); 400 games at 8+0.08 on GitHub, stop at
-10 or worse; full SPRT [0, 5] capped at 18000 games; then calibration on
the runner the same night as 1.0.0 (rule 13), the v2 check and an SPSA
re-tune before merging.
Gate for THIS run (Ayman's decision via the advisor, 18 October, before
any result; replaces the 0.3% loss gate for this run only): held-out
loss worse than main's -> stop; 0-0.3% better -> still play the 400 quick
games at 8+0.08; more than 0.3% better -> as planned. The games decide.
0.3% stays the rule for later networks until the control run gives the
run-to-run noise. A full SPRT of it may not hold up search-idea SPRTs on
GitHub for more than one day.
Data (checked 18 October): the real run reads exactly big-1..4.data in
sprt\coder2\data, 3.65 billion positions; the dry run's "1 billion" is
its single file (it trains 1 superbatch on big-1.data only). These are
the same positions main's network learned from, converted the same way,
but shuffled anew (main's own shuffled files were deleted on 13 October),
so the order differs from main's run; the control run below uses these
same files, which makes control vs 16 buckets a clean one-change pair.
Dry run (PC, 18 October 9:01): 42 s per superbatch (the 8-bucket network
about 30), file 25200704 bytes, loads in the engine (bench runs): the
real run takes about 7 hours.
Control run (advisor, owner, 18 October: the next GPU job whatever this
run gives): main's recipe unchanged (8 king buckets) with a new random
start (weight seed 20261018; bullet's default is fixed, and main used
it) on the same files as this run: tools\net\Control-Network.bat
(kb16.ps1 -KingBuckets 8 -Seed 20261018 -Tag control; branch
net-control-<date>). About 5 hours. Its SPRT against main measures how
much two from-scratch runs of one recipe differ (seed and shuffle);
16 buckets is then judged against the control as well as against main.
Result, training (PC, 18 October): 6.0 hours (38 s per superbatch), branch
net-kb16-20261018-0919, bench 928782 (same on the PC and here).
Gate 1, held-out loss (Session, 18 October 5:20 pm Pacific; the calibration
PGNs are on the PC only, so measured here on 150,341 positions from fresh
Trinity self-play games, 5000 nodes per move, played by main's build;
pure game-result loss, the fair column, because the other labels are
main's own search scores): main 0.064102, 16 buckets 0.063929, i.e.
0.27% better. Inside Ayman's 0-0.3% band, so the 400 quick games are
played (SPRT #28, 80 games x 5 jobs at 8+0.08, started 5:21 pm). The
loss-blend column favours main (0.019911 vs 0.019966), as expected when
the scores come from main's own search; it is not the gate. Under 0.4%
the ruler says nothing (section 3): the games decide.
Screen (SPRT #28, 400 games at 8+0.08, finished 5:32 pm): +18.3 +/- 18.9
(W 109, L 88, D 203; 0 time losses, 0 illegal moves, 0 crashes), well
above the -10 stop line. A quick look over-reads (#17: +13 became +6);
the full SPRT decides. Full SPRT continued from #28 at 5:33 pm (1800
games per job; cap 18000 games and one day of GitHub time).
Advisor's notes on the screen (18 October evening), logged before the
full SPRT ends: (1) first case where the 0.3% loss gate would have
stopped a network that then passed the 400-game screen; it feeds the
decision on the gate after the control run. Caveat: the 0.27% was
measured on fresh self-play positions, not on the calibration games the
0.3% gate was defined on, so "would have stopped" is likely, not
certain. (2) A pass against main makes it mergeable; "the buckets caused
it" needs 16 buckets vs the control (the clean one-change pair); no plan
for 32 buckets before that. (3) Expect the +18 to shrink toward the
central prediction (+2; range -8 to +10); no announcement as a win until
the full SPRT and the slower-clock test are done.
Two more rules (owner, 18 October evening, before any further result):
(a) "slower network" = more than 5% fewer nodes per second than main on
GitHub's speed check with 3 copies at once; only those also need 30+0.3.
16 buckets measured 1.5% slower, so 8+0.08 decides for it. (b) The
control network gets the same 400 quick games at 8+0.08 against main.
Session's caution: two 400-game screens each carry +/- 19, so their
difference carries about +/- 27; a control at +10 vs 16 buckets' +18
cannot separate "buckets" from "new shuffle and start". If 16 buckets
passes, the clean test is a direct match 16 buckets vs control.
Advisor, 18 October late (not a merge gate): if 16 buckets passes, it
joins the runner's 40/60 nights next to main and 1.0.0, to see how it
holds at long clocks before CCRL does. Noted: the 5% line was set after
16 buckets' speed (1.5%) was known, so it did decide that case; accepted
as reasonable on its own terms (1.5% is worth about 1 Elo).
SPRT #29 (continues #28; finished 19 October 4:21 UTC): 9400 games,
-2.00 +/- 3.96 (W 2239, L 2293, D 4868), LLR -2.35 of -2.94, 0 time
losses. The +18 of the screen shrank as the advisor expected. Continued
to the 18000-game cap (#30, 1720 games per job, 19 October 3:35 am
Pacific; no search SPRT is waiting, and the one-day limit ends about
5:20 pm). Most likely outcome: FAIL; an undecided result at the cap also
means not merged.
RESULT (SPRT #31, 19 October 7:15 am Pacific, at the 18000-game cap):
FAILED, -2.10 +/- 2.85 (W 4265, L 4374, D 9361; LLR -4.66; 0 time
losses). Not merged. Prediction -8 to +10, central +2: inside the range,
below the centre; "about 30% to pass" was too high. Held-out loss 0.27%
better and +18 on the screen: neither predicted the result, again.
Control run (PC, 18 October 17:02-22:06, 5.1 hours, 30.4 s per superbatch):
branch net-control-20261018-1702, bench 846777. 400-game screen vs main
started 19 October 3:34 am Pacific.
Control's longer test, what each outcome means (advisor and Session,
19 October 7:15 am Pacific, before any result beyond the 400-game
screen's -14.8 +/- 18.8): the control's SPRT against main is continued to
about 3000 games (+/- 7 at that length), after the 16-bucket test ends.
- Within +/- 5 of main: run-to-run luck is small; the failed networks
  were really worse, and new networks keep being judged against main.
- -10 or worse: luck is large; any new recipe needs two training runs
  (two seeds) before a verdict.
- Between -5 and -10 (or anything the +/- 7 cannot place): extend to
  about 6000 games before reading it.
Added by the advisor, 19 October, before the result:
- 16 buckets vs the control, head to head (same data order, one change),
  is played after the control's extended test, whatever 16 buckets did
  against main: if the control is really about -10, the buckets may be
  worth about +8 over the same data and start. It decides the next
  network run, not a merge.
- Best of several starts (Scout, 19 October; parked in ESTIMATES.md):
  if the control's extended test shows -8 or worse against main, it is
  taken up with the idle graphics card; within +/- 4 it is dropped.
  The Session's first objection ("mostly picks the luckiest") was too
  pessimistic: +/- 10 is the 95% range at 2000 games (sigma about 5), and
  with a true run-to-run spread of about 10, the best of 4 still gains
  about +9 (100 / sqrt(100 + 25) x 1.03); with a spread of about 3, about +2.
Control's extended test, step 1 (SPRT #32, 19 October 8:55 am Pacific,
3000 games): -8.92 +/- 7.02 (W 680, L 757, D 1563; 0 time losses).
By the rules written before it: between -5 and -10 -> extended to about
6000 games (#33, started 9:00 am); best-of-N trigger (-8 or worse):
MET at this length, so best of several starts is un-parked (plan and
prediction to follow, after the 6000-game reading, before any training).
Reading for now, as advised: "run-to-run differences may be large"; if
it holds, main is a high draw of its recipe and 16 buckets (-2 vs main)
may be about +7 over the same data and start: the head-to-head decides.
Control's extended test, final (SPRT #33, 6000 games): -9.67 +/- 4.92
(W 1346, L 1513, D 3141). By the rules written before: about -10, so
run-to-run luck is large: a new recipe needs two training runs before a
verdict, and best of several starts stays un-parked. Reading, kept
modest (one control run): main is probably a high draw of its recipe,
or the reshuffled data costs a little; the 2048 and more-data failures
(-30, -11) are still mostly real, but part of the -11 may be this.
Head-to-head 16 buckets vs control (same data, same order, one change):
SPRT [0, 5] at 8+0.08, up to 9000 games, started 19 October 5:24 pm
Pacific (#34). Prediction (before it): +7 +/- 7 (16 buckets -2 vs main,
control -10 vs main); about 50% to pass.
Advisor, 19 October evening, before Control2's result:
(1) Rule out setup drift before more starts. Code check (Session, done):
today's trainer differs from main's (8b4d54f) only by the 16-bucket
option and the seed option, both no-ops at 8 buckets and the default
seed; Cargo.lock and the conversion are unchanged; same command line as
big.ps1. Empirical check, to run on the PC after Control2:
tools\net\Drift-Check.bat trains 1 superbatch with each trainer (1024,
8 buckets, default seed, big-1.data) and compares loss and files.
Prediction: identical, or the same loss within GPU rounding. If not, the
-10 may be a setup change, and extra starts would not fix it.
(2) "About 10 Elo of run-to-run luck" is one sample: estimated range
3-15 until Control2 and the drift check are in. Best of several starts
on the 8-bucket recipe is expected to recover main's level, not beat it
by much; the real lever is 16 buckets, if it beats the control head to
head, in which case further starts use 16 buckets.
Drift check (PC, 20 October): old trainer (8b4d54f) loss 0.042848, new
trainer (net-kb16) loss 0.042848 after 1 superbatch, same data and seed:
equal to all printed digits; the saved files differ only by GPU
rounding. Verdict: NO setup drift, as predicted. The control's -10 is
run-to-run variation (random start and/or the new data order), not a
setup change.
Head-to-head result, first half (SPRT #34, 9000 games, finished 19
October): 16 buckets vs control +2.47 +/- 4.09 (W 2217, L 2153, D 4630),
undecided. Prediction +7 +/- 7: lower than predicted. With the two tests
against main (16 buckets -2.1, control -9.7, implying about +7.6 +/- 5.7)
the best joint guess for the buckets' effect is about +4, small and
uncertain. Continued to 18000 games (cap; no search test waiting).
Control2 (PC, 19 October 17:25-22:24, 5.0 hours, 29.9 s per superbatch):
branch net-control2-20261019-1725, bench 850479. 400-game screen vs main
started 11:44 pm.
Second control run (19 October, owner's daytime-GPU plan with the
advisor): tools\net\Control2-Network.bat (main's recipe unchanged, weight
seed 20261019, same data files; branch net-control2-<date>), about 5
hours, run in the day. It is both a second sample of the run-to-run
spread and a best-of-several candidate. Prediction (before training):
vs main -9 +/- 10 (if the spread is real, it lands anywhere from about
-20 to +2); vs the first control within +/- 10. It gets the same
400-game screen, then the extended test only if the screen is above -10.
How to read the control (advisor, 18 October, before its result;
wording corrected 19 October): main's recipe was trained once and
passed by +80, so main was not SELECTED for luck from several runs. It
is still one draw of its recipe and may have landed high by chance; the
control measures exactly that. One control run is one sample with its own test error: a
control at -5 to -10 means "run-to-run differences may be large", not
that they explain the 2048 and more-data failures, which most likely
stand. A second control run would be needed to say more.
Skeptic's note: if it fails narrowly, the next useful run is a control
(main's recipe, unchanged, new random start) to measure how much two
from-scratch runs differ; without that number every network result
since 9 October is hard to read.

## PC time requests (for the Session's PC queue in ESTIMATES.md)

| # | Job | Script | PC time | Status |
|---|---|---|---|---|
| N1 | Quantisation check, step 1 | tools\net\Quant-Check.bat | a few minutes of CPU (any time nothing else heavy is running; a GitHub test does not matter) | ready; deletes nothing |
| N2 | Wide network training, 2048 | tools\net\Wide-Network.bat | 2-3 h CPU (convert + shuffle) then about 8-10 h GPU (the 1024 run took 5.0 h); no calibration or SPRT on the PC meanwhile | DONE 15 October 17:51 Pacific: branch net-wide2048-20261015-0709 (10.7 h of training, 72 s per superbatch = 2.4x the 1024 run, above the 1.6-2x guessed; the first start failed on a script bug of Coder 2's, fixed by the Session, data kept) |
| N3 | Loss of every saved checkpoint on the same positions (condition 1): held-out game positions and training positions | tools\net\Net-Loss.bat | a minute or two of CPU, after N2 | DONE 15 October 17:56 (section 3) |
| N5 | More-data 1024 network (section 5): after Ayman has put the second Leela file in C:\Trinity\data\leela2 | tools\net\New-Data-Network.bat | about 1 h CPU then 5-6 h GPU; no calibration or SPRT on the PC meanwhile | ready after the download and the Session's review of newdata.ps1 (deletes its own raw parts) |
| N4 | Quantisation check of the 2048 network (Scout, 16 October): open a Command Prompt in C:\Trinity\tools\net and type `Quant-Check.bat -Branch net-wide2048-20261015-0709 -Checkpoint C:\Trinity\sprt\coder2\checkpoints\trinity-20261015-0709-wide2048-600` | tools\net\Quant-Check.bat | a few minutes of CPU, any time | optional; closes Scout's finding |

## Test requests (lane "network")

| # | Branch | Test | Prediction (written before) | Status |
|---|---|---|---|---|
| T1 | net-wide2048-20261015-0709 (commit 028a643, main merged in on 16 October UTC; bench 781681; 2048 neurons; builds and passes the tests; about 31% fewer nodes per second than main here) | SPRT vs main at 8+0.08, bounds [0, 5] | self-play +15 to +45 (central +30); CCRL-scale +5 to +25; about 70% to pass | FAILED (#23): -29.8 +/- 4.2 after 9000 games, 0 time losses. Prediction badly wrong; post-mortem in section 4 |
| T2 | same | 3000 games at 30+0.3 (the owner's condition 2 reads this result: -5 to +5 means one SPSA re-tune first; clearly negative closes the width path) | after #23: -20 to -35 (the 1024 network scored the same at both clocks); clearly negative, about 90% likely | running (#24, started by the Session 16 October) |

Then, if it passes: v2 check, calibration, SPSA re-tune (CLAUDE.md,
Coder 2 rule 4). The loss comparison (N3) runs on the PC meanwhile and
does not block T1.

(Earlier note: after N2 finishes and Coder 2 has updated the branch from main:
an SPRT of `net-wide2048-<date>` vs main at 8+0.08, bounds [0, 5]; if it
passes, 3000 games at 30+0.3 as for the 1024 network (the owner's
condition 2 reads the 30+0.3 result).
Branch state on 15 October: net-quantcheck and net-wide both carry main's
combo merge of 15 October, build, pass the tests, bench 780879.)
