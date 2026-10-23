#!/bin/bash
# Set WORK (engines trinity-release-1.1 and trinity-1.0.0) and REPO before running.
# Game part of the release robustness suite: 1.1 vs 1.0.0 under odd settings.
run() { name=$1; shift; echo "=== $name"; $REPO/sprt/tools/fastchess-linux-x86-64/fastchess -engine cmd=$WORK/trinity-release-1.1 name=T11 -engine cmd=$WORK/trinity-1.0.0 name=T100 -each proto=uci "$@" -openings file=$REPO/sprt/books/8moves_v3.pgn format=pgn order=random -repeat -recover -pgnout file=$WORK/robust/$name.pgn 2>&1 | grep -E "^Games|Elo:|loses on time|illegal|disconnect|crash|stall|Warning" | sort | uniq -c | sort -rn | head -8; }
run hash1-40x4 tc=40/4 option.Hash=1 option.Threads=1 -games 2 -rounds 40 -concurrency 2
run hash1024-40x4 tc=40/4 option.Hash=1024 option.Threads=1 -games 2 -rounds 10 -concurrency 1
run threads2-40x8 tc=40/8 option.Hash=64 option.Threads=2 -games 2 -rounds 8 -concurrency 1
run sd-5s tc=5 option.Hash=16 option.Threads=1 -games 2 -rounds 40 -concurrency 2
