# PC job runner: design (PROPOSAL, not approved, nothing built)

Status: design only, written 17 October 2026 at the owner's request.
No code, no CLAUDE.md change, nothing on the PC until Ayman approves.

## Why

The PC was busy about 40% of this week. It waits on a human loop: script,
double-click, wait, copy, paste. A runner on the PC takes the next job
from a queue on GitHub, runs it and saves the result back, so jobs
follow each other day and night. Expected: 40% busy -> 80-90%.

## What the Session can and cannot reach

The Session never gets a connection into the PC. The runner only *pulls*:
it reads the queue from GitHub and pushes results. The Session's only
lever is what it commits to `main`, and every lever below limits that.

## Safety rules (to go into CLAUDE.md before any code)

1. **Separate Windows account.** A standard (not administrator) local
   account `trinity-runner`. It may modify `C:\Trinity` and its own
   profile only. Other users' folders (Ayman's documents, browser
   passwords, photos) are closed to it by Windows itself.
2. **Approval list outside GitHub.** `C:\TrinityRunner\approved.txt`,
   writable only by Ayman, readable by the runner. Each line names a
   script, its SHA-256 fingerprint and its maximum run time. The runner
   checks the fingerprint before every job: if the Session changes an
   approved script, it stops running until Ayman approves the new
   version (one double-click on `Approve-Script.bat`, which shows what
   changed). Approving by name alone would not be enough, because the
   Session can rewrite a script under the same name.
3. **Narrow GitHub key.** A fine-grained token for `ayman803/trinity`
   only, permission "Contents: read and write", stored in the runner
   account's Windows Credential Manager (never in the repository).
   Two-factor sign-in on Ayman's GitHub account.
4. **Hard limits in the runner.** One job at a time; the time limit from
   approved.txt (the job is stopped when it runs over); no job starts
   below 50 GB free, and a running job is stopped below 30 GB; arguments
   from the queue must match a simple pattern (letters, digits, `-`,
   `_`, `.`, `+`, `/`) and are passed as a list, never through `cmd`.
5. **No deletes outside its own folders.** The runner itself deletes
   nothing. Job scripts keep their existing rules (only their own
   temporary files; never `C:\Trinity\checkpoints\*big1024*`).
6. **Stop switch.** If the file `C:\TrinityRunner\STOP` exists, the
   runner starts nothing new. `Stop-Runner.bat` on Ayman's desktop
   creates it; `Start-Runner.bat` removes it. A window or log file shows
   the current job.
7. **Everything logged.** Every job: queue line, script fingerprint,
   start, end, result or error, pushed to `runner/log/YYYY-MM-DD.txt`.
8. **Ayman's own double-clicks** still work: the runner waits while any
   Trinity window Ayman started is running (a lock file).

## Honest limits of this design

- Job scripts build Trinity from GitHub branches, and building runs code
  from the branch (`build.rs`) and then the engine itself. Fingerprints
  cover the scripts, not every branch's engine code. That code is
  contained by rule 1 (the account), which is why rule 1 matters most.
- The token can write to `main` (fine-grained tokens cannot be limited
  to one branch without branch protection, which would also block the
  Session). A misbehaving job could push to GitHub, but cannot run a
  changed script on the PC because of rule 2.
- Rust is installed per user, so the runner account needs its own Rust
  install; CUDA and git are shared. The GPU must work for a background
  account: this is tested first (phase 3), before anything else depends
  on it.
- Some scripts ask questions (Read-Host). The runner runs them
  non-interactively, so such a script fails cleanly instead of hanging;
  those prompts get an unattended default before the script is approved.

## Phases (each needs Ayman's OK)

0. Rules 1-8 into CLAUDE.md.
1. Account and folders: click-by-click steps for Ayman (about 20
   minutes): create `trinity-runner`, set folder permissions, create
   `C:\TrinityRunner`, the token, Rust for the new account.
2. The runner script (`runner/runner.ps1`, about 200 lines), reviewed by
   the advisor before approval.
3. Test with harmless jobs only: `bench`, then a 10-game match, then a
   GPU check (trainer `--help` and one training superbatch).
4. Real queue: calibrations and SPRT-style PC jobs first; training runs
   only after a week without incident.

Cost: about 2-3 Session turns to build, 20-30 minutes of Ayman's time
for phase 1. Benefit: roughly doubles useful PC hours.
