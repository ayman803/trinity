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
   owner to paste results. One exception (owner, 18 October 2026): a daily
   morning check-in (scheduled task "Trinity morning check-in", 6:20 am
   Pacific, in a persistent session with the repository attached) reads
   trinity-results, updates ESTIMATES.md, and queues next jobs only from
   already-approved scripts and already-planned work; a read-only task
   ("Trinity phone summary", 6:35 am) sends the owner a short phone summary. It never merges, never changes scripts that need
   approval, and never starts training or data generation. It also keeps
   the "Weekly count" line (Pace rules, ESTIMATES.md Test queue) current:
   candidates tested and merged this week (owner, 20 October). When it
   queues, a script edited after the last approval is not a reason to
   wait: the runner always runs the approved version, so it is enough that
   the approved version accepts the job's parameters (19 October;
   this overrides the stricter "unchanged since the approved commit" test
   in the check-in's own instructions).
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
before any code (with the advisor's and Scout's additions):
1. It runs under a separate standard (non-administrator) Windows account,
   `trinity-runner`, in its own clone `C:\TrinityRunner\repo`. It may
   write only there and in `C:\TrinityRunner\state`. `C:\Trinity`, where
   Ayman runs scripts, is read-only for it (Windows permissions, set by the
   installer, which also removes the default write access every signed-in
   account gets on folders under `C:\`). Big data and checkpoints are
   reached read-only through links in its clone.
2. Approval pins one commit. `C:\TrinityRunner\approved.txt`, written only
   by Ayman with Approve-Scripts (never in GitHub), holds one approved
   commit, the fingerprint of everything under `scripts/`, `tools/` and
   `runner/jobs/` at that commit, and a time limit per script. The runner
   checks out exactly that commit, checks the fingerprint, and runs the
   job's `.ps1` directly with self-update off (`-NoUpdate` and
   TRINITY_NO_UPDATE); newer scripts wait until Ayman approves a newer
   commit. Only the queue itself is read from the latest `main`.
3. Results go to a separate repository, `ayman803/trinity-results`. The
   runner's GitHub key is a fine-grained token for that repository only
   (Contents: read and write), stored outside both repositories; it can
   read the public `trinity` repository but never change it.
4. One job at a time; each job stopped at its time limit; no job starts
   below 50 GB free and a running job is stopped below 30 GB; queue
   arguments are checked against a simple pattern. Only scripts with a
   results-repository mode may be queued (the runner's own list).
5. It deletes nothing outside its own folders; irreplaceable files are
   protected by rule 1's read-only permissions, not by promises.
6. If `C:\TrinityRunner\STOP` exists it starts nothing new
   (Stop-Runner / Start-Runner shortcuts).
7. Every job is logged in `trinity-results` (`runner/log/`).
8. The installed runner is a copy in `C:\TrinityRunner` (read-only for the
   runner account); changes in `runner/` on GitHub reach the PC only when
   Ayman reinstalls.
9. Windows Update may not restart the machine on its own, the machine
   never sleeps while a job runs, and no job starts while a restart is
   pending (Ayman restarts when convenient). (Owner and advisor, 17
   October:) otherwise the PC may sleep after an hour unused; a wake timer
   wakes it at 11 pm and 6 am to check the queue, so a job queued in the
   day may wait until 11 pm.
10. (Advisor, 17 October) Approve-Scripts itself lists new risky lines
   (deletes, moves, downloads, git push, Windows settings), so Ayman does
   not rely only on the Session's summary. At most one approval a day (a
   second one needs typing AGAIN). Every job's log line names the approved
   commit. trinity-results is public: no user names, personal paths or
   device ids in it.
11. Real jobs (all but the hello test) start after 30 minutes without
   keyboard or mouse use, at any hour (owner and advisor, 19 October: the
   runner is for more PC use, not a night shift). An idle beacon in
   Ayman's own account writes his last keyboard or mouse time to a file
   the runner can only read. If he comes back during a job, the job stops,
   does not count ("DISTURBED") and runs again later (at most 3 times). If
   the beacon is silent, jobs start only between 11 pm and 7 am.
12. First real jobs: a 200-game calibration of release 1.0.0 (plumbing
   check), then a full known-answer calibration of 1.0.0 (expected about
   3466), then current `main`.
13. (Advisor and Scout, 18 October) Release 1.0.0 is the standing reference:
   re-calibrated on the runner once or twice a week, and on any night a
   candidate is calibrated. A version's strength is reported as its gap
   over 1.0.0 measured the same night, not as a single number (1.0.0 gave
   3466 by day on 12 October and 3478 overnight on the runner). Runner
   measurements count when the PC was idle throughout (day or night;
   disturbed ones are dropped); they are not mixed with manual runs made
   while the PC was in use. (The 12-Elo day/night gap of 18 October is
   itself +/- 14, so it is no evidence of a day/night bias.) A gap against a 1.0.0 run from another
   night is labelled "reference from <date>" (conditions alone moved 1.0.0
   by 12). When CCRL publishes its own rating
   for 1.0.0, that becomes the anchor for converting gaps to the CCRL scale.
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

A scheduled task runs at 6:44 am Pacific (after the 6:20 am morning
check-in has filled in the overnight results) and writes scout/YYYY-MM-DD.md
plus one line per idea in scout/INDEX.md. It never touches code, tests or
ESTIMATES.md. Read scout/INDEX.md before planning the next step.

## Pace rules (owner with the advisor, 20 October 2026)

Network lane: about 20% of GitHub test time at most, one experiment at a
time. Search/speed lane: at least 2 new candidates tested per day
(forensics, Scout's list, the speed profile), predictions first. Details
in ESTIMATES.md, Test queue.

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
- Every network merge: SPRT (8+0.08, and 30+0.3 for slower networks:
  more than 5% fewer nodes per second than main on GitHub's speed check
  with 3 copies at once; owner, 18 October, before the 16-bucket result), the
  v2 check (19-V2-Check.bat), calibration, then an SPSA re-tune.
- Disk budget for training: about 200 GB. Training data takes 32 GB per
  billion positions, plus about 52 GB for shuffling and spare.
- The big1024 checkpoints are never deleted (reference run).
