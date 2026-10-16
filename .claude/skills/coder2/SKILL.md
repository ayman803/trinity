---
name: coder2
description: Coder 2's working rules and pre-test checklist for Trinity's network lane (training runs, network checks, PC scripts in tools/net). Load at the start of any Coder 2 session and before every test request or PC-script push.
---

# Coder 2 (network lane) checklist

Read first: CLAUDE.md ("Coder 2"), NETWORK.md (rules at the top, then the
log), ESTIMATES.md ("Lessons so far", "Changes"), scout/INDEX.md.

## Files and branches
- Push to `main` only NETWORK.md and files under `tools/net/`. Engine code
  and `.nnue` files go on `net-...` branches; the Session merges them.
- Never edit ESTIMATES.md, CLAUDE.md or `scripts/`.
- Before every test request: merge `origin/main` into the branch, build,
  run `cargo test --release`, check `trinity bench`, then write the request
  in NETWORK.md ("Test requests", lane "network") with a prediction.
- `.claude/` was added with the owner's consent on 16 October 2026 (hook,
  this skill); touch nothing else there.

## PC scripts (tools/net)
- They write only inside `C:\Trinity\sprt\coder2\` and delete nothing;
  a script that must delete its own files says so at its top and the
  Session reviews it before Ayman runs it.
- Never use `$out`, `$root`, `$work`, `$exesuffix`, `$onwindows` as
  variable names: PowerShell ignores case and they overwrite the shared
  `$script:` values (this broke a training run on 15 October).
- Before pushing a script: `pwsh -NoProfile -File tools/net/Check-Scripts.ps1`
  (syntax and name clashes) and a dry run of the script with
  `pwsh -NoProfile -File tools/net/<script>.ps1` (it must stop with a
  clear "PROBLEM:" line when the PC's files are absent). The SessionStart
  hook installs `pwsh` in cloud sessions.
- Publish results to `main` only through `Publish-FileToBranch` in
  `net-common.ps1` (NETWORK.md and `tools/net/*` only).

## Predictions (write before the run or test, in NETWORK.md)
- Self-play Elo range, CCRL-scale range, chance to pass, and a stop-early
  threshold (for example "-10 or worse after 1000 games").
- Speed costs: measure nodes per second with several copies running at
  once (the PC runs 14 games) and convert on the log scale: the time-odds
  ruler is about 42 Elo per doubling of speed (+6.8 at +12%), so a 30%
  slowdown is about -22, not -18. Measure on the test machines when
  possible (ESTIMATES.md, nnue-simd lesson).
- Held-out loss ruler (NETWORK.md section 3): differences under 0.4% mean
  nothing; 1% of relative held-out loss is worth perhaps +5 to +15 Elo
  before any speed cost, not +15-20 (the 2048 network lost 30 with a
  1.3% better held-out loss and a 31% speed cost).
- Compare networks on held-out positions (Trinity's calibration games,
  result labels), never on training positions alone: a bigger network
  memorises. `tools/net/data/heldout-positions.txt` + `trinity netloss
  <n> <files> --positions=...` scores any network in the cloud.
- One change at a time per training run.

## History that bears on the network (as of 16 October 2026)
- Capacity transferred to rated engines (king buckets +17 CCRL, 1024 from
  scratch +50); refinement on more Leela data on a converged network gave
  about 0; weight averaging 0; width 2048 lost 30 (speed).
- Training speed on the PC: 1024 network 0.50 min per superbatch (5.0 h
  for 600); 2048: 1.2 min (10.7 h). Data: 32 GB per billion positions.
- Quantisation costs 0.006% of held-out loss: no re-rounding work needed.
