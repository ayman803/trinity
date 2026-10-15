# Working with the owner of this project

The owner is not a programmer and follows instructions step by step on
Windows. Every instruction must be concrete:

- Name the exact button, file, menu item or setting, and say where it is
  on screen (for example "the mark one step left of the knob, labelled
  Recommended").
- Say what they will see when it worked, and what to send back.
- Never leave them to infer a meaning ("normal effort", "the usual way").
  If unsure what their screen shows, ask for a screenshot instead of
  guessing.
- Never guess elapsed time. Check the clock with `date -u` and use the
  times shown in their screenshots. The owner is on US Pacific time.
- Keep replies short: credit is limited. Only message results that need
  a decision; batch steps where possible.
- Effort setting: Medium ("Recommended") for routine steps; High only for
  errors or new design work.

## Machines

- PC: i9-10900KF, RTX 3080, 32 GB, Windows 10, repo at C:\Trinity. Does
  data generation, training (GPU), calibration and SPRT tests.
- Laptop: i7-1355U, 32 GB, Windows 11, repo at C:\Trinity. Runs SPRT
  tests only.

## Standing rules (set by the owner, October 2026)

1. Goal: CCRL top 100 (about 3455) within one year, starting October 2026.
   Stop rule: if it is clearly off track, say so plainly.
2. After every new network, calibrate against rated engines
   (`5-Calibrate.bat`). Keep at least two opponents where Trinity scores
   between 25% and 75%; replace any opponent outside that range.
3. Always distinguish self-play Elo (SPRT vs `main`) from CCRL-scale Elo
   (calibration). Expect self-play gains to shrink against other engines.
4. Log every predicted gain next to the actual result in `ESTIMATES.md`,
   and use that history to calibrate future estimates.
5. Protect credit: no scheduled check-ins or PR watching. Wait for the
   owner to paste results.
6. When Trinity is near top-100 strength, prepare a release for the CCRL
   testers.
7. Before each next step, briefly review the plan as a skeptic: what could
   be wrong with these numbers?

## Release checklist (before any CCRL submission)

1. Check the source for close similarity to major open-source engines,
   especially Rust ones (akimbo, Viridithas, Reckless, Svart, etc.).
2. Run a move-similarity test against Stockfish, Reckless and a few
   others, and report the numbers.
3. Add a CREDITS section to README.md covering the bullet trainer, the
   Leela/Stockfish training data, and that the code was written with
   Claude.
4. Syzygy tablebases (CCRL testers use 3-4-5 piece tables): supported
   since 5 October 2026 via shakmaty-syzygy (owner chose GPL; Trinity is
   GPL-3.0-or-later). Calibration passes C:\Trinity\syzygy to all
   engines when that folder exists. Measure the gain with calibration,
   not SPRT.

## Originality rule (set by the owner, 13 October 2026)

Absolute, for the Session, Fable and Scout alike. For every new idea from
now on, do not read other engines' code, commit logs, test logs, or forum
posts about their techniques. Drop any idea you recognise as another
engine's. Allowed sources: Trinity's own code and data, first principles,
and science from outside computer chess. Ideas already in Trinity stay.
One exception: the release checks may run other engines (similarity test)
and compare their source text mechanically to prove Trinity copies
nothing; nothing learned there may be used.
Second exception (owner, 13 October 2026): loss forensics may use an
outside engine (Stockfish) purely as a referee that scores positions, to
find which of Trinity's moves was the mistake. Nothing is taken from how
that engine works.

## Scout (daily idea task)

A scheduled task runs at 5:46 am Pacific and writes scout/YYYY-MM-DD.md
plus one line per idea in scout/INDEX.md. It never touches code, tests or
ESTIMATES.md. Read scout/INDEX.md before planning the next step.

## Coder 2 (network work, set by the owner, 14-15 October 2026)

A second coding session (Fable by default) owns the network: the design of
the next training run (data, size, length) and Scout's quantisation check.
1. Coder 2 works only on branches named `net-...` and never pushes to
   `main`. The Session tests and merges everything into `main`.
2. Coder 2 keeps its notes in NETWORK.md on the fixed branch `net-notes`
   (the Session and Scout read it there; the Session merges it into main
   from time to time) and never edits ESTIMATES.md. Only the Session edits
   ESTIMATES.md.
   Coder 2 runs in the cloud and cannot reach the PC: it gives Ayman exact
   steps (a .bat file on its branch) for anything that must run there.
   The integer network is in the repo (nets/default.nnue); the float
   weights exist only on the PC (checkpoints\...\optimiser_state\weights.bin).
3. The originality rule applies to Coder 2 too, including Coda by name.
4. Tests: Coder 2 asks for a test in NETWORK.md; the Session adds it to the
   test queue in ESTIMATES.md (lane "network") and runs it like any other.
   Every network merge follows the usual steps: SPRT, a combination test if
   needed, the v2 check (19-V2-Check.bat), and calibration (5-Calibrate.bat).
5. GitHub test machines: at most 2 tests at once for the Session's lane
   (search) and at most 1 for Coder 2's lane (network).
6. The PC runs one heavy job at a time (training, data preparation or
   calibration). Ayman decides the order.
7. Disk budget for training: about 200 GB (C: had 265 GB free on 15
   October). Training data takes 32 GB per billion positions, plus about
   52 GB for shuffling and spare (scripts/big.ps1).
