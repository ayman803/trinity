# Coder 2's PC scripts

Double-click scripts for the network work (Coder 2's lane; see CLAUDE.md
and NETWORK.md). They run on Ayman's PC like the ones in the main folder:
each `.bat` first brings `C:\Trinity` up to date, then runs its `.ps1` next
to it. Engine code and networks they need are built from `net-...`
branches; nothing here changes `main`'s engine.

Rules (owner, 15 October 2026): these scripts write only inside their own
output folder, `C:\Trinity\sprt\coder2\` (source, builds, data,
checkpoints, results), and never delete or move anything outside it. A
script that deletes or moves files says so at its top, and the Session
reviews it before Ayman runs it. `sprt\coder2` can be deleted at any time
to free space; everything in it is rebuilt or re-made when needed.

| Script | What it does | Time | Deletes |
|---|---|---|---|
| `Quant-Check.bat` | Integer vs float network on positions from Trinity's calibration games (Scout's quantisation idea, step 1). Writes `sprt\coder2\results\quantcheck-<date>.txt`. | a few CPU minutes | nothing |
| `Wide-Network.bat` | Trains a 2048-neuron network from scratch on the Leela data with the same recipe as main's 1024 network (one change: the width) and publishes it as a `net-wide2048-<date>` branch for SPRT. Data in `sprt\coder2\data` (117 GB), checkpoints in `sprt\coder2\checkpoints`, loss log in `sprt\coder2\results\train-<name>.log`. Do not start it before the Session has scheduled it (NETWORK.md). | 2-3 h CPU, then about 15-20 h GPU | its own `big-<n>.raw` files after shuffling (needs the Session's review) |
| `Net-Loss.bat` | Scores every saved checkpoint (the 1024 run in `checkpoints\`, Coder 2's runs in `sprt\coder2\checkpoints\`) on the same positions: held-out ones from Trinity's calibration games (label = game result; no network trained on them) and, if `sprt\coder2\data\big-1.data` exists, training ones too; also every network that was ever in `main` (from the repository's history), for the loss-to-Elo exchange rate. Writes `sprt\coder2\results\netloss-<date>.txt`. | a minute or two | nothing |
| `New-Data-Network.bat` | The 1024 recipe exactly as main's network was trained, plus a second Leela file (`C:\Trinity\data\leela2`): reuses the 4 converted parts in `sprt\coder2\data`, converts 2 more parts from the second file (64 GB), trains from scratch, publishes `net-data1024-<date>`. | 1 h CPU, then about 5-6 h GPU | its own `second-<n>.raw` files after shuffling (needs the Session's review) |
