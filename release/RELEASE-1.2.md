# Release 1.2: package plan and Ayman's steps

Written by the Session, 25 November 2026 (advisor's note of the same day;
release checklist item 4). Preparing is not offering: nothing goes to
CCRL until Ayman says so, and only Ayman writes to Graham.

Dates: freeze 1 December (Ayman's word that morning); checks done by
about 2 December; release published and Ayman's email about 3-8 December.

## What the package is

A GitHub release page "Trinity 1.2.0" (tag `v1.2.0`) with five files,
all built by GitHub from the frozen commit, each checked against the
frozen bench number before it is attached:

| File | For |
|---|---|
| `trinity-1.2.0-windows-x86-64-v3.exe` | Windows, processors with AVX2 (most since about 2013). The file a CCRL tester would use. |
| `trinity-1.2.0-windows-x86-64-v2.exe` | Windows, older processors |
| `trinity-1.2.0-linux-x86-64-v3` | Linux, AVX2 |
| `trinity-1.2.0-linux-x86-64-v2` | Linux, older processors |
| `trinity-1.2.0-linux-aarch64-musl` | 64-bit ARM Linux and Android (one static file) |

The network is inside each file; nothing else is needed. Settings a
tester sees: `Hash` (MB), `Threads`, `Move Overhead` (ms), `SyzygyPath`.

Which file: the AVX2 file (`x86-64-v3`) on any processor from about 2013
on. Measured on the PC on 25 November (one thread, Hash 256 MB): it
searches 2.18 million positions per second, the older-processor file
1.46 million, a third slower. The release page and README say so in one
plain line. CCRL's tester already runs the AVX2 file of 1.0.0
(ESTIMATES.md, Release 1.2, item 4b).

## What the Session does (no steps for Ayman)

At the freeze, on Ayman's "freeze":
1. Version number 1.2.0 in Cargo.toml; README tester text (the drafted
   edits of 18 November); record the frozen commit's full id and its
   bench number in ESTIMATES.md. Tag `v1.2.0` on that commit.
2. Robustness suite on the frozen build (release/README.md): UCI checks
   and the game checks against 1.0.0.
3. Queue on the PC, the same night: 40/60 calibration of the frozen
   build and of 1.0.0 with Hash 256 (five opponents), and the speed
   check (`scripts/speedcheck.ps1 -Release 1.0.0`).
4. Checks already run on today's main and repeated only if main changes
   before the freeze: source comparison and move similarity (ESTIMATES.md,
   "Release checks").
5. After Ayman's step B: read the build logs (each file's bench), and
   after step D the published page (five files, notes).

## Ayman's steps

Each step says what you will see when it worked. If the screen looks
different, send a screenshot instead of guessing.

### Before the freeze (any day up to 30 November)

**A1. Similarity test on the PC (about 20 minutes).** Open the folder
`C:\Trinity` and double-click `13-Similarity-Test.bat`. Leave the PC
alone while it runs. It ends with a table of percentages and the green
line "Done. Copy the table above ...". Send a screenshot of the table
(or paste it).

**A2. One reinstall and one approval** (for the speed check; the
Session gives the date and the exact lines to expect in ESTIMATES.md,
"Decisions needed").

### Release day (after the Session says "checks passed", about 2-3 December)

**B. Build the files (2 minutes of your time, then about 10 minutes of
waiting).**
1. In your browser open `https://github.com/ayman803/trinity/actions`.
2. In the list on the left, click **Release**.
3. On the right, click the grey button **Run workflow**. A small panel
   opens with three boxes.
4. Leave "Use workflow from" on **Branch: main**. Fill the boxes with
   the three values the Session gives you that day (copy and paste):
   - Version number: `1.2.0`
   - Full id of the commit: 40 letters and digits
   - Bench node count: a number of about 6 digits
5. Click the green **Run workflow** button in that panel.
6. After a few seconds a new line "Release" appears in the list with a
   yellow dot. When the dot turns into a green tick (about 10 minutes),
   it worked. A red cross means it stopped: tell the Session, nothing
   was published.

**C. Look at the draft.**
1. Open `https://github.com/ayman803/trinity/releases`.
2. At the top there is "Trinity 1.2.0" with a grey label **Draft**
   (only you can see it). Under "Assets" it lists the five files above.
3. Tell the Session "draft is up". The Session checks the build logs
   and answers "bench matches on all five" or says what is wrong.

**D. Try the Windows file on the laptop (the clean-machine test, 5
minutes).**
1. On the laptop, on that draft page, click
   `trinity-1.2.0-windows-x86-64-v3.exe` to download it. If the browser
   warns about the file, choose **Keep**.
2. Double-click the downloaded file. A black window opens and waits.
   (If Windows shows "Windows protected your PC", click **More info**,
   then **Run anyway**.)
3. Type `uci` and press Enter. Near the top you should see
   `id name Trinity 1.2.0`.
4. Type `bench` and press Enter. After a few seconds the last line
   shows a number followed by "nodes": it must be the bench number the
   Session gave you.
5. Type `quit` and press Enter. Send a screenshot of the window before
   you close it.

**E. Publish (only when C and D are fine and you want it public).**
1. On the draft page click the pencil icon (**Edit**) at the top right
   of the draft.
2. Scroll to the bottom and click the green **Publish release** button.
3. The "Draft" label is gone and the page has the address
   `https://github.com/ayman803/trinity/releases/tag/v1.2.0`. Tell the
   Session "published"; it checks the page and queues the speed check
   of the published files.

**F. Your email to Graham (yours alone).** The Session will give you
the facts to draw on, in a short list: the release link, what changed
since 1.0.0 in two lines, which file to use, the settings, and our own
measurement next to CCRL's rating of 1.0.0. You decide what to write
and when.

## If something merges before the freeze

Only the checks of "What the Session does", point 4, are repeated (about
an hour of cloud time, plus A1 again). Nothing else changes.
