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
8. Revive rule (owner, 17 October 2026): once the goal is met, or progress
   flattens (no merged gain over several weeks of steady testing), park
   Trinity rather than force big bets, and revive it when a new generation
   of Claude models is available. The revived session starts by reading
   ESTIMATES.md (lessons, parked ideas) and proposes fresh ideas under the
   same rules.

## PC job runner (owner, 17 October 2026; design in docs/runner-design.md)

A runner on the PC (and optionally the laptop) takes jobs from
`runner/queue.txt` on `main` and runs them unattended. Rules, fixed
before any code:
1. It runs under a separate standard (non-administrator) Windows account,
   `trinity-runner`, which may modify only `C:\Trinity` and its own
   folders. Ayman's files are closed to it by Windows.
2. It runs only scripts listed in `C:\TrinityRunner\approved.txt`, which
   only Ayman edits (never in GitHub). Each entry holds the script's
   fingerprint (with the shared helper files) and a time limit; a changed
   script stops running until Ayman approves it again.
3. Its GitHub key is a fine-grained token for `ayman803/trinity` only
   (Contents: read and write), stored outside the repository.
4. One job at a time; each job stopped at its time limit; no job starts
   below 50 GB free and a running job is stopped below 30 GB; queue
   arguments are checked against a simple pattern.
5. It deletes nothing outside its own folders; job scripts keep their own
   rules (never the big1024 checkpoints).
6. If `C:\TrinityRunner\STOP` exists it starts nothing new
   (Stop-Runner / Start-Runner shortcuts on Ayman's desktop).
7. Every job is logged to `runner/log/` on GitHub.
8. The installed runner is a copy outside the repository; changes in
   `runner/` on GitHub reach the PC only when Ayman reinstalls.
The Session adds jobs to `runner/queue.txt`; it never gets a connection
into the PC.

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

## Coder 2 (folded back, 16 October 2026)

From 14 to 16 October a second coding session ("Coder 2") owned the
network. By the owner's review rule its first trained network had to pass
SPRT; the 2048-neuron network failed (-29.8 at 8+0.08, -19.7 at 30+0.3),
so on 16 October the network work folded back into the Session. Its tools
stay: NETWORK.md (notes, history) and the PC scripts in `tools/net/`
(Quant-Check, Net-Loss, Wide-Network, New-Data-Network), now maintained
by the Session. They write only inside C:\Trinity\sprt\coder2.
Standing parts of that setup that still apply:
- The PC runs one heavy job at a time; Ayman decides the order. The PC
  queue is in ESTIMATES.md.
- Every network merge: SPRT (8+0.08 and 30+0.3 for slower networks), the
  v2 check (19-V2-Check.bat), calibration, then an SPSA re-tune.
- Disk budget for training: about 200 GB. Training data takes 32 GB per
  billion positions, plus about 52 GB for shuffling and spare.
- The big1024 checkpoints are never deleted (reference run).
