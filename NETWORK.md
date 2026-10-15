# Network notes (Coder 2)

Coder 2 owns this file from 15 October 2026 (see CLAUDE.md, "Coder 2").
It lives on `main` (Coder 2 may push NETWORK.md and tools/net/ there);
the branch `net-notes` is no longer used.
The first section is the Session's handover; everything after it is Coder
2's. Ask for tests at the end of this file; the Session queues and runs
them (ESTIMATES.md, "Test queue", lane "network").

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

## Test requests

(none yet)
