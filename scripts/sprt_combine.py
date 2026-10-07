"""Combine the self-play results of several GitHub Actions jobs into one
SPRT result (used by .github/workflows/sprt.yml).

Each job leaves a small text file with fastchess's last "Games:" and
"Ptnml(0-2):" lines and the two bench numbers. The totals are added up
(also to the totals of an earlier run, if one is given) and judged with the
same test fastchess uses on the PC: pentanomial, "normalized" Elo model,
bounds [elo0, elo1], alpha = beta = 0.05. Only finished game pairs count:
a pair cut off by the time limit is left out, as in fastchess's own
pentanomial totals. The LLR is the usual normal
approximation of fastchess's value (within about 0.05 of it).

Usage: python3 sprt_combine.py <results dir> <dev> <base> <tc> <elo0> <elo1> <jobs>
"""

import glob
import math
import os
import re
import sys

ALPHA = BETA = 0.05


def parse(path):
    text = open(path, encoding="utf-8", errors="replace").read()
    out = {}
    m = re.findall(r"Games: (\d+), Wins: (\d+), Losses: (\d+), Draws: (\d+)", text)
    if m:
        out["wld"] = [int(x) for x in m[-1][1:]]
    m = re.findall(r"Ptnml\(0-2\): \[(\d+), (\d+), (\d+), (\d+), (\d+)\]", text)
    if m:
        out["ptnml"] = [int(x) for x in m[-1]]
    m = re.search(r"^tc=(\S+)", text, re.M)
    if m:
        out["tc"] = m.group(1)
    for key in ("dev_bench", "base_bench", "time_losses"):
        m = re.search(rf"^{key}=(\d+)", text, re.M)
        if m:
            out[key] = int(m.group(1))
    return out


def elo(score):
    score = min(max(score, 1e-6), 1 - 1e-6)
    return -400 * math.log10(1 / score - 1)


def judge(ptnml, elo0, elo1):
    pairs = sum(ptnml)
    xs = [0, 0.25, 0.5, 0.75, 1]
    mean = sum(x * c for x, c in zip(xs, ptnml)) / pairs
    var = sum(c * (x - mean) ** 2 for x, c in zip(xs, ptnml)) / pairs
    var = max(var, 1e-9)
    k = 800 / math.log(10)
    # Per-game t-value: pairs carry two games.
    t = (mean - 0.5) / math.sqrt(var) / math.sqrt(2)
    t0, t1 = elo0 / k, elo1 / k
    llr = pairs * (t1 - t0) * (2 * t - t0 - t1)
    se = math.sqrt(var / pairs)
    lo, hi = elo(mean - 1.96 * se), elo(mean + 1.96 * se)
    return {
        "elo": elo(mean),
        "elo_err": (hi - lo) / 2,
        "nelo": k * t,
        "nelo_err": 1.96 * k / math.sqrt(2 * pairs),
        "llr": llr,
        "score": mean,
    }


def main():
    folder, dev, base, tc = sys.argv[1:5]
    elo0, elo1 = float(sys.argv[5]), float(sys.argv[6])
    jobs = int(sys.argv[7])
    ptnml, wld, benches, done, time_losses = [0] * 5, [0] * 3, set(), 0, 0
    earlier = None
    for path in sorted(glob.glob(os.path.join(folder, "**", "*.txt"), recursive=True)):
        r = parse(path)
        if "ptnml" not in r:
            continue
        if os.path.basename(path) == "total.txt":
            earlier = r
        else:
            done += 1
        ptnml = [a + b for a, b in zip(ptnml, r["ptnml"])]
        wld = [a + b for a, b in zip(wld, r.get("wld", [0, 0, 0]))]
        time_losses += r.get("time_losses", 0)
        benches.add((r.get("dev_bench"), r.get("base_bench")))
        if "tc" in r and r["tc"] != tc:
            sys.exit(f"The earlier run used time control {r['tc']}, this one {tc}; not combining.")
    if sum(ptnml) == 0:
        sys.exit("No results found: did every job fail? Open a job to see why.")
    if len(benches) > 1:
        sys.exit(f"The jobs tested different versions (bench numbers {sorted(benches)}); not combining.")
    dev_bench, base_bench = benches.pop()
    j = judge(ptnml, elo0, elo1)
    lower, upper = math.log(BETA / (1 - ALPHA)), math.log((1 - BETA) / ALPHA)
    games = 2 * sum(ptnml)
    if j["llr"] >= upper:
        verdict = f"PASSED - '{dev}' is stronger than '{base}'"
    elif j["llr"] <= lower:
        verdict = f"FAILED - '{dev}' is not stronger than '{base}'"
    else:
        verdict = "NOT DECIDED YET - run it again with 'Continue from run' to add games"
    lines = [
        f"SPRT RESULT: {verdict}",
        f"Where: GitHub Actions, {done} of {jobs} jobs finished" + (" (plus an earlier run)" if earlier else ""),
        f"Test: {dev} (bench {dev_bench}) vs {base} (bench {base_bench}), tc {tc}, bounds [{elo0:g}, {elo1:g}]",
        f"Games: {games}, Wins: {wld[0]}, Losses: {wld[1]}, Draws: {wld[2]}, "
        f"Points: {wld[0] + wld[2] / 2:.1f} ({100 * j['score']:.2f} %)",
        f"Ptnml(0-2): {ptnml}",
        f"Games lost on time (either side): {time_losses}",
        f"Elo: {j['elo']:.2f} +/- {j['elo_err']:.2f}, nElo: {j['nelo']:.2f} +/- {j['nelo_err']:.2f}",
        f"LLR: {j['llr']:.2f} ({lower:.2f}, {upper:.2f}) [{elo0:.2f}, {elo1:.2f}]",
    ]
    print("\n".join(lines))
    with open("total.txt", "w", encoding="utf-8") as f:
        f.write(f"Games: {games}, Wins: {wld[0]}, Losses: {wld[1]}, Draws: {wld[2]}\n")
        f.write("Ptnml(0-2): [" + ", ".join(map(str, ptnml)) + "]\n")
        f.write(f"dev_bench={dev_bench}\nbase_bench={base_bench}\ntime_losses={time_losses}\ntc={tc}\n")
    with open("result.txt", "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
