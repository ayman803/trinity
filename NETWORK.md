# Network notes (Coder 2)

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
then 5-Calibrate.bat on the new main (the Session's), then
Wide-Network.bat overnight. Two conditions:
1. Compare the training loss with the 1024 run at superbatch 300 and
   report. The 1024 run's loss log was never saved (big.ps1 showed it on
   screen only), so, as the owner allows, both runs' checkpoints are scored
   on the same positions instead: tools\net\Net-Loss.bat runs the engine
   command `trinity netloss` (branch net-quantcheck) on every saved
   checkpoint of both runs with the first 250,000 positions of the shuffled
   data part big-1 (a random sample), the trainer's own loss (sigmoid
   output vs the wdl-0.5 target), plus the pure-result and pure-score
   losses and the mean cp error. Caveats: these are training positions
   (both runs trained on the whole file; no held-out positions exist, and
   holding some out would change the recipe, so that is for a later run);
   and the Session's housekeeping keeps only main's checkpoint and the 4
   newest, so the 1024 run's checkpoint 300 may be gone (Quant-Check.bat
   lists what is left). Then the comparison is at the superbatches both
   runs still have (probably 450-600) and at 600. Prediction: at the same
   superbatch the 2048 run's loss is 1-3% lower (relative), at 600 about
   2-4% lower. Not lower at all: the width is unused, expect no Elo. Much
   lower (over 6%) with a flat Elo result: memorisation of the data.
2. If 2048 lands between -5 and +5 at 30+0.3, one SPSA re-tune of the
   search settings before the verdict.

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

## PC time requests (for the Session's PC queue in ESTIMATES.md)

| # | Job | Script | PC time | Status |
|---|---|---|---|---|
| N1 | Quantisation check, step 1 | tools\net\Quant-Check.bat | a few minutes of CPU (any time nothing else heavy is running; a GitHub test does not matter) | ready; deletes nothing |
| N2 | Wide network training, 2048 | tools\net\Wide-Network.bat | 2-3 h CPU (convert + shuffle) then about 8-10 h GPU (the 1024 run took 5.0 h); no calibration or SPRT on the PC meanwhile | GO (owner, 15 October); scheduled after the Session's calibration; wide.ps1 deletes its own raw data parts (rule 5: for the Session's review) |
| N3 | Loss of every saved checkpoint on the same positions (condition 1) | tools\net\Net-Loss.bat | about a minute of CPU, after N2 (or during it for an early look at checkpoint 300: it is light) | ready once N2 has made the data; deletes nothing |

## Test requests (lane "network")

(none yet; after N2 finishes and Coder 2 has updated the branch from main:
an SPRT of `net-wide2048-<date>` vs main at 8+0.08, bounds [0, 5]; if it
passes, 3000 games at 30+0.3 as for the 1024 network (the owner's
condition 2 reads the 30+0.3 result).
Branch state on 15 October: net-quantcheck and net-wide both carry main's
combo merge of 15 October, build, pass the tests, bench 780879.)
