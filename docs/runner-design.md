# PC job runner

Status (17 October 2026): rules approved and in CLAUDE.md ("PC job
runner"), which wins where this file differs. Code written (runner/),
not installed. Phase 1 needs Ayman at the PC.

## Why

The PC was busy about 40% of the week, waiting on a human loop (script,
double-click, copy, paste). The runner takes the next job from
`runner/queue.txt` and runs it unattended, day and night.

## How it works

- `C:\TrinityRunner\runner.ps1` (installed copy, read-only for the runner
  account) starts at boot as the standard account `trinity-runner`.
- Every 5 minutes: STOP switch? restart pending? under 50 GB free? another
  Trinity program running? If none, it fetches GitHub into its own clone
  `C:\TrinityRunner\repo`, reads the queue from the latest `main`, and
  takes the first job for this machine that has not run.
- It checks out exactly the commit in `C:\TrinityRunner\approved.txt` and
  verifies the fingerprint of everything under `scripts/`, `tools/` and
  `runner/jobs/` at that commit. It runs the job's `.ps1` directly
  (not the .bat) with `-NoUpdate` and `TRINITY_NO_UPDATE=1`, so the code
  cannot change mid-job, and stops it at its time limit or below 30 GB.
- Results: the scripts' publish functions see `TRINITY_RESULTS_DIR` and
  write into a clone of `ayman803/trinity-results`; the runner logs every
  job there (`runner/log/<machine>.txt`, `runner/output/<id>.txt`).
- Cleared scripts (with a results mode): runner/jobs/hello.ps1,
  calibrate, timeuse, forensics, v2check, threads, quantcheck, netloss.
  Training scripts are not cleared yet (they write into the read-only data
  folders); that comes after a clean week, with its own write area.
- `Approve-Scripts` (Ayman's desktop) shows what changed in the scripts
  since the last approval and approves the latest commit with Y.

## What protects Ayman's machine

1. The account: standard user, `C:\Trinity` read-only (installer removes
   the default write access every signed-in account has on folders under
   `C:\` and adds a deny-write rule for `trinity-runner`). It cannot plant
   code where Ayman double-clicks, nor touch checkpoints or data.
2. The pinned, fingerprinted commit: the Session can queue jobs, but a
   changed script runs only after Ayman approves it.
3. The key: writes only to `trinity-results`, so no job can push code to
   `trinity` that a later job would build.
4. Limits: one job at a time, time limits, disk floors, argument pattern,
   STOP switch, no sleep, no automatic update restarts while Ayman is
   signed in, nothing starts while a restart is pending.

Residual risks, stated plainly: engine code built from branches runs
under the runner account (contained by point 1, not prevented); the
GitHub key can change `trinity-results`; Windows may still restart for an
update if nobody is signed in.

## Phase 1: Ayman's steps (about 20 minutes, once per machine)

1. GitHub: create a new public repository `trinity-results` with a README
   (button "New" on github.com, tick "Add a README file").
2. GitHub: Settings > Developer settings > Personal access tokens >
   Fine-grained tokens > Generate new token. Name `trinity-runner`;
   Repository access: Only select repositories > `trinity-results`;
   Permissions > Repository permissions > Contents: Read and write.
   Generate, and copy the key (it is shown once).
3. Check that two-factor sign-in is on for the GitHub account.
4. PC: double-click `C:\Trinity\runner\Install-Runner.bat`, answer Yes to
   the administrator question, type `pc`, paste the key.
5. Double-click "Trinity Approve-Scripts" on the desktop and type Y.

## Phase 3: tests (harmless first)

1. `runner/jobs/hello.ps1`: account name, tools, GPU visible, and a check
   that C:\Trinity is NOT writable for the runner.
2. A short real job (time use), then a calibration.
3. Training: only after a week without incident, with a write area
   designed for it.
