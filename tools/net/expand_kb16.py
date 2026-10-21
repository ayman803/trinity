"""Copy an 8-king-bucket network into the 16-bucket layout (src/nnue.rs).

Every 16-layout bucket lies inside one 8-layout bucket, so each new bucket
takes its parent's input weights and the network evaluates exactly as
before. Used to check the engine's 16-bucket build (same bench node count
as the 8-bucket network) and to measure its speed with identical searches.

    python tools/net/expand_kb16.py nets/default.nnue out.nnue
"""
import sys

HIDDEN, INPUTS, OUTPUT_BUCKETS = 1024, 768, 8
PARENT = [0, 1, 2, 3, 4, 4, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7]


def main(src, dst):
    data = open(src, "rb").read()
    size8 = 2 * (8 * INPUTS * HIDDEN + HIDDEN + OUTPUT_BUCKETS * (2 * HIDDEN + 1))
    if not size8 <= len(data) < size8 + 64:
        sys.exit(f"{src}: {len(data)} bytes, expected an 8-bucket 1024 network ({size8} plus padding)")
    block = 2 * INPUTS * HIDDEN  # bytes of one king bucket's input weights
    buckets = [data[i * block:(i + 1) * block] for i in range(8)]
    rest = data[8 * block:size8]
    out = b"".join(buckets[p] for p in PARENT) + rest
    out += b"\0" * (-len(out) % 64)  # bullet pads to a multiple of 64 bytes
    open(dst, "wb").write(out)
    print(f"{dst}: {len(out)} bytes (16 king buckets)")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
