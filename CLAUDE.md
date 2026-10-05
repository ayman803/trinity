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
