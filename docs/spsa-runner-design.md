# Re-tuning on the runner (design for review, 20 October 2026)

Owner's request (20 October, with the advisor): make re-tuning a recurring
runner job in idle hours, with Scout's rated-opponent version
(scout/2026-10-20.md) as the design to evaluate. Nothing is built yet; the
runner job needs Ayman's approval like every script.

## Scout's design

SPSA (scripts/spsa.py) on the runner, but each step plays the "up" and
"down" settings against the four calibration opponents, on the balanced
calibration book, at 10+0.1, Syzygy on; 8-10 settings; about 12,000 games
a night for 3 nights; then judge tuned vs main on the runner with 1.0.0
the same night (rule 13), plus one opponent the tuner never played. Pass:
gap over main at least +5 on two nights, held-out opponent not worse.

## Skeptic review: the information per game is about 4x lower

- In self-play, one game "up vs down" measures the difference directly.
  Against an opponent, the difference needs two games ("up vs X" and
  "down vs X"), and the variance of a difference of two game results is
  twice that of one. Per game played: about 1/4 of the information.
- 3 nights x 12,000 games = 36,000 games, worth about 9,000 self-play
  games. The 1024 re-tune used 35,280 self-play games for 20 settings
  and its moves were all small. With 8-10 settings and a quarter of the
  information, most of the movement would be noise.
- At 10+0.1 the games are twice as long as the GitHub tune's 5+0.05, so
  the 12,000 a night is optimistic for 4 cores (check: the 4000-game
  calibration takes 2.4 hours at 3-4 games at a time, so about 12,000 in
  7 hours; it fits only if the runner does nothing else that night).
- The judging rule is sound (Scout's pass rule plus a held-out opponent),
  and the external-validity argument is right: self-play tuning shrank
  3-4x on the calibration scale.

Advisor's correction (20 October): "the earlier tune didn't carry over"
rests on a single calibration, +3 with error bars of about +/- 14. That
is no evidence either way, so this is an experiment, not a known fix.

## Decision (Ayman with the advisor, 20 October): 4 settings

Chosen as the 4 that moved most in the two earlier tunes, in step sizes
(7 October first tune, then the 11 October 1024 re-tune); each moved the
same way both times:
QS_FUTILITY 150 -> 162 -> 171 (+1.0, +0.8 steps), LMP_BASE 300 -> 349
-> 359 (+1.2, +0.2), RAZOR_MUL 250 -> 266 -> 279 (+0.8, +0.7),
RFP_MARGIN 75 -> 81 -> 85 (+0.8, +0.5). (The Session's first guess,
FUT_BASE and LMR_QUIET_BASE, barely moved: 0.2 and 0 steps; replaced.)
21 October: razoring was removed (no-razor, +58), so RAZOR_MUL was
replaced by the fifth most moved, LMR_HIST_DIV (-0.3, -0.2 steps; RAZOR_BASE,
fourth, belongs to razoring too). Fresh chains C and D tune the new engine. 24 October: QS_FUTILITY (quiescence futility removed,
+60) replaced by FUT_MUL (futility widened, +5.8).
Conditions: (1) the unseen-opponent check decides; (2) calibration
nights for search/speed candidates that passed on GitHub come before
tuning nights; (3) experiment, not fix (above).

## Proposal (as reviewed)

1. Same idea, fewer settings: tune 4, not 8-10. The ones whose best value
   most plausibly differs against other engines (superseded by the
   decision above: the 4 that moved most in the earlier tunes). Fewer settings need fewer games for the same
   accuracy (the noise in each estimated direction grows with the number
   of settings moved at once).
2. Paired games: "up" and "down" play the same opening, same colour,
   against the same opponent, back to back. Outcomes on the same opening
   are correlated, which removes part of the extra noise.
3. Recurring job, state carried: each idle night continues from the last
   state (kept in trinity-results/spsa/state.json, read back by the next
   job), until 6 nights (about 70,000 games, about 18,000 self-play
   equivalents for 4 settings: comparable per setting to the 1024 tune).
4. Noise check at night 3: two independent chains (two halves of the
   games, own seeds). If they move fewer than 3 of the 4 settings in the
   same direction, stop: the signal is too weak at this budget.
5. Judge with Scout's rule after night 6.

Prediction (logged now, before anything is built): CCRL scale 0 to +6;
about 35% that it passes Scout's rule. Self-play may be 0 or negative.

Cost: 6 runner nights (calibrations of candidates move to idle day hours
or the other nights); one new runner job script (runner/jobs/spsa.ps1,
with a results-repository mode) and an `--opponents` mode in spsa.py;
one approval.

Built 20 October as a mode of the already-approved calibration script
(no runner reinstall): `scripts/calibrate.ps1 -Tune -Chain A|B -Minutes
270` (code in scripts/tune-opponents.ps1; PowerShell, since the PC has
no Python). Each step: "up" and "down" each play 2 x 14 games (both
colours, same openings) against one opponent, in turn; settings move by
the difference of their scores (the spsa.py schedule, 11,000 planned
pairs per chain). Progress saved after every step, so a disturbed job
loses one step; state and a summary go to trinity-results spsa/.
Dry run in the cloud (20 October, 1+0.01, a stand-in opponent): 6
steps over two jobs, resumed from the saved state, settings reach the
engine as UCI options. Queue lines, after Ayman's approval, on nights
without a candidate calibration:
`<id> | pc | scripts/calibrate.ps1 | -Tune -Chain A -Minutes 270` and the
same with `-Chain B` (from 21 October: chains C and D).
