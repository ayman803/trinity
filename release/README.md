# Release robustness suite (advisor, 23 October 2026)

Run on every release, and on any version a CCRL tester will use. It checks
the odd but legal things a tester's setup may do, not strength.

1. UCI checks (`python3 -I release/uci_suite.py <engine>`, about 1 minute):
   hash 1 MB and 1024 MB; 4 threads; `go infinite` + `stop`; `stop` right
   after `go`; `isready` and `quit` during a search; 30 ms left without
   increment; 40/15-style `movestogo 40` and `movestogo 1` with 2 s left;
   a long move list; en passant, promotion, castling, mate-in-1,
   one-legal-move and 50-move-counter positions; Hash resize and Clear
   Hash between games. Every check must pass.
2. Games (fastchess, the release against the previous release): repeating
   40/4 with Hash 1 MB (80 games) and Hash 1024 MB (20 games), 40/8 with
   2 threads each (16 games), sudden death 5 s with no increment (80
   games). Pass: no crash, no illegal move, no disconnect, no game lost on
   time by the release.

Results are logged in ESTIMATES.md, "Release checks".
