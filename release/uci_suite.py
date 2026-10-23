"""Release robustness suite, UCI part: odd but legal things a GUI or tester may do."""
import subprocess, sys, time, threading, queue

def start(path):
    p = subprocess.Popen([path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)
    q = queue.Queue()
    threading.Thread(target=lambda: [q.put(l.rstrip()) for l in p.stdout], daemon=True).start()
    return p, q

def send(p, s): p.stdin.write(s + "\n"); p.stdin.flush()

def wait_for(q, prefix, timeout):
    end = time.time() + timeout
    while time.time() < end:
        try:
            l = q.get(timeout=max(0.01, end - time.time()))
        except queue.Empty:
            break
        if l.startswith(prefix):
            return l, time.time()
    return None, time.time()

results = []
def check(name, ok, detail=""):
    results.append((name, ok, detail)); print(("PASS " if ok else "FAIL ") + name + (" - " + detail if detail else ""), flush=True)

def fresh(path, opts=()):
    p, q = start(path); send(p, "uci"); l, _ = wait_for(q, "uciok", 5)
    for o in opts: send(p, o)
    send(p, "isready"); r, _ = wait_for(q, "readyok", 20)
    return p, q, (l is not None and r is not None)

path = sys.argv[1]
# 1. Hash sizes 1 MB and 1024 MB, a search in each.
for h in (1, 1024):
    p, q, ok = fresh(path, [f"setoption name Hash value {h}"])
    send(p, "position startpos"); t = time.time(); send(p, "go movetime 300"); l, t2 = wait_for(q, "bestmove", 5)
    check(f"hash {h} MB: setoption, search, bestmove", ok and l is not None, f"{(t2-t)*1000:.0f} ms" if l else "no bestmove")
    send(p, "quit"); p.wait(5)
# 2. Threads 4.
p, q, ok = fresh(path, ["setoption name Threads value 4", "setoption name Hash value 64"])
send(p, "position startpos moves e2e4 e7e5 g1f3"); t = time.time(); send(p, "go movetime 500"); l, t2 = wait_for(q, "bestmove", 5)
check("threads 4: search and bestmove", ok and l is not None, f"{(t2-t)*1000:.0f} ms" if l else "no bestmove")
# 3. go infinite then stop.
send(p, "position startpos"); send(p, "go infinite"); time.sleep(0.3); t = time.time(); send(p, "stop"); l, t2 = wait_for(q, "bestmove", 3)
check("go infinite + stop (4 threads): bestmove within 1 s", l is not None and t2 - t < 1.0, f"{(t2-t)*1000:.0f} ms" if l else "none")
# 4. stop immediately after go.
send(p, "go infinite"); send(p, "stop"); l, t2 = wait_for(q, "bestmove", 3)
check("stop right after go: bestmove", l is not None and l.split()[1] != "(none)", l or "none")
# 5. isready during a search.
send(p, "go infinite"); time.sleep(0.2); send(p, "isready"); r, _ = wait_for(q, "readyok", 2); send(p, "stop"); wait_for(q, "bestmove", 3)
check("isready during search answers readyok", r is not None)
# 6. quit during a search: process exits.
send(p, "go infinite"); time.sleep(0.2); t = time.time(); send(p, "quit")
try: p.wait(3); check("quit during search: exits within 3 s", True, f"{(time.time()-t)*1000:.0f} ms")
except subprocess.TimeoutExpired: check("quit during search: exits within 3 s", False); p.kill()
# 7. Very little time: wtime 30 ms, no increment.
p, q, ok = fresh(path)
send(p, "position startpos moves e2e4"); t = time.time(); send(p, "go wtime 30 btime 30"); l, t2 = wait_for(q, "bestmove", 3)
check("go btime 30 (no increment): bestmove in time", l is not None and (t2 - t) < 0.03 + 0.02, f"{(t2-t)*1000:.0f} ms" if l else "none")
# 8. 40/15-style: movestogo with no increment.
send(p, "position startpos"); t = time.time(); send(p, "go wtime 900000 btime 900000 movestogo 40"); l, t2 = wait_for(q, "bestmove", 120)
check("go wtime 15 min movestogo 40: bestmove, under 1/10 of the clock", l is not None and (t2 - t) < 90, f"{(t2-t):.1f} s" if l else "none")
# 9. movestogo 1 with 2 s left: must not flag.
send(p, "go wtime 2000 btime 2000 movestogo 1"); t = time.time(); l, t2 = wait_for(q, "bestmove", 5)
check("go wtime 2 s movestogo 1: bestmove within 2 s", l is not None and (t2 - t) < 2.0, f"{(t2-t)*1000:.0f} ms" if l else "none")
# 10. Long move list (a 200-ply game) then search.
moves = "g1f3 g8f6 f3g1 f6g8 " * 24
send(p, "ucinewgame"); send(p, "position startpos moves " + moves.strip()); send(p, "go movetime 200"); l, _ = wait_for(q, "bestmove", 5)
check("position with a 96-move list (repetitions), search", l is not None, l or "none")
# 11. Edge FENs: en passant, promotion, castling rights, mate, stalemate-ish.
for name, fen in [("en passant", "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3"),
                  ("promotion", "8/P7/8/8/8/8/k6K/8 w - - 0 1"),
                  ("castling both sides", "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1"),
                  ("mate in 1", "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1"),
                  ("one legal move", "k7/8/1K6/8/8/8/8/7R b - - 0 1"),
                  ("50-move counter 99", "8/8/4k3/8/8/4K3/4P3/8 w - - 99 120")]:
    send(p, "position fen " + fen); send(p, "go movetime 200"); l, _ = wait_for(q, "bestmove", 5)
    check(f"FEN {name}: bestmove", l is not None and l.split()[1] != "(none)" and l.split()[1] != "0000", l or "none")
# 12. Hash change after searching, then clear (GUIs do this between games).
send(p, "setoption name Hash value 256"); send(p, "setoption name Clear Hash"); send(p, "isready"); r, _ = wait_for(q, "readyok", 20)
send(p, "position startpos"); send(p, "go depth 8"); l, _ = wait_for(q, "bestmove", 20)
check("Hash resize to 256 MB after searching, then search", r is not None and l is not None)
send(p, "quit"); p.wait(5)
print(f"SUMMARY {sum(1 for r in results if r[1])} of {len(results)} passed")
