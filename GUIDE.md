# Trinity: guide for your PC

This guide tells you exactly what to click and type. You never need to edit
code: Claude writes the code, and your PC does the testing and training.

**How we work together**

1. Claude makes an improvement and uploads it to GitHub as a separate
   *branch* (a named copy of the code, like `search-tweak-1`).
2. You run a test on your PC overnight: the new version plays thousands of
   fast games against the current version (`main`).
3. In the morning you paste the result to Claude. If it **PASSED**, the
   change is kept; if it **FAILED**, it is thrown away.

This kind of test is called **SPRT** (sequential probability ratio test). It
keeps playing until the statistics are clear, and then stops by itself. No
change is kept unless it is proven by games, because many ideas that
"obviously" should help actually make an engine weaker.

---

## 1. One-time setup (about 30–60 minutes)

### Step 1: Install Git

Git is the program that downloads the code from GitHub.

1. Go to <https://git-scm.com/download/win>. The download starts by itself.
2. Run the installer and click **Next** on every page (the default options
   are fine), then **Install** and **Finish**.

### Step 2: Download Trinity

1. Press the **Windows key**, type `cmd` and press **Enter**. A black window
   (the *Command Prompt*) opens.
2. Type this line exactly and press **Enter**:

   ```
   git clone https://github.com/ayman803/trinity C:\Trinity
   ```

3. Because the repository is private, a window will ask you to sign in to
   GitHub. Choose **Sign in with your browser** and approve it.
4. When it says `done`, close the black window.

You now have a folder `C:\Trinity`. Keep it there (the path must not
contain spaces).

### Step 3: Run the setup

1. Open **File Explorer** and go to `C:\Trinity`.
2. Double-click **`1-Setup.bat`**.
3. It installs:
   - Microsoft's **C++ Build Tools**, which Rust needs on Windows. Windows
     asks for permission: click **Yes**. A Visual Studio installer window
     may appear; let it finish. This is the slow part (5–15 minutes).
   - **Rust**, the programming language Trinity is written in.
4. Then it builds Trinity, runs its self-tests, and downloads the testing
   tools.
5. It is finished when you see **"Setup finished successfully."** Press any
   key to close the window.

If you see a red **PROBLEM:** line, copy the whole window's text (select it
with the mouse, then press Enter) and paste it to Claude.

---

## 2. Running a test (the regular routine)

1. Double-click **`2-Run-SPRT-Test.bat`**.
2. It shows a numbered list of versions, newest first. Type the number of
   the branch Claude told you about and press **Enter**. Just pressing
   **Enter** picks number 1, the newest.
3. It builds both versions and starts playing. Leave it running, overnight
   if needed. It can take anywhere from 20 minutes to 10+ hours.
4. When it ends, a box like this appears:

   ```
   SPRT RESULT: PASSED - 'search-tweak-1' is stronger than 'main'
   Games: 9432, Wins: ..., Losses: ..., Draws: ...
   Elo: 6.1 +/- 4.2, ...
   ```

   Select those lines, copy them, and paste them to Claude. They are also
   saved in `C:\Trinity\sprt\results\LATEST-RESULT.txt`.

**Things to know**

- **Threads.** The test uses 14 of your 20 CPU threads, so your PC stays
  usable for light things like browsing and email. Avoid games or other heavy
  programs during a test; they don't make it wrong, just slower and noisier.
- **Sleep.** While a test is running, your PC won't go to sleep by itself
  (the screen may still turn off). As soon as the test finishes, or you
  close the window, your normal sleep settings apply again. Nothing is
  changed in your Windows settings.
- **Stopping early.** Press **Ctrl+C** in the window, or just close it. You
  can restart the same test later; it starts over.
- **Different number of threads.** If Claude asks, open a Command Prompt in
  `C:\Trinity` and type, for example, `2-Run-SPRT-Test.bat -Concurrency 10`.

---

## 3. Training a neural network (NNUE)

A neural network evaluation is what makes modern engines strong. We train
our own on your RTX 3080. It's a three-step cycle: **generate data → train →
test**.

### One-time: install the NVIDIA CUDA Toolkit

1. Make sure your NVIDIA graphics driver is up to date (GeForce Experience,
   or <https://www.nvidia.com/drivers>).
2. Go to <https://developer.nvidia.com/cuda-downloads> and choose
   **Windows → x86_64 → 10 → exe (local)**. Download and run it with the
   default options (**Express**). It's a large download (about 3 GB).
3. **Restart your PC** afterwards.

### Step A: generate training data (one night)

Double-click **`3-Generate-Training-Data.bat`**. Trinity plays itself using 14
threads and saves positions into `C:\Trinity\data`. The window shows
progress and the estimated time left. The default is 30 million positions,
about one night. The files are about 1 GB per 30 million positions.

### Step B: train (about 15–60 minutes)

Double-click **`4-Train-Network.bat`**. It:

1. shuffles the data,
2. trains the network on your graphics card, and
3. uploads the finished network to GitHub as a new branch called
   `net-<date>-<time>`. GitHub may ask you to sign in again the first time.

### Step C: test it

Run **`2-Run-SPRT-Test.bat`** and choose that `net-...` branch. The first
network should beat the hand-written evaluation by a lot. Paste the result
to Claude, who will merge the network into `main` if it passed.

Later networks are trained the same way: more data plus a better previous
network gives a better next network.

### Step D: measure the real rating (after each new network)

Double-click **`5-Calibrate.bat`** when nothing else is running. Trinity
plays 1,000 games against each of three engines with a known CCRL rating
(akimbo 3474, Simbelmyne 3193, Inanis 3046). It takes about 2 hours and
ends with an estimated CCRL rating. Paste the result to Claude.
The first run also builds the three opponents from their source code
(a few minutes).

---

## 4. Where things are

| Folder / file            | What it is                                          |
|--------------------------|-----------------------------------------------------|
| `1-Setup.bat` … `5-…bat` | The five buttons you use                            |
| `sprt\results\`          | Test results and game records (PGN)                 |
| `data\`                  | Training data                                       |
| `checkpoints\`           | Networks produced by training                       |
| `src\`                   | The engine's source code (Claude's side)            |

You can delete `sprt\` or `checkpoints\` at any time to free disk space;
they are recreated when needed. `data\` holds the training data: only
delete it when Claude says so.

## 5. If something goes wrong

- **A red "PROBLEM:" line.** Copy the whole window text and paste it to
  Claude.
- **"The Trinity folder path contains a space".** Move the folder to
  `C:\Trinity`.
- **"CUDA Toolkit is not installed".** Install it (section 3) and restart the
  PC.
- **The window closes immediately.** Open a Command Prompt, type
  `cd C:\Trinity` and press Enter, then type the file name (for example
  `2-Run-SPRT-Test.bat`) and press Enter. The message will stay visible.
