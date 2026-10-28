"""Guard: no untested engine code on main (advisor, 28 October 2026).

A commit on main, or a push to main, is refused if it changes how Trinity
plays, unless:
  - its first line starts with "Merge", says "passed" and names the test
    ("#76"), as in "Merge quiet-skip (SPRT #78 passed, +0.4 +/- 2.1)"; or
  - it puts the engine back exactly as it was in a commit already on
    origin/main (an undo, or bringing origin/main's tested code in).
Engine code: src/ (except the analysis tools below), Cargo.toml,
Cargo.lock, build.rs and nets/default.nnue. A change of the version
number alone does not count.

Used by .githooks/commit-msg and .githooks/pre-push, which the cloud
sessions switch on at start (.claude/hooks/session-start.sh). Not used on
the PC.
"""

import hashlib
import re
import subprocess
import sys

ENGINE = ["src", "Cargo.toml", "Cargo.lock", "build.rs", "nets/default.nnue"]
TOOLS = {"src/datagen.rs", "src/forensics.rs", "src/select.rs", "src/similarity.rs", "src/timeuse.rs"}
VERSION_LINE = {
    "Cargo.toml": re.compile(r'(?m)^(version = )"[^"]*"'),
    "Cargo.lock": re.compile(r'(?m)^(name = "trinity"\nversion = )"[^"]*"'),
}
MERGE_OK = re.compile(r"^Merge\b.*\bpassed\b.*#\d+|^Merge\b.*#\d+.*\bpassed\b")
HISTORY = 300


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True, check=True).stdout


def fingerprint(tree):
    """Hash of the engine files in a commit or tree, version numbers left out."""
    h = hashlib.sha256()
    for line in git("ls-tree", "-r", tree, "--", *ENGINE).splitlines():
        meta, path = line.split("\t", 1)
        if path in TOOLS:
            continue
        if path in ("Cargo.toml", "Cargo.lock"):
            text = git("show", f"{meta.split()[2]}")
            h.update(path.encode() + VERSION_LINE[path].sub(r'\1""', text).encode())
        else:
            h.update(line.encode() + b"\n")
    return h.hexdigest()


def tested_states():
    try:
        commits = git("rev-list", "--first-parent", f"-{HISTORY}", "origin/main").split()
    except subprocess.CalledProcessError:
        return set()
    return {fingerprint(c) for c in commits}


def problem(new_tree, parent, message, states):
    """Why this change may not go on main, or None."""
    if parent is not None and fingerprint(new_tree) == fingerprint(parent):
        return None
    if MERGE_OK.search(message.splitlines()[0] if message else ""):
        return None
    if states is None:
        states = tested_states()
    if fingerprint(new_tree) in states:
        return None
    return "it changes engine code without a passed test"


def refuse(what, why):
    sys.stderr.write(
        f"REFUSED by .githooks/guard-main.py: {what} {why}.\n"
        "Engine code reaches main only as a merge whose first line names the\n"
        'passed test, e.g. "Merge quiet-skip (SPRT #78 passed, ...)". Commit\n'
        "candidates on their own branch.\n"
    )
    sys.exit(1)


def commit_msg(msg_file):
    try:
        branch = git("symbolic-ref", "--short", "HEAD").strip()
    except subprocess.CalledProcessError:
        return
    if branch != "main":
        return
    with open(msg_file, encoding="utf-8") as f:
        message = "\n".join(l for l in f.read().splitlines() if not l.startswith("#")).strip()
    try:
        parent = git("rev-parse", "HEAD").strip()
    except subprocess.CalledProcessError:
        parent = None
    why = problem(git("write-tree").strip(), parent, message, None)
    if why:
        refuse("this commit on main:", why)


def pre_push():
    zero = "0" * 40
    states = None
    for line in sys.stdin.read().splitlines():
        local_ref, local_sha, remote_ref, remote_sha = line.split()
        if remote_ref != "refs/heads/main" or local_sha == zero:
            continue
        known = remote_sha != zero and subprocess.run(
            ["git", "cat-file", "-e", f"{remote_sha}^{{commit}}"], capture_output=True
        ).returncode == 0
        # Commits not yet on GitHub (if its newest main is not fetched yet,
        # everything not on a fetched origin branch; git refuses that push
        # anyway until it is pulled).
        span = [local_sha, f"^{remote_sha}"] if known else [local_sha, "--not", "--remotes=origin"]
        for c in git("rev-list", "--first-parent", *span).split():
            parents = git("rev-list", "--parents", "-n", "1", c).split()[1:]
            message = git("log", "-1", "--format=%B", c).strip()
            if states is None:
                states = tested_states()
            why = problem(c, parents[0] if parents else None, message, states)
            if why:
                refuse(f"commit {c[:10]} on main:", why)


if __name__ == "__main__":
    if sys.argv[1] == "commit-msg":
        commit_msg(sys.argv[2])
    else:
        pre_push()
