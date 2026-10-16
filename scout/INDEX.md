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
