# Advisor notes

Notes from the advisor (Ayman's main contact) for the Session and Scout.
Rules: CLAUDE.md, "No relay". The pace routine reads the open notes every
morning, acts on them within the rules, and marks each one under it with a
dated "Done: ..." or "Not done: <reason>". Only the advisor adds notes.
Decisions reserved for Ayman still go to him as "Decision needed".

## Open

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
