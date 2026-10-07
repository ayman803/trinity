"""SPSA tuning of Trinity's search settings (used by .github/workflows/spsa.yml).

`chain`: one tuning chain on one machine. Each step nudges every setting
up or down at random (by about its step size), plays a few game pairs
between the "up" and the "down" versions with fastchess, and moves each
setting towards the side that scored better. Step sizes shrink as the
tuning goes on (the usual SPSA schedule, as on fishtest:
alpha 0.602, gamma 0.101, A = 10% of the planned pairs, r_end 0.002).

`combine`: average the chains of one run into the state the next run
continues from, and print the values.

The settings, their defaults, limits and step sizes come from src/tune.rs.

Usage:
  spsa.py chain --engine E --fastchess F --book B --minutes M --seed S
                --total-pairs N [--state previous.json] --out chain.json
  spsa.py combine --out state.json chain1.json chain2.json ...
"""

import argparse
import json
import random
import re
import subprocess
import time
from pathlib import Path

ALPHA, GAMMA, R_END = 0.602, 0.101, 0.002
TC = "5+0.05"
PAIRS_PER_STEP = 6
CONCURRENCY = 3


def settings():
    src = (Path(__file__).resolve().parent.parent / "src" / "tune.rs").read_text()
    out = {}
    for m in re.finditer(r"^\s*([A-Z_]+) = (-?\d+), (-?\d+), (-?\d+), ([\d.]+);", src, re.M):
        name, default, lo, hi, step = m.groups()
        out[name] = {"default": int(default), "min": int(lo), "max": int(hi), "step": float(step)}
    if not out:
        raise SystemExit("no settings found in src/tune.rs")
    return out


def play(args, plus, minus, seed):
    """Wins and losses of the "plus" engine, and games lost on time."""

    def engine(name, values):
        return ["-engine", f"cmd={args.engine}", f"name={name}"] + [f"option.{k}={v}" for k, v in values.items()]

    cmd = (
        [args.fastchess]
        + engine("plus", plus)
        + engine("minus", minus)
        + ["-each", "proto=uci", f"tc={TC}", "option.Hash=16", "option.Threads=1"]
        + ["-openings", f"file={args.book}", "format=epd", "order=random", "-srand", str(seed)]
        + ["-repeat", "-games", "2", "-rounds", str(PAIRS_PER_STEP), "-concurrency", str(CONCURRENCY)]
        + ["-draw", "movenumber=40", "movecount=8", "score=10"]
        + ["-resign", "movecount=3", "score=600", "twosided=true"]
    )
    text = subprocess.run(cmd, capture_output=True, text=True).stdout
    m = re.findall(r"Games: (\d+), Wins: (\d+), Losses: (\d+), Draws: (\d+)", text)
    if not m:
        raise RuntimeError("fastchess gave no result:\n" + text[-2000:])
    games, wins, losses, _ = (int(x) for x in m[-1])
    on_time = len(re.findall(r"^Finished game.*loses on time", text, re.M))
    return wins, losses, games, on_time


def chain(args):
    params = settings()
    state = json.loads(Path(args.state).read_text()) if args.state else {}
    theta = {k: float(state.get("theta", {}).get(k, p["default"])) for k, p in params.items()}
    k = int(state.get("pairs", 0))
    games = time_losses = 0
    n = args.total_pairs
    big_a = 0.1 * n
    rng = random.Random(args.seed)
    deadline = time.time() + 60 * args.minutes
    step_seconds = 0.0
    while time.time() + 1.5 * step_seconds < deadline:
        started = time.time()
        kk = k + 1
        plus, minus, flips, c_k = {}, {}, {}, {}
        for name, p in params.items():
            c_k[name] = p["step"] * (n / kk) ** GAMMA
            flips[name] = rng.choice((-1, 1))
            up = theta[name] + c_k[name] * flips[name]
            down = theta[name] - c_k[name] * flips[name]
            plus[name] = round(min(max(up, p["min"]), p["max"]))
            minus[name] = round(min(max(down, p["min"]), p["max"]))
        wins, losses, played, on_time = play(args, plus, minus, args.seed * 1_000_003 + kk)
        result = wins - losses
        for name, p in params.items():
            a_end = R_END * p["step"] ** 2
            a_k = a_end * (big_a + n) ** ALPHA / (big_a + kk) ** ALPHA
            theta[name] += a_k * result * flips[name] / c_k[name]
            theta[name] = min(max(theta[name], p["min"]), p["max"])
        k += played // 2
        games += played
        time_losses += on_time
        step_seconds = time.time() - started
        print(f"pairs {k}: plus {wins}-{losses}  " + " ".join(f"{n_}={v:.1f}" for n_, v in theta.items()), flush=True)
    Path(args.out).write_text(json.dumps({"pairs": k, "theta": theta, "games": games, "time_losses": time_losses}))


def combine(args):
    params = settings()
    chains = [json.loads(Path(p).read_text()) for p in args.chains]
    if not chains:
        raise SystemExit("no chains finished")
    theta = {name: sum(c["theta"][name] for c in chains) / len(chains) for name in params}
    pairs = round(sum(c["pairs"] for c in chains) / len(chains))
    games = sum(c["games"] for c in chains)
    time_losses = sum(c.get("time_losses", 0) for c in chains)
    Path(args.out).write_text(json.dumps({"pairs": pairs, "theta": theta}))
    lines = [
        f"SPSA RESULT: {len(chains)} chains, {games} games this run ({TC}, {CONCURRENCY} at a time), "
        f"{pairs} pairs per chain so far",
        f"Games lost on time (either side): {time_losses} ({100 * time_losses / max(games, 1):.2f}% of games)",
        "setting: default -> tuned",
    ]
    lines += [f"{name}: {p['default']} -> {round(theta[name])}" for name, p in params.items()]
    print("\n".join(lines))
    Path("result.txt").write_text("\n".join(lines) + "\n")


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("chain")
    for flag in ("--engine", "--fastchess", "--book", "--out"):
        c.add_argument(flag, required=True)
    c.add_argument("--state")
    c.add_argument("--minutes", type=float, required=True)
    c.add_argument("--seed", type=int, required=True)
    c.add_argument("--total-pairs", type=int, required=True)
    m = sub.add_parser("combine")
    m.add_argument("--out", required=True)
    m.add_argument("chains", nargs="*")
    args = ap.parse_args()
    chain(args) if args.cmd == "chain" else combine(args)


if __name__ == "__main__":
    main()
