#!/usr/bin/env python3
"""Source comparison for the release checklist (item 1): does Trinity's
source share text with another engine's?

Mechanical only (CLAUDE.md, originality rule, first exception): the other
engines' files are read by this program, never by a person or a model, and
it prints nothing of theirs: only percentages, and for the longest shared
runs TRINITY's own lines.

Method (the same as on 11 October 2026): every Rust, C or C++ file, with
comments and string contents removed, is cut into tokens; every run of 25
consecutive tokens is a "window". Reported: the share of Trinity's windows
that also occur in the other engine, (a) exactly and (b) after renaming
every name to one word and every number to another (so renamed copies
still match; language keywords are kept).

Usage: python3 -I release/source_similarity.py <trinity dir> <name=dir>...
       [--pairs] also compares the other engines with each other, as a
       yardstick for what unrelated engines share.
"""
import os, re, sys

WINDOW = 25
EXTS = ('.rs', '.c', '.cc', '.cpp', '.h', '.hpp')
KEYWORDS = set("""
as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub ref
return self Self static struct super trait true type unsafe use where while async await dyn
auto bool case char class constexpr default delete do double explicit float friend goto inline int long
namespace new noexcept nullptr operator private protected public short signed sizeof switch template this
throw try typedef typename union unsigned using virtual void volatile
u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 str String Vec Option Some None Result Ok Err Box
""".split())
TOKEN = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|\d[\w.]*|\S")


def strip(text):
    """Remove comments and the contents of string and char literals."""
    out, i, n = [], 0, len(text)
    while i < n:
        c = text[i]
        two = text[i:i + 2]
        if two == '//':
            j = text.find('\n', i)
            i = n if j < 0 else j
        elif two == '/*':
            j = text.find('*/', i + 2)
            i = n if j < 0 else j + 2
        elif c == '"':
            i += 1
            while i < n and text[i] != '"':
                i += 2 if text[i] == '\\' else 1
            i += 1
            out.append(' "" ')
        elif c == "'" and i + 2 < n and (text[i + 2] == "'" or (text[i + 1] == '\\' and "'" in text[i + 2:i + 6])):
            j = text.find("'", i + 2 if text[i + 1] == '\\' else i + 1)
            i = j + 1
            out.append(" '' ")
        else:
            out.append(c)
            i += 1
    return ''.join(out)


def files(root):
    for d, dirs, names in os.walk(root):
        dirs[:] = [x for x in dirs if x not in ('.git', 'target', 'build', 'node_modules')]
        for f in sorted(names):
            if f.endswith(EXTS):
                yield os.path.join(d, f)


def load(root):
    """Per file: (path, exact tokens, renamed tokens, line number of each token)."""
    result = []
    for path in files(root):
        try:
            text = open(path, encoding='utf-8', errors='replace').read()
        except OSError:
            continue
        clean = strip(text)
        exact, renamed, lines = [], [], []
        line = 1
        pos = 0
        for m in TOKEN.finditer(clean):
            line += clean.count('\n', pos, m.start())
            pos = m.start()
            t = m.group(0)
            exact.append(t)
            if t[0].isdigit():
                renamed.append('0')
            elif (t[0].isalpha() or t[0] == '_') and t not in KEYWORDS:
                renamed.append('x')
            else:
                renamed.append(t)
            lines.append(line)
        result.append((path, exact, renamed, lines))
    return result


def windows(tokens):
    return [hash(tuple(tokens[i:i + WINDOW])) for i in range(len(tokens) - WINDOW + 1)]


def window_set(src, which):
    s = set()
    for f in src:
        s.update(windows(f[which]))
    return s


def share(a, bset, which):
    total = hit = 0
    for f in a:
        w = windows(f[which])
        total += len(w)
        hit += sum(1 for x in w if x in bset)
    return 100.0 * hit / max(1, total), total


def longest_runs(a, bset, top=5):
    """Longest runs of consecutive shared exact windows in Trinity; returns
    (tokens, path, first line, last line)."""
    runs = []
    for path, exact, _, lines in a:
        w = windows(exact)
        i = 0
        while i < len(w):
            if w[i] in bset:
                j = i
                while j + 1 < len(w) and w[j + 1] in bset:
                    j += 1
                runs.append((j - i + WINDOW, path, lines[i], lines[j + WINDOW - 1]))
                i = j + 1
            else:
                i += 1
    return sorted(runs, reverse=True)[:top]


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    pairs = '--pairs' in sys.argv
    trinity = load(args[0])
    others = {}
    for a in args[1:]:
        name, d = a.split('=', 1)
        others[name] = load(d)
    ntok = sum(len(f[1]) for f in trinity)
    print(f"Trinity: {len(trinity)} files, {ntok} tokens, windows of {WINDOW} tokens")
    print("| Trinity vs | files | tokens | exact | names and numbers renamed |")
    print("|---|---|---|---|---|")
    sets = {}
    for name, src in others.items():
        sets[name] = (window_set(src, 1), window_set(src, 2))
        e, _ = share(trinity, sets[name][0], 1)
        r, _ = share(trinity, sets[name][1], 2)
        print(f"| {name} | {len(src)} | {sum(len(f[1]) for f in src)} | {e:.2f}% | {r:.2f}% |")
    for name in others:
        runs = longest_runs(trinity, sets[name][0])
        if runs:
            print(f"Longest exact runs shared with {name} (Trinity's side): " +
                  "; ".join(f"{n} tokens at {os.path.relpath(p, args[0])}:{a}-{b}" for n, p, a, b in runs))
    if pairs:
        print("Yardstick, other engines against each other (exact / renamed):")
        names = list(others)
        for i, a in enumerate(names):
            for b in names[i + 1:]:
                e1, _ = share(others[a], sets[b][0], 1)
                r1, _ = share(others[a], sets[b][1], 2)
                e2, _ = share(others[b], sets[a][0], 1)
                r2, _ = share(others[b], sets[a][1], 2)
                print(f"  {a} in {b}: {e1:.2f}% / {r1:.2f}%;  {b} in {a}: {e2:.2f}% / {r2:.2f}%")


if __name__ == '__main__':
    main()
