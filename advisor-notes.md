# Advisor notes

Notes from the advisor (Ayman's main contact) for the Session and Scout.
Rules: CLAUDE.md, "No relay". The pace routine reads the open notes every
morning, acts on them within the rules, and marks each one under it with a
dated "Done: ..." or "Not done: <reason>". Only the advisor adds notes.
Decisions reserved for Ayman still go to him as "Decision needed".

## Open

### 2026-11-22 · Eval scale, not just pessimism: three more CCRL games (advisor)
Games #46 (Hobbes, draw at move 142), #53 (PZChessBot), #60 (ZigQueen)
added to ccrl/2026-11-21-tournament.md and games/ccrl. #46 confirms the
queen-ending misjudgement (Trinity -3.06 for 28 moves, Hobbes +0.30,
draw). #53 shows the same inflation the other way (Trinity +0.9 to +1.5
where PZChessBot said +0.1 to +0.55; draw). Widen the 21 November
forensics: measure 1.0.0 and main on the drawn positions of #24, #46
and #53 (every 5th move after move 30), report the size of Trinity's
eval against the opponent's and against the result, and test the
hypothesis "eval scale too large / no shrink toward 0 in drawish
endings". If main (WDL 0.25) already shrinks them, log it as a side
benefit of the WDL change; if not, propose a candidate (eval-scale or a
drawishness damping term) with a prediction, as a normal test.

### 2026-11-22 · Add an ARM64 (linux-aarch64, musl) build: 1.0.0 now, every release from 1.2 (Ayman, 22 November, 8:08 am: "yes, add ARM")
GitHub issue #2 (Arzam18, for the Mobile Chess Engine Circuit, MCEC: an
Android engine circuit on a phone): "Could you add linux-aarch64 build
that is linked against musl for arm64 linux and Android?" Ayman: yes.
Session:
1. release.yml: add a target aarch64-unknown-linux-musl, statically
   linked (one file, no system libraries: that is what lets it run on
   Android). Build on GitHub's arm64 Ubuntu runner if available to this
   public repo, else cross-compile (cargo-zigbuild or cross) and run the
   bench under qemu. Name: trinity-<version>-linux-aarch64-musl.
   The engine already has non-x86 fallbacks (nnue.rs scalar path,
   tt.rs prefetch), so no engine code should change; if any does, it
   goes through the guard and a test like any engine change.
2. Build it for release 1.0.0 from the exact 1.0.0 commit. Gate: bench
   must equal 780879 (identical play), `uci` must answer `id name
   Trinity 1.0.0`, and a short test (a few games or a 1000-node search
   at several depths) without a crash. Add the file to the existing
   v1.0.0 release only when the gate passes (an added platform, not a
   new version; no other change to that release or its notes except
   one line listing the new file). Log the bench and speed in ESTIMATES.
3. Tell Ayman when it is up (the phone summary / evening email), with
   the link, so he can post his own "it's up" comment on issue #2.
   Nobody but Ayman comments on the issue.
4. From 1.2 on, the aarch64-musl file is part of every release and of
   the release checklist (same bench gate).
Not now: a hand-tuned NEON path for ARM speed; only if Ayman later says
mobile results matter.

Done (Session, 22 November): 1. release.yml builds
trinity-<version>-linux-aarch64-musl on GitHub's arm64 runner (static,
same bench gate), commit 956f85b; no engine code changed. 2. New workflow
release-add.yml built 1.0.0 from faef85c6 (tag checked): bench 780879 at
1.05M nps, `id name Trinity 1.0.0`, searches to depth 16 and four games
without a crash or illegal move; only then attached to v1.0.0 with one
added line in the notes (run 45043557965). Logged in ESTIMATES (Release
checks). 3. Ayman told with the link; nobody else comments on issue #2.
4. In the Release 1.2 list (item 6).

### 2026-11-21 · URGENT for forensics: Trinity 1.0.0 badly misjudges a queen ending (live, Trinity - Hobbes 4.0, ccrl.live/16066)
Ayman, 6:17 pm: "pass on to session now". Screenshots in games/ccrl
(16066-trinity-vs-hobbes-*.jpg); the game file follows when it ends.
Evals as shown on ccrl.live (White's side; Trinity is White):
- move 16: Trinity -0.69, Hobbes -0.14
- move 23: Trinity -2.22, Hobbes -0.41
- move 32 (both engines give the SAME line: 32...Qxd3 33.Qxd3 c1=Q+
  34.Kg2 Qxb2+ 35.Kg3 fxg4 36.hxg4, a queen ending, Black one pawn up):
  Trinity -2.73, Hobbes -0.32
- move 37 (Q+2 vs Q+3): Trinity -3.04, Hobbes -0.35
- move 41: Trinity -3.24; Hobbes -0.31 with a main line in which White
  regains e5 (42.Qa8+ Kh7 43.Qe4+ Qg6 44.Qxe5 Qxg4): level material.
So 1.0.0's network calls a queen ending one pawn down (or level) about
-3, where the opponent and general endgame knowledge say "drawish".
Same direction as #24 (StockDory) and the opening of this game.
Session, today if possible:
1. Take the positions from the screenshots (and the PGN when it comes)
   and run 1.0.0 AND current main (wdl25-900) on them: if main's evals
   are sane (|eval| about 0.3-1 for Q+1 pawn up), the WDL work already
   fixed it; log that. If not, it is a real network weakness.
2. If main is also far too pessimistic: a forensics set of queen endings
   (Q+k vs Q+k-1 pawns, from Trinity's own games and the tablebase
   positions we have) with Syzygy truth, to measure the error and decide
   whether training data needs more such endings. Predictions first.
3. Check how the eval scale (EVAL_SCALE, the x1.2 scaling) affects such
   positions: an over-scaled eval inflates exactly these numbers.
No action that changes main without the usual tests.
Done (2026-11-22, Session): positions rebuilt from the screenshots (no PGN yet) and run through 1.0.0, long900 and main (wdl25-900): all three give about -2.4 to -2.7 after 32.Qc3 and in the queen ending after 36.hxg4 with Black to move, and about -0.2 to -0.3 with White to move (checks found). Static network evals of queen endings a pawn up: 0.35 to 1.95 pawns, main no better than 1.0.0 (and its x1.2 scale inflates them). So (1) not fixed by the WDL work: a real network weakness; (2) the tablebase measurement needs the PC (the tables cannot be fetched from the cloud: proxy 403), a small script and an approval; planned after the 1 December freeze, with the training-data question after it; (3) the scale: EVAL_SCALE 120 inflates these numbers; an endgame-only scale is a possible one-change test, also after the freeze. Details: ESTIMATES.md, "Queen endings".
Correction (2026-11-22, Session, after Scout's check of 995 queen endings in Trinity's own games and the Stockfish 16 referee): "a real network weakness" went too far. The referee gives -1.19 after 32.Qc3 and -1.08 in the ending with Black to move (0.00 with White to move): Black is better, and Trinity's -2.7 overstates it by about 1.5 pawns. In Trinity's own games a typical one-pawn-down queen ending gets about -0.7 and was drawn 97% of the time. So: one unconfirmed sample, no pattern; nothing planned unless more cases appear. ESTIMATES.md "Queen endings" corrected.

### 2026-11-21 · Trinity's speed on Graham's machine: 1.22M nps (advisor, from a ccrl.live screenshot)
Ayman's screenshot of ccrl.live/16066 at 5:52 pm Pacific (Trinity 1.0.0
White vs Hobbes 4.0, after 10. g4): Trinity at depth 26, 14.4M nodes,
1.22M nodes per second, on one physical core of Graham's Ryzen 5950X
(image: games/ccrl/16066-trinity-vs-hobbes-speed.jpg). Ours, 12 October:
bench 1.62M nps with the windows-x86-64-v3 (AVX2) release file and 1.28M
with the x86-64-v2 file, on the PC. Game positions with a big hash run
slower than bench, so 1.22M fits either "v3 with game overhead" or "v2".
Session: measure Trinity 1.0.0's nps in game conditions on the PC (a few
40/15-like games, hash 256 MB, v3 and v2 release files) to tell which
build Graham is likely running. If the v3 game figure is clearly above
about 1.35M, he is probably on v2 (about 20% slower, maybe 10-15 Elo):
the 1.2 package README must then say plainly which file to use, and
Ayman may choose to mention it to Graham himself. No contact otherwise.
For scale, other engines seen live: Ayano 1.18M, Ember 564K, Hobbes 488K.
Not done yet (2026-11-22, Session): no approved script measures nps in game conditions at 256 MB hash with the release files. Planned as part of the freeze checks (the stability run at CCRL's conditions, advisor's 1 December plan): it will log nps for the v3 (AVX2) and v2 release builds on the PC, and the 1.2 README will say which file to use.

### 2026-11-21 · Tournament games and what the rank means: see ccrl/2026-11-21-tournament.md
Trinity 1.0.0 is 3/5 unbeaten in Graham's "Hobnobbing With Royalty"
(12+8, 16 engines, ccrl.live/16066), with a won attack vs Valkand (#39)
in which its eval led the opponent's by about 2 pawns for 10 moves, and
a held -0.9 book position vs clrsrc (#32). Full record, field, rules
and the reading of rank 76-78 in ccrl/2026-11-21-tournament.md. Weekly
CCRL check: add Trinity's later games from this event to that file and
watch 1.0.0's rating as they come in. Scout may use the two games
(Trinity's own games) as forensics material; the opponents' moves teach
nothing about their code and are not a source of ideas.

### 2026-11-21 · Trinity 1.0.0 entered in a CCRL tournament (Graham to Ayman, 20 November)
Graham's email of 20 November: Trinity 1.0.0 plays in his event
"Hobnobbing with Royalty EQ2026": 12 minutes + 8 seconds, 4 cycles, 60
rounds, 16 engines (Hobbes 4.0, Cinder 0.6.1, PZChessBot 8.0, Crownd 1.0,
ZigQueen 6.6.0, Ayano 0.11.0, Heimdall 1.5.2, NGN 0.4.0, NARC Genesys
5.1, StockDory Barracuda, Trinity 1.0.0, clrsrc 1.4.0, Erebus 2.2.7,
Valkand 1.1, CeylonDemon 3.3, Ember 1.4.0); live at ccrl.live/16066.
About 60 more games for 1.0.0, which feed its CCRL rating. Weekly CCRL
check: note 1.0.0's game count and rating as these games are added; a
move of more than about 20 is worth a line in ESTIMATES. Nothing to do
otherwise; no contact with CCRL except by Ayman. The 1 December freeze
is unaffected.

### 2026-11-21 · Freeze 1.2 on 1 December (Ayman, 21 November, 7:27 am: "freeze 1 December")
CCRL list of 21 November: Trinity 1.0.0 ranked 76-78 at 3522 +/- 26 (262
games); rank 50 = 3573. Ayman: the next version to Graham is "the very
best we have". Plan, replacing the 25-28 November target of the 14
November note:
- Until 30 November: network lane runs moredata-600 (queued) and the 1350
  run (length lever, replicated twice), each with its test vs main as
  soon as trained; whichever passes AND clears its checks (40/60
  calibration, v2 check; merge on Ayman's OK) by 30 November goes in.
  Search candidates only if a real idea exists. Nothing is held back
  waiting for one more result: what main holds on 1 December is 1.2.
- 1 December: freeze main as release 1.2 (tag only, as planned; no
  release page until Ayman says). Then the release checks of the 14
  November note: similarity checks and CREDITS (start now if not
  done), same-night 40/60 calibration of the frozen build and 1.0.0, a
  stability run at CCRL's conditions (40/15 scaled, hash 256 MB, Syzygy,
  no pondering), Windows builds (general and AVX2), README saying it
  supports Threads, a clean-machine test on the laptop.
- About 8 December: package ready; Ayman writes to Graham himself.
Rule 2 flag from 21 November: akimbo at 67% vs main; plan its
replacement opponent before the freeze so the release calibration stays
inside 25-75%.
Noted (2026-11-22, Session): (a) the 1350 run is not queued: at WDL 0.25 length added nothing measurable (wdl25-900 vs wdl25 at 600: -1.0 +/- 3.9, SPRT #133); the length lever was replicated only at WDL 0.5. The network lane's freeze candidate is moredata-600 (trained last night), now in its test against main. (b) akimbo: a replacement opponent needs Ayman to download one engine; candidates should sit about 3560-3620 on CCRL's 1-CPU list (the weekly check of 21 November lists single-CPU entries at ranks 40-60: Chess System Tal 3.00, Devre 7.0, Minke 7.0.0, JustBot 0.5.0, Motor 0.9.0, ZigQueen 6.4.0, Deep Sjeng 3.6). Put to Ayman as a choice in the next routine once the advisor has picked one or two.

### 2026-11-20 · Night-sleep should also cover nights with no jobs (advisor)
Reinstalled 19 November 18:17 (night-sleep in place). That night the
queue was empty and no job ran, so no job "ended" and the PC stayed awake
all night (Ayman asked at 3 am whether anything was running). Extend the
rule (CLAUDE.md runner rule 9's intent: "put the pc to sleep when Trinity
is done for the night"): from 21:00, if the queue is empty or nothing
fits before 06:55, and 30 minutes without input, sleep, whether or not a
job ran that night. Same guards (no job running, no restart pending, no
STOP, not by day). Needs a reinstall; write Ayman's steps when ready.
Done (2026-11-20, Session): runner.ps1 (commit 167dd2c): the night sleep no longer needs a job to have ended that night; all other guards unchanged (tested with simulated states). Ayman's steps are in ESTIMATES.md, Decisions needed (20 November), together with the approval for kb16.ps1 -MoreData (Scout's moredata-600).

### 2026-11-18 · Sleep the PC when the night's work is done (Ayman, 18 November, 7:16 am)
Ayman: "put the pc to sleep when Trinity is done for the night .. schedule
it". Written into CLAUDE.md runner rule 9. Session, runner change (needs
a reinstall; write the Install-Runner steps for Ayman when ready):
between 21:00 and 06:55, after a job ends, if no queued job fits before
06:55 (by rule 14's own estimate) or the queue is empty, and the beacon
shows 30 minutes without input, log "SLEEP: night's work done" and put
the PC to sleep (rundll32 powrprof.dll,SetSuspendState 0,1,0 with
hibernation off, or a scheduled task in Ayman's own account that the
runner triggers, if the runner account cannot sleep the machine). Never
during a job, never by day, never with a restart pending. Sleep: Never
stays the Windows setting. Runner-Status should show the SLEEP line.
Also record in the weekly check: hours of PC work per day (about 5 on
average over 4-17 November, peak 9.6) so the load stays visible; Ayman
is checking temperatures and drive health this week.
Done (2026-11-19, Session): built in runner.ps1 (commit after c6ab357): at night, after a job has ended, if the queue is empty or nothing fits before 06:55, with 30 minutes without input (or nobody signed in), no restart pending, no STOP and no other Trinity program: log and publish "SLEEP (the night's work is done ...)", enable the runner account's shutdown privilege and call SetSuspendState (sleep, wake timers on). A failure is logged ("SLEEP failed: ...") and the PC stays awake. Decision logic tested in the cloud with simulated states; the real sleep can only be seen on the PC after the next reinstall (Ayman's steps in today's reply). The SLEEP line goes to trinity-results, so the check-in sees it. Hours of PC work per day: for the Saturday weekly check.

### 2026-11-17 · wdl25-900 passed vs main; its checks queued tonight (advisor, 6:40 pm, Ayman: "PC is free, start now")
#132: net-wdl25-900-main vs base-1117 PASSED, +5.54 +/- 3.80, 9400 games
(8+0.08, [0, 5]). The queue was empty (wdl25-900s2 finished 13:46), so I
queued the network-merge checks CLAUDE.md requires, same night: 40/60
calibration of net-wdl25-900-main, then 1.0.0 at 40/60 (rule 13
reference), then its v2 check (2026-11-17-01..03). Session, on the
morning run: log the prediction for the 40/60 gap BEFORE reading the
result if possible (prior: main's +78 +/- 21 plus 0 to +6), start the
SPSA re-tune of wdl25-900 if not already planned, start the
wdl25-900s2 test vs main (the replication), and write the merge as
"Decision needed: merge wdl25-900 after its checks?" when the checks and
re-tune are in. It is the 1.2 network candidate if it clears them by the
freeze (about 25-28 November).
Done (2026-11-18, Session): 40/60 prior logged next to the result (main +78 plus 0 to +6; result +74 +/- 22 the same night). SPSA re-tune: its chain (2 runs on the 600 version, EVAL_SCALE stayed at 120) moved to wdl25-900 for run 3 today. Before the replication, Scout's head-to-head wdl25-900 vs wdl25 x1.2 runs first (one network test at a time): it picks the 1.2 network and settles whether length still helps at WDL 0.25; the wdl25-900s2 test follows it. "Decision needed: merge wdl25-900" goes to Ayman when run 3 and the head-to-head are in.

### 2026-11-17 · New goal: CCRL top 50 (Ayman, 17 November, 5:07 pm)
Ayman: "change our goal to be ranked in top 50" (CLAUDE.md rule 1 now
says so). The advisor's "about 3600, roughly rank 50" of this morning was
an extrapolation from the ranks seen around 1.0.0 (70-77 at 3522-3533),
not a reading. Session: in the weekly CCRL check (ccrl/YYYY-MM-DD.md)
record the ratings at ranks 45, 50 and 55 of the best-versions list and
how many of ranks 40-60 are 4CPU entries; put the rank-50 rating in the
Test queue next to our CCRL-scale estimate for main (1.0.0's CCRL rating
plus main's same-night 40/60 gap, labelled as an estimate). Two levers to
keep in view, no action yet: (1) networks (wdl25-900 and its
replication), the main source of gains since October; (2) multi-threading:
4 threads vs 1 measured +99.6 (12 October) and +79.5 (23 October) in our
games, and the list ranks 4CPU entries alongside 1CPU ones. Whether
Trinity is ever tested at 4CPU is the testers' choice and goes through
Ayman alone; the release README should simply say that Trinity supports
Threads.
Done (2026-11-21, Session): first weekly check with the new items (ccrl/2026-11-21.md): 1.0.0 3522 +/- 26 (262 games), now ranked 76-78; rank 45 3579, rank 50 3573, rank 55 3568; 14 of 21 entries at ranks 40-60 are 4CPU. In the ESTIMATES.md Calibrations section next to our CCRL-scale estimate for main (1.0.0's rating plus the same-night gap): about 3596 at 40/60, about 3635 at 10+0.1, labelled as estimates with the scale caveat. PC hours per day for 14-21 November logged there too (about 4.8 on average).
Noted (2026-11-18, Session): the weekly CCRL check (Saturday 21 November) will record ranks 45, 50 and 55 and the 4CPU count in ranks 40-60; the README line on Threads is in the Release 1.2 item 3 draft, for the freeze.

### 2026-11-17 · wdl25-900 paused 19 superbatches short of the end (advisor)
First resumable run under rule 18 worked: started 18:20 with -Minutes 510,
saved and paused at superbatch 881 of 900 at 01:40 (7h19m of training),
output in trinity-results runner/output/2026-11-14-01.txt. It was NOT a
keyboard stop (Ayman was away); the time budget ran out: actual pace 32.2
s per superbatch against the planned 0.5 min (30 s), plus start-up and
save time. When the budget would leave less than about 30 superbatches,
let the run finish instead of pausing (finishing 10 minutes past the
budget is fine at 1:40 am; the hard limit is 6:55). Use the measured
pace from the run's own log for the next estimate. Minor: the job's last
lines show "fatal: unable to get credential storage lock ... Permission
denied" after the part was saved; check the result still reached
trinity-results every time (it did this time) and silence or fix it.
Done in part (2026-11-17, Session): kb16.ps1 now finishes instead of pausing when 30 superbatches or fewer would be left (commit 63dc150; needs the next approval, not urgent: the pause cost one 11-minute restart). The pace estimate stays 0.55 min per superbatch: the 881 superbatches took 440 minutes including start-up, so it was right; only the last stretch was the problem. Not done: the credential-lock message. It is git's "store" helper trying to save the key after a push into the runner's read-only secret folder; the pushes themselves succeed (both parts' files are in trinity-results). Harmless; left as it is unless a push is ever missing.

### 2026-11-16 · PC asleep all day; reinstalled runner still treats the -Resume job as "long" (advisor, from Runner-Status 5:09 pm)
Runner-Status, 16 November: "woke from sleep (asleep since about 06:57)"
at 16:53:45, so the PC slept from 6:57 am to 4:53 pm; it also slept at
22:46 and 23:06 on 15 November. Sleep: Never is not holding (or Ayman is
putting it to sleep by hand; asked). Session: have the installer, and
Runner-Status, print the ACTIVE power plan's AC standby timeout
(powercfg /query SCHEME_CURRENT SUB_SLEEP STANDBYIDLE) and hibernate
timeout, so we see the real setting, not the Settings page. Also check
hibernate ("asleep" in the log may be hibernate; HIBERNATEIDLE) and set
it to 0 as well. Second: Ayman reinstalled at 16:57:48 ("Runner started
on 'pc'"), and the very next line is "Waiting: 2026-11-14-01 is a long
job; long jobs start from 21:00" although that job has -Resume (rule
18). Either Install-Runner did not get the 16 November runner.ps1 (its
pull of C:\Trinity may have failed silently) or rule 18 is not in the
installed code path. Make Runner-Status print the runner's version (the
commit of runner/runner.ps1 it was installed from) on its first line so
this is checkable from a screenshot; and make Install-Runner stop with a
clear message if the pull fails. Approval of the 16 November scripts
not yet confirmed (no STARTED line since the reinstall).
Done (2026-11-17, Session): cause found on 16 November from Ayman's screenshots: C:\Trinity had not been updated since late October (d73bd78), so both the 13 and 16 November reinstalls ran the old installer (no Away, no rule 18, and it set Sleep back to 1 hour each time, hence the sleeping PC). After git pull (to 67add28) and a third reinstall the new runner ran rule 18 the same evening (wdl25-900 started 6:20 pm with -Minutes 510; approval 52f6e65 confirmed by that line). As asked: Runner-Status now prints the installed version (C:\Trinity commit and runner.ps1's last change) and the active plan's AC sleep and hibernate timeouts; Install-Runner.bat stops with a message if its git pull fails (commit 63dc150, with the next reinstall). Hibernate is already set to 0 by the installer.

### 2026-11-14 · Night of 14-15 November: fallback 40/60 pair queued (advisor; Ayman: "off PC for 12 hours, make up lost time")
Ayman is away from the PC from about 4:40 pm to 4:40 am. After the
Friday jobs (dry run, lowlr-tail, 10+0.1 pair, done about 11 pm) the
wdl25-900 training (7.5 h) fits only if it starts by about 10:35 pm;
otherwise the runner would idle the rest of the night. So I appended a
same-night 40/60 pair, main then 1.0.0 (2026-11-14-02/03, 60 rounds each,
about 2 h each), as the fallback: it is the headline number for the
release (the last 40/60 gap is from 24 October, before four merges) and
it is needed before the freeze anyway. The runner takes the first job
that fits, so wdl25-900 still goes first when it fits. Session: log the
pair's prediction in ESTIMATES.md on the morning run (my prior: gap +70
to +100 at 40/60, from +105 at 10+0.1 and the earlier 40/60:10+0.1 ratio
of about 0.6-0.8), and if only one of the two ran, label the gap
"reference from <date>".
Done (2026-11-15, Session): prior logged next to the result in ESTIMATES.md (Calibrations): main at 40/60 3577, +67 +/- 21 over 1.0.0 "reference from 23 October", just below +70 to +100; 1.0.0's own 40/60 (2026-11-14-03) did not fit and is still queued, so a same-night pair is still owed before the freeze.

### 2026-11-14 · Saturday evening: Friday's jobs first (Ayman, 14 November, 4:20 pm Pacific: "catch up with what was missed Friday")
Ayman retired early and left the PC free at about 4:20 pm, Sleep: Never
confirmed by his screenshot (Power & sleep: Screen 10 minutes, Sleep
Never), and was asked to click "Trinity Away". At his word the advisor
moved 2026-11-14-01 (wdl25-900) behind Friday's missed jobs, so the
queue order tonight is: 2026-11-13-03 (resume dry run), -04 (lowlr-tail),
-01 (main at 10+0.1), -02 (1.0.0 at 10+0.1), then 2026-11-14-01
(wdl25-900, about 7.5 h). Expected, if the first job starts about 4:40
pm: dry run and lowlr-tail done by about 5:50 pm, the calibration pair by
about 11:10 pm; wdl25-900 then needs a slot ending by 6:55 am (the runner
judges kb16 jobs by their longest recent run, 448 min, so it must start
by about 10:35 pm): it runs tonight only if the calibrations finish
early, otherwise it waits for the next idle slot (Sunday from 9 pm, or
earlier if Ayman clicks Away again). Session: nothing to change; log the
results as usual, and a lowlr-tail result is still void if the dry run
failed.
Done (2026-11-15, Session): the jobs started at 9:04 pm, not 4:40 pm (they all waited for 9 pm as "long" jobs; if Away was clicked, any keyboard or mouse use more than a minute after the click cancelled it: from the next reinstall the grace is 10 minutes). Dry run PASSED (resumed vs uninterrupted 0.008%, run-to-run 0.009%), so lowlr-tail counts; it trained in 54 minutes, is not broken (held-out loss 0.081623 vs Control2 0.081577) and its games vs Control2 started today. wdl25-900 is tonight's first job.

### 2026-11-14 · Release prep for 1.2 starts now (Ayman, 14 November, 10:04 am: "start release prep")
Preparing is not offering: rule 6 still holds (nothing goes to CCRL until
1.0.0's rating has settled and Ayman says so; nobody contacts Graham but
him). Target: freeze main about 25-28 November, checks done by 2-5
December. Session, on quiet days from now, in this order, each logged in
ESTIMATES.md under a "Release 1.2" heading:
1. Release checklist item 1: mechanical source-similarity check against
   the major open-source Rust engines (akimbo, Viridithas, Reckless,
   Svart, and others); report the numbers. Originality rule: compare text
   mechanically, learn nothing from it.
2. Item 2: move-similarity test against Stockfish, Reckless and a few
   others (a GitHub job or a PC job on the approved scripts); report the
   numbers.
3. Item 3: CREDITS section in README.md (bullet trainer, Leela/Stockfish
   data, written with Claude); no CCRL mention anywhere.
4. Package plan: Windows builds (general and AVX2), default network
   inside, version string "Trinity 1.2", README that says which build to
   use, a test on the laptop (unzip, run, one game). Write the steps for
   Ayman now; run them at the freeze.
5. At the freeze (Ayman's word, not before): same-night 40/60 calibration
   of the release build and 1.0.0; one stability run at CCRL-like
   settings (40/15, large hash, Syzygy 3-4-5) checking crashes and time
   losses; then the package.
The network in 1.2 is whatever main holds at the freeze (long900 now;
wdl25-900 if it passes and is merged by then); no waiting for one more
candidate. Scout: no release ideas needed; keep to Elo.
Not done yet (2026-11-15, Session): "Release 1.2" heading made in ESTIMATES.md with the five items; none started today (the network tests and the runner fixes took the day). Item 1 needs the other engines' public repositories added to the session for a read-only, mechanical comparison.

### 2026-11-14 · Runner silent 13-14 November: two causes (advisor, from Runner-Status)
Runner-Status (14 November 7:57 am): the runner ran all along. (a) It
waited for 21:00 for every queued job, including the 10-minute resume
dry run: "Waiting: 2026-11-13-03 is a long job; long jobs start from
21:00". Rule 17's long/short test goes by the script's time limit (kb16
= 9 h), so a -DryRun job counts as long. Session: classify by job, not
script: kb16.ps1 -DryRun (and any job whose own argument fixes a short
run) is short; or give the dry run its own job script with a short
limit. (b) The PC slept at about 17:05 on 13 November and did not wake
until Ayman turned it on at 07:27 on 14 November: it slept through the
21:00 start, the 23:00 wake timer and the 06:00 wake timer. The reinstall
appears to have put Sleep back to one hour (Ayman had set Never on 12
November; the installer's "keeps Sleep: Never" did not hold, to be
confirmed by his screenshot). Wake timers are now proven not to fire;
with Sleep at Never they are not needed, so the installer should set and
keep Sleep: Never explicitly (powercfg -change standby-timeout-ac 0) and
say so in its output. Missed: 2026-11-13-01..04 and the long1350 parts;
all still queued. Also log the CCRL result (3527 +/- 42, 100 games) as
provisional: re-score the forecast only when CCRL's testing of 1.0.0 is
complete and the rating is ranked.
Done (2026-11-15, Session): (a) runner.ps1: a -DryRun job with no history now counts as 30 minutes (the dry run's own history, 6 minutes, already makes later dry runs short); (b) Install-Runner.ps1 sets Sleep: Never with powercfg and checks it, and the one-shot wake test is gone. Both from the next reinstall (not urgent: Sleep is Never by hand now). The CCRL result is logged as provisional (Calibrations, ESTIMATES.md); forecast re-scoring waits for the settled, ranked rating. The long1350 parts were taken out on 14 November (Scout: retrain at WDL 0.25 if wdl25-900 confirms).

### 2026-11-13 · Reinstall and approval done; wake test inconclusive (advisor)
Ayman reinstalled the runner at 7:10 am (Away button, Sleep: Never kept)
and approved a55d874 (3 files, 3 Remove-Item lines) at 7:13. Installer
wake test: he put the PC to sleep at about 7:14; Runner-Status shows
"woke from sleep (asleep since about 07:21)" at 07:38:16, the wake being
his own. The one-shot 7:16 alarm fell before the PC was actually asleep,
so the wake timer is still unverified. Not urgent while Sleep is Never.
When the runner is next changed: make the installer's wake test arm an
alarm for "now + 6 minutes" only after it has seen the PC enter sleep
(or tell Ayman to put it to sleep first and arm the alarm from the
runner's next tick), so the test is conclusive. Also: the 6:20 check-in
posted nothing this morning (no commit by 7:40); if its session is
blocked or its wake failed, say so in ESTIMATES.md and in the evening
update.
Done (2026-11-13, Session): the check-in was not blocked, only late: it fired at 7:44 am and posted 8cfa31b (wdl25 trained; Friday calibrations queued); Scout also ran about 40 minutes late, so the morning tasks were delayed, not broken. With the approval in, the resume dry run and lowlr-tail are queued ahead of tonight's calibrations (ESTIMATES.md, Test queue item (6)). The installer's wake test (arm the alarm only after the PC is asleep) is noted for the next runner change; not urgent while Sleep is Never.

### 2026-11-12 · Runner silent 10-12 November: cause found (advisor, from Runner-Status on the PC)
The runner was alive throughout. Runner-Status log: after each wake from
sleep with nobody signed in, "Idle time unknown (the idle beacon has not
written for 3 minutes); real jobs wait for 11 pm" (11 Nov 17:28 and
22:01, 12 Nov 07:38); the beacon writes only while Ayman is signed in
(it wrote at 7:39 today, right after sign-in). At 11 pm on 11 November
the PC was asleep: woke 22:01 (asleep since 17:55), slept again 22:06, no
wake at 23:00, next wake 05:03, by then too late for wdl25 before 6:55.
Also 10 Nov 07:40: one "git fetch failed: could not resolve host
github.com" (transient DNS after wake). Today's workaround: Ayman sets
Sleep to Never so the 11 pm window is reachable; the three queued jobs
(2026-11-11-01..03) should run tonight. For the Session, runner work
(not under the approval fingerprint, needs a reinstall): (1) find why the
11 pm wake timer did not fire (the install may need `powercfg` wake-timer
settings checked and a retry at 23:05/23:10); (2) when the PC wakes by
timer and the beacon is stale, that is exactly the idle case: consider
treating "beacon stale AND no interactive session unlocked" as idle so
timer wakes in the day can start short jobs (rule 11's intent, Ayman and
advisor 19 October: the runner is for more PC use). Put a proposal in
ESTIMATES.md; a rule change goes to Ayman as a Decision needed.
Partly done (2026-11-12, Session): at Ayman's request ("start in 5 minutes"; "Yes" to a button) the runner gets a desktop button "Trinity Away": after he clicks it, jobs start 5 minutes after his last use instead of 30 and long jobs need not wait for 9 pm, until he touches the keyboard or mouse again (24 hours at most). It still needs a working beacon, so it does not cover your point (2) (stale beacon after a timer wake). The installer now keeps Sleep: Never if Ayman set it (a reinstall would have reset it to 1 hour), and Install-Runner.bat pulls the newest files first. Not done yet: (1) why the 11 pm wake timer did not fire, and (2) the stale-beacon proposal; both for the next pace routine.
Not done (2026-11-13, Session): (1) and (2) deferred: with Sleep set to Never the PC no longer sleeps, so the 11 pm wake and the stale beacon after a timer wake do not block jobs now (wdl25 ran on 12 November at 6:06 pm). They come back if Ayman wants the PC to sleep again; the proposal will be written then.

### 2026-11-12 · Scout's 12 November file: build the resume mode now (advisor)
The length lever is replicated (900-s2 vs 600-s2 +4.81 +/- 2.71, next to
long900 vs Control2 +5.65 +/- 2.73): by the plan of 8 November that
settles it, resumable training is worth building. Session: build it as
Scout lays out (trainer `resume` mode from the newest checkpoint with the
planned total, kb16.ps1 `-Resume`, a runner job that resumes its tag's
newest checkpoint), and put the lowlr-tail script in the same commit so
ONE approval carries both. Check the one open point first: whether
bullet's checkpoint carries the optimiser's running averages; if not, say
so in NETWORK.md and accept the re-warm. Test it in the cloud with a dry
run (plan 3 superbatches, stop after 1, resume to 3; compare the
end-of-run loss with an uninterrupted 3-superbatch run on the same data
and seed). When it is ready, write ONE "Decision needed: approve commit
<hash> (resume mode + lowlr-tail)" with the Approve-Scripts steps for
Ayman. Lane order after the PC is back: wdl25 (queued, needs nothing),
then 1350 (first resumable run), then equal-passes. The PC being silent
is Ayman's to fix; nothing to queue until the log moves.
Done (2026-11-12, Session): built as laid out. Trainer branch net-kb16-resume (6c4e8af396, a "part" mode and a "compare" mode); kb16.ps1 -Resume, -StopAt, -From. Open point: bullet's checkpoint DOES carry AdamW's running averages (momentum.bin, velocity.bin, restored by load_from_checkpoint): no re-warm. lowlr-tail needs no script of its own: it is a kb16.ps1 line with -From control2 (the cosine stays at its floor past the planned total). One change from the note: the dry run cannot train in the cloud (bullet at this commit has no CPU backend), so the script logic was tested here against a fake trainer and the real GPU comparison (uninterrupted twice vs stopped-and-resumed, 3 superbatches) is the first PC job after the approval. "Decision needed: approve" written in ESTIMATES.md with the Approve-Scripts steps. equal-passes still needs kb16.ps1 to take the more-data files.

### 2026-11-10 · CCRL: 1.0.0 is in Graham's queue (Ayman, 10 November, 2:52 pm Pacific)
Graham (CCRL) replied to Ayman: "Double Gauntlet in 16073". Trinity 1.0.0
is queued for a double gauntlet; the number is his batch or queue
reference, not yet confirmed. Session and check-in: from now on, check
the CCRL 40/15 list for "Trinity 1.0.0" once a week (lists update on
Saturdays) and, when a rating appears, log it in ESTIMATES.md as the
anchor (CLAUDE.md runner rule 13: gaps over 1.0.0 convert to the CCRL
scale from it) next to our own estimate for 1.0.0 (3466 by day, 3478 on
the runner, 10+0.1). Compare, and if the two differ by more than about
20, say so plainly and re-score the forecast. Rule 6 stands: no newer
version is offered to the testers until 1.0.0 is rated and Ayman decides;
nobody contacts CCRL except Ayman himself.
Done (2026-11-14, Session): first weekly check, run by a short session in the "trinity" environment (ccrl/2026-11-14.md). The list of 13 November has "Trinity 1.0.0 64-bit" at 3527 +/- 42 after 100 games (56.0%), not yet ranked (about rank 73-74 at that rating). Our estimate was 3466 by day and 3478 on the runner: CCRL is 49-61 higher, more than 20, said plainly in ESTIMATES.md, but within about 1.2 of CCRL's own 100-game error. Logged as the provisional anchor; the forecast is re-scored when the rating settles. Rule 6 and rule 8 noted for Ayman as "for later" in Decisions needed.
Not done yet (2026-11-11, Session): the weekly check starts Saturday 14 November (the pace routine, which runs in the Session). The CCRL site (computerchess.org.uk) cannot be reached from the cloud sessions today: the network proxy refuses it (403), and a web fetch failed too. A personal cloud environment "trinity" that allows that one site was created on 11 November (Ayman approved it on a card). The running Session cannot switch to it, so on Saturdays the pace routine starts a short session in that environment to read the CCRL 40/15 list; if that fails, a screenshot from Ayman or a check by the advisor. Our estimate for 1.0.0 to compare with: 3466 by day, 3478 on the runner (10+0.1).

### 2026-11-07 · Today's Scout file (Ayman, 7 November, 6:54 am: "tell session now")
Read scout/2026-11-07.md in full on today's run, not only "## For the
Session": its "Measured and dropped" section retires four search ideas
before any game (keep killers, no multi-cut, TT replacement variants, a
two-slot TT); Scout says the last measurement, finished after its push,
agrees. Don't queue those. The one new idea, [network] equal-passes data
test, depends on resumable training (Scout, 4 November): put a dated
status for resumable training under this note (built, planned, or not
started) so the plan for the next network run is visible. Scout's point 3
(Hash 16 MB in every test; CCRL plays with far more) is a check, not an Elo
test: note it in the Test queue as a calibration to run when the tester's
hash setting is known; no decision from Ayman needed.
Status of resumable training (2026-11-12, Session): built (trainer branch net-kb16-resume, kb16.ps1 -Resume/-StopAt/-From), waiting for Ayman's approval and a PC dry run.

Done (2026-11-07, Session): read the whole file; the five measured-and-dropped search ideas are logged in ESTIMATES.md ("Planned, not started") and will not be queued; the hash-size check is noted there as a calibration for when the tester's hash setting is known.
Resumable training, status 2026-11-07: NOT STARTED (planned). Proposed by Scout on 4 November; it needs a trainer change (save the weights, optimiser state and superbatch number every 25 superbatches; resume on the same schedule) plus kb16.ps1 support, which is under tools/ and so needs Ayman's approval. Order: after long900's re-tune and merge (one network experiment at a time), then lowlr-tail (about 50 minutes, no resume needed), then resumable training for long1350 or the equal-passes data test.

### 2026-11-06 · Ayman's decision: merge tm-floor2 (his word, 6 November, 7:23 am Pacific: "yes")
Answer to the routine's "merge tm-floor2 anyway?" (ESTIMATES.md, 6
November): YES, merge it. Grounds (advisor): two independent tests at two
clocks agree (+4.11 +/- 2.59 at 8+0.08, passed; +3.24 +/- 2.56 at 40/10,
LLR 2.94 against a 2.944 line), 0 time losses in 36000 games, time-only
change. Log it in ESTIMATES.md as a ONE-OFF EXCEPTION to the rule fixed
on 21 October, not as a pass. New standing rule from today (advisor with
Ayman): a test may be extended past its planned cap at most once, and a
candidate that misses the line after that extension is dropped; no
further exception. Merge commit: "Merge tm-floor2 (SPRT #91+#94 passed
+4.11; 40/10 #99+#104+#106 +3.24, LLR 2.94, merged by Ayman's decision of 6
November)". Then the usual bench check and push.

Done (2026-11-06, Session): merged (commit b379e0f, "Merge tm-floor2 (SPRT #91 + #94 passed at 8+0.08, +4.11; 40/10 ... +3.24, undecided at the 18000 cap)", merged by Ayman's decision); bench 982007 unchanged, tests pass. Logged in ESTIMATES.md as a one-off exception, not a pass; the extend-once rule is in the process rules.

### 2026-11-05 · Scout on Fable 5.1: a 2-week trial (Ayman, 5 November)
Ayman switched the Scout session to Fable 5.1 on 5 November. The trial runs
to 19 November. Measure: the share of Scout's ideas from 6 November on that
pass their test (bounds [0, 5]), against the baseline of about 1 in 4
(search candidates tested since 20 October: 5 passed of about 20). Session:
tag each new candidate's ESTIMATES.md row with "(Scout, Fable)" when the idea
came from a Fable-run scout file, so the count is mechanical; on 19 November
write the count under this note. Scout: your stored instructions still say
"keep the run short"; that line predates CLAUDE.md's usage rule of 20
October and no longer applies: do the full job. Favour ideas with a clear
mechanism and a cheap, decisive test.

Done (2026-11-06, Session): tagging started; no-checkext is the first row marked "(Scout, Fable)" (lowlr-tail is logged as planned, not yet tested). The count goes under this note on 19 November.
COUNT (2026-11-19, Session), Scout's ideas from the Fable-run files of 6-18 November that got a decided test, against bounds [0, 5]: search 0 of 3 (no-checkext -4.4, chk-noreduce -5.1, tm-nodefloor -2.8); network 2 of 3 (wdl25 x1.2 +8.9 vs Control2 and +7.6 vs main; wdl25-900 +5.5 vs main; lowlr-tail -2.7, failed). Total 2 of 6 (33%), against the baseline of about 1 in 4. Not counted: the wdl-length head-to-head (a diagnostic, stopped at -1.0) and wdl0 (screens start today). The two passes are the trial's main gain: wdl25-900 is the 1.2 network candidate. Small numbers: 2 of 6 is no clear evidence either way against 1 in 4.
Noted (2026-11-11, Session): with the search quota gone (Ayman, 11 November), the advisor proposes counting Fable's network ideas too, judged by their head-to-heads, rather than ending the trial early. The 19 November count will list search ideas (0 of 3 passed so far) and network ideas (wdl25, training tonight) separately.

### 2026-11-04 · Ayman's decisions (his words, 4 November, 6:48 pm Pacific)
1. long900: "My view, in advance: yes, merge it." This is Ayman's OK for
   CLAUDE.md rule 5. Merge long900 once its SPSA re-tune is done and the
   re-tuned version shows no loss (its own SPRT check as usual). If the
   re-tune or that check goes wrong, don't merge: put it to Ayman as a
   "Decision needed" with the numbers. Log the merge in ESTIMATES.md.
2. tm-floor2: "extend tm-floor2". Continue its 40/10 test from 12,000 to
   18,000 games (the usual cap), same base and bounds [0, 5]. Merge only
   if it passes there. Log honestly that this extension was decided after
   seeing the 12,000-game result (+3.82 +/- 3.14).

Done (2026-11-05, Session): (1) long900 passed vs main (+7.03); its SPSA re-tune started; it will be merged after the re-tune and its SPRT check, per this OK. (2) tm-floor2's 40/10 test continued from 12000 to 18000 games, same base and bounds; the post-hoc extension is logged in ESTIMATES.md.

### 2026-11-04 · Decision lines only when a decision is ready
The "Decisions needed" section is read word for word by the 7:43 phone
summary, so a line like "merge long900? Not yet" reaches Ayman as a
decision he can't make yet. Write a dated "Decision needed" line only when
everything it depends on is done and Ayman can answer yes or no; keep
"waiting for X" status in the Test queue instead. Move today's long900
line there until its checks are in.

Done (2026-11-05, Session): the long900 line moved out of "Decisions needed" into its plan entry; the section now says none are open.

### 2026-11-03 · Relay ended
Ayman no longer pastes messages between agents. Read this file and the
"## For the Session" section of the day's scout file on every pace-routine
run. Confirm here once the routine has done this for the first time.

Done (2026-11-04, Session): the pace routine read this file and scout/2026-11-04.md "## For the Session" on its 4 November run and acted on both.

### 2026-11-03 · Pace routine: name the shared files in its own steps
CLAUDE.md now says the routine reads this file and the scout file's
"## For the Session" section, but the routine's stored step 1 lists what
it reads and names neither. Add both to step 1 (you changed step 6 from
your own chat, so you can change step 1 the same way). Also note: the
phone summary moved to 7:43 am so it reports the routine's decisions the
same morning, not the next day.

Done (2026-11-04, Session): step 1 of the routine now names advisor-notes.md and the scout file's "## For the Session" section.

### 2026-11-03 · Forecast scoring, next review
At the next forecast review: (1) also score the pass probabilities (of the
tests given about 30% to pass, did about 30% pass?), since the queue
ordering by expected value depends on them; (2) re-score range coverage
after the 1.5x widening, aiming for 70-80% inside the range. If this was
already logged on 27 October, mark it done.

Not done yet (2026-11-04, Session): planned for the next forecast review; pass probabilities were scored once on 27 October (expected 8.3 passes, actual 5), range coverage not yet re-scored after the widening.

### 2026-11-03 · long900, if it passes vs Control2
It is the first clean one-change network result (training length only).
If it passes: calibration next to 1.0.0 the same night, the v2 check and
the SPSA re-tune, then "Decision needed: merge long900" to Ayman with those
numbers. It is not merged by the routine (CLAUDE.md rule 5).

Done in part (2026-11-04, Session): it passed (+5.65). Its test vs main runs on GitHub; calibration next to 1.0.0 and the v2 check are queued for tonight; the re-tune follows; then "Decision needed: merge long900" to Ayman.

## Done
