# Scout ideas index

One line per idea ever proposed: date | short name | status.

2026-10-12 | Bucket spread as eval-uncertainty signal for LMR | proposed
2026-10-12 | Weight averaging of late training checkpoints | tested 13 Oct: ~0 (SPRT #11 +0.4, #12 -1.3), dropped
2026-10-12 | Loss forensics (turning points, 10x-time re-search, Pareto) | done 13 Oct: ~60% of mistakes fixed by more time (ESTIMATES.md)
2026-10-13 | Learned time allocation from forensics labels (value of information) | proposed
2026-10-13 | Ensemble across threads: helpers use an equal-strength averaged network | proposed
2026-10-13 | Closed-loop clock: feedback controller tracking a planned remaining-time curve | proposed
2026-10-13 | Ablation audit of pruning rules on forensics mistake positions | proposed
2026-10-13 | Mirror twin: average eval of position and its left-right mirror at PV nodes | checked 14 Oct: the network already mirrors each side by king file, so both evals are identical; dropped
2026-10-13 | On-policy fine-tuning: train on positions from Trinity's own games, labelled by its deeper search | proposed
2026-10-13 | Winner's-curse discount at crowded nodes (max-of-noisy-estimates bias) | proposed
2026-10-14 | Finding: SPRT #16 nnue-simd commit has no AVX2 code (test-only change); re-test needed | proposed
2026-10-14 | Identical-binary guard in the SPRT workflow (checksum both engines before playing) | proposed
2026-10-14 | Time-odds ruler: price speed changes from measured Elo per extra time (tm-control +6.8 at 1.12x) | proposed
2026-10-14 | Equal-marginal-value test: extra time in one move band at a time | proposed
2026-10-14 | Rounding-aware quantisation of the network (match float output) | proposed
2026-10-15 | [network] Loss-to-Elo exchange rate from held-out loss of networks with known Elo | proposed
2026-10-15 | [network] Width-scaled learning rate before judging the 2048 network | proposed
2026-10-16 | [network] Finding: the 2048 integer network that lost (-29.8) was never scored; check quantisation, overflow and incremental eval at 2048 | proposed
2026-10-16 | [network] First-article check: 200-400 games before a full network SPRT | proposed
2026-10-18 | [search] Interleaved, opening-paired calibration of candidate and 1.0.0 within one night (A-vs-A check first) | proposed
2026-10-19 | [network] Two seeds per recipe for verdicts; best of 3-4 seeds of the current recipe as a cheap gain (winner's curse) | proposed
2026-10-20 | [search] SPSA on the runner against the rated opponents (calibration book, 10+0.1), held-out opponent as check | proposed
2026-10-21 | [search] Factorial split of #38: test "score quiets lazily" and "drop quiets once one is pruned" separately vs the old main | proposed
2026-10-22 | [search] Audit sampling: re-search 1 in 200 pruned nodes to measure each rule's error rate and nodes saved (pointer: razoring lacked RFP's mate/TB guard on the TT-adjusted eval) | proposed
2026-10-23 | [search] Measure eval error by side (behind vs ahead) and set the remaining "behind" pruning margins from it (razor/qsfut/fut-wide vs rfp-wide pattern) | proposed
2026-10-26 | [search] Smaller late-move reduction when static eval is far below alpha (eval-bias table: errors large when behind) | proposed
2026-10-27 | [search] Re-screen parked ideas on today's engine (hist-prune, nonpawn-corr first): context changed by the removals and #38 | proposed
2026-10-28 | [search] Follow #38's mechanism (drop-half +18.6): lmp-tight, and drop-keep-checks as a diagnostic | proposed
2026-10-31 | [search] Matchup forensics: Prune rates Trinity 10-40 lower than the other opponents in every run; compare loss profiles to find what it exploits | proposed
2026-11-02 | [search] corr-integral: correction tables settle at 2/3 of the eval gap (they chase the leftover and decay); additive update to reach the full gap, plus an unsure-threshold variant to split corr-avg's two causes | proposed
2026-11-03 | [search] corr-cap: count correction entries at the 32 cp cap before judging corr-integral; cap 64 only if >5% sit there (finding: tm-floor2 owes its pre-fixed 40/10 test before merge) | proposed
2026-11-03 | [network] Sibling-ranking check: score networks by how they order the moves of one position vs a deep search (top-1, Kendall's tau), validated on networks with known game results | proposed
2026-11-04 | [network] Resumable training (save every 25 superbatches with optimiser state and step; resume on the same schedule) so long1350 can run across two nights; plumbing check first (for the Session: long900 still needs its SPRT vs main) | proposed
2026-11-06 | [search] no-checkext: the unconditional check extension (first commit, never tested alone, not in the campaign list); measured: 26% fewer nodes per depth on game positions, one ply deeper at 300 ms; test [-4, 1] | proposed
2026-11-06 | [network] lowlr-tail: continue Control2's 600 checkpoint 100 superbatches at the floor rate (no high-rate restart) to tell whether long900's gain was more passes or more low-rate steps; about 1 GPU hour | proposed
2026-11-07 | [network] Equal-passes data test: the more-data run (-11) also had fewer passes (10.7 vs 16.4), and long900 showed passes matter; re-test 5.65B positions at 24.6 passes vs long900 (needs resumable training). Measured and dropped today: no-killerclear (+16% nodes), no-multicut (+4%), three TT replacement variants and a two-way-bucket prototype (worse under game pressure) | proposed
2026-11-09 | [search] chk-noreduce: never reduce a move that gives check (no-checkext showed check lines are worth more than depth); measured: +0.2% nodes median, +12% mean on game positions, same depth at 300 ms | proposed
