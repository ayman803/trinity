# Trinity

A UCI chess engine written from scratch in Rust, with an NNUE evaluation.
Its code was written by Claude (Anthropic's AI model); see
[Credits](#credits).

**Goal:** reach the top 100 of the [CCRL](https://computerchess.org.uk/ccrl/)
rating list (about 3455) by October 2027. Our own estimate in October 2026
is about 3380 (CCRL scale, 1 CPU); see [ESTIMATES.md](ESTIMATES.md) for every
measurement and prediction.

**For the person running the tests:** see **[GUIDE.md](GUIDE.md)**. It covers
setup, overnight SPRT tests and network training, step by step.

## Principles

- **Original code.** Ideas from the chess-programming community are fair
  game. Code from other engines is never copied.
- **Every change is tested.** Nothing that changes playing strength lands on
  `main` without passing an SPRT test (see `scripts/sprt.ps1`).
- **Bench fingerprint.** `trinity bench` prints a node count. A change that
  claims to be a pure speed-up must not change it.

## Features (phase 1)

**Board and move generation**

- Bitboards with magic-bitboard slider attacks. The tables are generated at
  build time in `build.rs`.
- Fully legal move generation using check masks and pin rays.
- A perft test suite of 21 positions, covering the standard ones plus
  en-passant, castling and promotion edge cases. The node counts are
  cross-checked against the python-chess library.

**Search** (`src/search.rs`)

- Iterative deepening with aspiration windows and principal variation search.
- Transposition table: lock-free and shared between threads.
- Quiescence search with SEE and delta pruning.
- Pruning: null-move pruning, reverse futility pruning, razoring, late-move
  pruning, futility pruning and SEE pruning.
- Late-move reductions; check, singular and multi-cut extensions; internal
  iterative reduction; mate-distance pruning.
- Move ordering: hash move, SEE-ranked captures with capture history,
  killers, counter-moves, butterfly history and 1/2-ply continuation
  history.
- Repetition, fifty-move and insufficient-material draw detection.
- Lazy SMP multi-threading and time management.

**Evaluation**

- NNUE (`src/nnue.rs`): a `(768 → 512)×2 → 1` network with SCReLU and 8
  output buckets by piece count, incrementally updated. Trained with bullet
  on our own self-play data, then on Leela-derived data (see Credits).
- A simple hand-crafted evaluation (`src/eval.rs`), used until the first
  network is trained and for the first round of self-play data.

**Endgame tablebases**

- Syzygy tablebases through the `shakmaty-syzygy` library (UCI option
  `SyzygyPath`): the root move comes from the DTZ tables, and the search
  uses WDL results after captures and pawn moves.

**Tooling**

- Self-play data generation in bullet's `ChessBoard` format
  (`trinity datagen`).
- The NNUE trainer, built on [bullet](https://github.com/jw1912/bullet) and
  pinned to a tested commit (`trainer/`).
- One-click Windows scripts for SPRT testing with fastchess, data generation
  and training (`scripts/`, `*.bat`).

## Building

```
cargo build --release
```

or, as testing frameworks such as OpenBench do:

```
make EXE=<name> [EVALFILE=<network file>]
```

The binary is `target/release/trinity` (`trinity.exe` on Windows). For the
fastest build on your own machine, set `RUSTFLAGS="-C target-cpu=native"`.

**Network embedding**

- If `nets/default.nnue` exists, it is compiled into the binary.
- `EVALFILE=<path>` embeds a different file instead.
- Without a network, the hand-crafted evaluation is used.
- The engine reports which evaluation it uses after `uci`.

**Command-line modes**

| Command                                                   | Purpose                                        |
|-----------------------------------------------------------|------------------------------------------------|
| `trinity`                                                 | UCI mode (what GUIDE and match runners use)    |
| `trinity bench [depth]`                                   | Fixed-depth search on 44 positions; prints `<nodes> nodes <nps> nps` |
| `trinity perft <depth> [fen]`                             | Move-generator node count                      |
| `trinity datagen threads=N positions=N nodes=N out=DIR`   | Self-play training data                        |

**UCI options**

- `Hash` (MB, default 16)
- `Threads` (default 1)
- `Move Overhead` (ms, default 10)
- `SyzygyPath` (folders with Syzygy files, separated by `;` on Windows and
  `:` elsewhere)

Extra commands for debugging: `d` (show the board), `eval`, `bench`, and
`go perft N`.

## Testing

```
cargo test --release     # perft suite, hashing, NNUE vs a float reference, data-format round trip
```

The CI (`.github/workflows/ci.yml`) runs the tests and bench on Linux and
Windows for every push, and stores the Windows `trinity.exe` as a download.

## Repository layout

```
src/            engine
  types.rs        squares, pieces, moves
  attacks.rs      attack tables (magic bitboards from build.rs)
  board.rs        position, make-move, SEE, FEN
  movegen.rs      legal move generation, perft (+ tests)
  search.rs       alpha-beta search, time management, Lazy SMP
  tt.rs           transposition table
  eval.rs         hand-crafted evaluation
  nnue.rs         neural network evaluation
  uci.rs          UCI protocol
  bench.rs        bench positions
  datagen.rs      self-play data generation
trainer/        NNUE trainer (bullet)
scripts/        PowerShell scripts behind the .bat files
nets/           the network compiled into the engine
```

## Roadmap

Phase 1 is done: correct move generation, alpha-beta search with standard
pruning, UCI, the NNUE pipeline and the testing infrastructure.

**Next steps**, each one SPRT-tested:

1. Train the first network from hand-crafted-eval self-play data, then
   iterate: new data from the better engine, retrain.
2. Search speed: staged move generation, a cheaper move picker, and lazy
   accumulator updates.
3. Grow the network: king-bucketed inputs with horizontal mirroring (in
   progress), then a bigger hidden layer.
4. Tune the search parameters. The pruning margins are currently reasonable
   defaults, not tuned values.
5. Better time management (node-based best-move stability) and SMP tuning.
6. More data: longer self-play runs, and optionally Leela's public training
   data.
7. CCRL readiness: portable release builds (x86-64-v2/v3), long stability
   runs, then submission to the testers.

## Credits

- **Code:** written by Claude, Anthropic's AI model, working in Claude Code.
  The project owner directs the work, runs every test on their own
  machines and decides what is merged. Trinity's code is original:
  ideas come from the open chess-programming community (the Chess
  Programming Wiki and the published ideas of engines such as Stockfish),
  but no code was copied from other engines.
- **Network trainer:** [bullet](https://github.com/jw1912/bullet) by Jamie
  Whiting and contributors (MIT licence), with the `bulletformat` crate.
- **Training data:** after a first stage on Trinity's own self-play games,
  the networks are refined on Leela-derived training data: positions from
  [Leela Chess Zero](https://lczero.org)'s self-play games, as published in
  Stockfish's binpack format by the Stockfish community for its own network
  training (read with the `sfbinpack` crate). Thank you to both projects
  for making this data public.
- **Tablebases:** the Syzygy tablebases by Ronald de Man, probed with
  [shakmaty-syzygy](https://github.com/niklasf/shakmaty-syzygy) by Niklas
  Fiekas (GPL-3.0).
- **Testing:** [fastchess](https://github.com/Disservin/fastchess) for
  matches and SPRT, Stefan Pohl's UHO opening books, the `8moves_v3` book,
  and [python-chess](https://github.com/niklasf/python-chess) to check the
  perft numbers. Ratings are measured against engines on the
  [CCRL](https://computerchess.org.uk/ccrl/) list.

## Licence

Trinity is free software under the GNU General Public License, version 3
or later (GPL-3.0-or-later); see [LICENSE](LICENSE).
