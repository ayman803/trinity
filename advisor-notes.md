# Advisor notes

Notes from the advisor (Ayman's main contact) for the Session and Scout.
Rules: CLAUDE.md, "No relay". The pace routine reads the open notes every
morning, acts on them within the rules, and marks each one under it with a
dated "Done: ..." or "Not done: <reason>". Only the advisor adds notes.
Decisions reserved for Ayman still go to him as "Decision needed".

## Open

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
