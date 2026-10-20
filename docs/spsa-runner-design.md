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

## Proposal

1. Same idea, fewer settings: tune 4, not 8-10. The ones whose best value
   most plausibly differs against other engines are the pruning margins
   that decide what Trinity refuses to look at: RFP_MARGIN, FUT_BASE,
   LMP_BASE, LMR_QUIET_BASE. Fewer settings need fewer games for the same
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

Decision for Ayman and the advisor: build the 4-setting version (above),
Scout's 8-10-setting version, or neither.
