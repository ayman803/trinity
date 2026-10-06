# Trinity

A UCI chess engine in Rust with an NNUE evaluation.

I want to build a better mousetrap. I'm not a programmer, so Claude writes the code and I call the shots. I'd rather try an original idea and fail than to have no original ideas at all.

Claude (Anthropic's AI) writes the code in Claude Code. I run every test
on my own two machines and decide what gets merged. No code is copied from
other engines.

Strength: about 3380 on the CCRL scale (1 CPU), measured against
akimbo, Patricia, Bread and Prune at 10+0.1. The goal is the CCRL top 100
(about 3455) by October 2027. Every prediction and result is logged in
[ESTIMATES.md](ESTIMATES.md), including the ones that went wrong.

## Building

```
cargo build --release
```

or `make EXE=<name> [EVALFILE=<net>]`. The network in `nets/default.nnue`
is built into the binary. `trinity bench` prints `<nodes> nodes <nps> nps`.

UCI options: `Hash`, `Threads`, `Move Overhead`, `SyzygyPath`.

## What's inside

- Bitboards, legal move generation, perft-tested.
- Alpha-beta (PVS) with the usual pruning, reductions and extensions,
  history-based move ordering, correction history, Lazy SMP.
- NNUE: (768x8 king buckets, mirrored -> 512)x2 -> 8 output buckets,
  SCReLU.
  Trained first on Trinity's own games, then on Leela-derived data.
- Syzygy tablebases.

The Windows `.bat` files and `scripts/` are what I use to generate data,
train networks and run SPRT tests. [GUIDE.md](GUIDE.md) explains them.

## Credits

- Code: Claude (Anthropic), see above.
- Trainer: [bullet](https://github.com/jw1912/bullet) by Jamie Whiting and
  contributors.
- Training data: Leela Chess Zero self-play positions, in the binpack form
  the Stockfish community published for its own training. Thanks to both.
- Tablebases: Syzygy by Ronald de Man, read with
  [shakmaty-syzygy](https://github.com/niklasf/shakmaty-syzygy) by Niklas
  Fiekas.
- Testing: [fastchess](https://github.com/Disservin/fastchess), Stefan
  Pohl's UHO books, python-chess for perft checks, and the engines on the
  [CCRL](https://computerchess.org.uk/ccrl/) list.
- Ideas: the Chess Programming Wiki and the open engine community.

## Licence

GPL-3.0-or-later, see [LICENSE](LICENSE).
