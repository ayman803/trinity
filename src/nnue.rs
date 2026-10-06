//! NNUE evaluation: a `(768xKB -> HIDDEN)x2 -> 1` network with SCReLU
//! activation, king input buckets (one set of input weights per region of
//! the board the perspective's own king stands in, mirrored so the king is
//! always on files a-d) and output buckets (one set of output weights per
//! material range), matching the layout that the bullet trainer writes (see
//! `trainer/src/main.rs`).
//!
//! The first layer ("accumulator") is updated incrementally as moves are
//! made, which is what makes NNUE fast enough for alpha-beta search.

use crate::board::Board;
use crate::types::*;

pub const HIDDEN: usize = 512;
/// Output buckets by number of pieces on the board: (pieces - 2) / 4.
pub const OUTPUT_BUCKETS: usize = 8;
pub const QA: i32 = 255;
pub const QB: i32 = 64;
pub const SCALE: i32 = 400;
const INPUTS: usize = 768;

/// King bucket for each square of the own king, seen from its side, after
/// mirroring onto files a-d (index = rank * 4 + file). MUST match the
/// trainer. Fine near the home corner where kings usually stand, coarse
/// further up.
#[rustfmt::skip]
pub const KING_BUCKET_LAYOUT: [usize; 32] = [
    0, 1, 2, 3,
    4, 4, 5, 5,
    6, 6, 6, 6,
    6, 6, 6, 6,
    7, 7, 7, 7,
    7, 7, 7, 7,
    7, 7, 7, 7,
    7, 7, 7, 7,
];
pub const KING_BUCKETS: usize = 8;

/// (bucket offset, square xor for mirroring) of a king on `rel_ksq` (a
/// square seen from the king's own side).
#[inline(always)]
fn king_bucket(rel_ksq: Square) -> (usize, usize) {
    let file = rel_ksq % 8;
    let (f4, flip) = if file > 3 { (7 - file, 7) } else { (file, 0) };
    (KING_BUCKET_LAYOUT[(rel_ksq / 8) * 4 + f4] * INPUTS, flip)
}

#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct Accumulator {
    pub vals: [i16; HIDDEN],
}

#[repr(C)]
pub struct Network {
    /// One column of `HIDDEN` weights per input feature and king bucket
    /// (quantised by QA).
    feature_weights: [Accumulator; INPUTS * KING_BUCKETS],
    feature_bias: Accumulator,
    /// Per bucket: the first `HIDDEN` weights apply to the side to move,
    /// the rest to the other side (quantised by QB).
    output_weights: [[i16; 2 * HIDDEN]; OUTPUT_BUCKETS],
    output_bias: [i16; OUTPUT_BUCKETS],
    /// False for networks trained without king buckets: they use bucket 0
    /// and no mirroring, so they evaluate exactly as before.
    bucketed: bool,
}

/// Size in bytes of a network file without bullet's trailing padding.
#[cfg_attr(not(trinity_net), allow(dead_code))]
pub const NET_BYTES: usize = 2 * (KING_BUCKETS * INPUTS * HIDDEN + HIDDEN + OUTPUT_BUCKETS * (2 * HIDDEN + 1));

/// The previous architecture without king buckets. Loaded as it was trained:
/// a single bucket and no mirroring (see `Network::bucketed`).
#[cfg_attr(not(trinity_net), allow(dead_code))]
const UNBUCKETED_BYTES: usize = 2 * (INPUTS * HIDDEN + HIDDEN + OUTPUT_BUCKETS * (2 * HIDDEN + 1));

/// The earlier architecture (256 hidden, no output buckets). Such files are
/// still accepted: they convert exactly into the current layout by giving
/// the extra neurons zero weights and every bucket the same output weights.
const LEGACY_HIDDEN: usize = 256;
#[cfg_attr(not(trinity_net), allow(dead_code))]
const LEGACY_BYTES: usize = 2 * (INPUTS * LEGACY_HIDDEN + LEGACY_HIDDEN + 2 * LEGACY_HIDDEN + 1);

impl Network {
    #[cfg_attr(not(trinity_net), allow(dead_code))]
    /// Parse a network from the raw little-endian i16 layout that bullet
    /// writes to `quantised.bin`. Trailing padding is ignored.
    pub fn from_bytes(bytes: &[u8]) -> Result<Box<Network>, String> {
        // bullet pads files to a multiple of 64 bytes.
        let fits = |size: usize| (size..size + 64).contains(&bytes.len());
        let (hidden, buckets, king_buckets, size) = if fits(NET_BYTES) {
            (HIDDEN, OUTPUT_BUCKETS, KING_BUCKETS, NET_BYTES)
        } else if fits(UNBUCKETED_BYTES) {
            (HIDDEN, OUTPUT_BUCKETS, 1, UNBUCKETED_BYTES)
        } else if fits(LEGACY_BYTES) {
            (LEGACY_HIDDEN, 1, 1, LEGACY_BYTES)
        } else {
            return Err(format!(
                "network file is {} bytes, expected {NET_BYTES} (hidden size {HIDDEN}, {KING_BUCKETS} king buckets, \
                 {OUTPUT_BUCKETS} output buckets), {UNBUCKETED_BYTES} (no king buckets) or {LEGACY_BYTES} \
                 (hidden size {LEGACY_HIDDEN}), plus padding",
                bytes.len()
            ));
        };
        let mut vals = bytes[..size].chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]]));
        // Allocate zeroed on the heap: the struct is too large for the stack.
        let mut net: Box<Network> = unsafe {
            let layout = std::alloc::Layout::new::<Network>();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut Network;
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            Box::from_raw(ptr)
        };
        // Anything not read stays zero (unused neurons of a legacy net).
        net.bucketed = king_buckets > 1;
        for col in net.feature_weights[..king_buckets * INPUTS].iter_mut() {
            for v in col.vals[..hidden].iter_mut() {
                *v = vals.next().unwrap();
            }
        }
        for v in net.feature_bias.vals[..hidden].iter_mut() {
            *v = vals.next().unwrap();
        }
        for bucket in 0..buckets {
            for half in 0..2 {
                for v in net.output_weights[bucket][half * HIDDEN..half * HIDDEN + hidden].iter_mut() {
                    *v = vals.next().unwrap();
                }
            }
        }
        for b in net.output_bias[..buckets].iter_mut() {
            *b = vals.next().unwrap();
        }
        // A single-bucket net uses the same output layer for every bucket.
        for bucket in buckets..OUTPUT_BUCKETS {
            net.output_weights[bucket] = net.output_weights[0];
            net.output_bias[bucket] = net.output_bias[0];
        }
        Ok(net)
    }
}

#[cfg(trinity_net)]
static EMBEDDED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/net.bin"));

static NETWORK: std::sync::OnceLock<Option<&'static Network>> = std::sync::OnceLock::new();

/// The network compiled into the binary, if any.
pub fn network() -> Option<&'static Network> {
    *NETWORK.get_or_init(load_embedded)
}

#[cfg(trinity_net)]
fn load_embedded() -> Option<&'static Network> {
    // A mismatched network is a build mistake: fail loudly rather than
    // silently playing with a different evaluation.
    match Network::from_bytes(EMBEDDED) {
        Ok(n) => Some(Box::leak(n)),
        Err(e) => panic!("embedded network is invalid: {e}"),
    }
}

#[cfg(not(trinity_net))]
fn load_embedded() -> Option<&'static Network> {
    None
}

/// Where `perspective`'s input weights come from in `b`: (offset of its
/// king bucket, square xor that mirrors files when its king is on e-h).
#[inline(always)]
fn view(net: &Network, b: &Board, perspective: usize) -> (usize, usize) {
    if !net.bucketed {
        return (0, 0);
    }
    let ksq = b.king_sq(perspective);
    king_bucket(if perspective == WHITE { ksq } else { flip(ksq) })
}

/// Input feature index of `piece` on `sq`, seen from `perspective` (whose
/// king bucket and mirroring are `v`): our pieces come first, and Black's
/// view is flipped vertically, so both sides share the same weights.
#[inline(always)]
fn feature(perspective: usize, v: (usize, usize), piece: Piece, sq: Square) -> usize {
    let (color, pt) = (piece_color(piece), piece_type(piece));
    let (offset, mirror) = v;
    if perspective == WHITE {
        offset + (color * 6 + pt) * 64 + (sq ^ mirror)
    } else {
        offset + ((color ^ 1) * 6 + pt) * 64 + (flip(sq) ^ mirror)
    }
}

/// Accumulators for both perspectives, indexed by colour.
#[derive(Clone, Copy)]
pub struct AccPair {
    pub acc: [Accumulator; 2],
}

impl AccPair {
    pub fn from_board(net: &Network, b: &Board) -> AccPair {
        AccPair { acc: [refresh(net, b, WHITE), refresh(net, b, BLACK)] }
    }

    /// Compute the accumulators after `m` (played on `before`, giving
    /// `after`) from the parent's accumulators by adding/removing only the
    /// changed features.
    #[cfg(test)]
    pub fn update(&self, net: &Network, before: &Board, after: &Board, m: Move) -> AccPair {
        let mut out = *self;
        self.update_into(&mut out, net, &mut RefreshCache::new(net), before, after, m);
        out
    }

    /// Like `update`, but writes into `out` (no temporary copy). When the
    /// mover's king changes bucket or side of the board, its accumulator is
    /// rebuilt instead, through `cache`.
    #[inline(always)]
    pub fn update_into(
        &self,
        out: &mut AccPair,
        net: &Network,
        cache: &mut RefreshCache,
        before: &Board,
        after: &Board,
        m: Move,
    ) {
        let us = before.stm;
        let moved = before.piece_at(m.from());
        let mut adds: [(Piece, Square); 2] = [(NO_PIECE, 0); 2];
        let mut subs: [(Piece, Square); 2] = [(NO_PIECE, 0); 2];
        let (mut na, mut ns) = (0, 0);

        subs[ns] = (moved, m.from());
        ns += 1;
        let placed = if m.is_promotion() { make_piece(us, m.promo_piece()) } else { moved };
        adds[na] = (placed, m.to());
        na += 1;
        if m.is_capture() {
            let cap_sq = if m.is_ep() { m.to() ^ 8 } else { m.to() };
            subs[ns] = (before.piece_at(cap_sq), cap_sq);
            ns += 1;
        } else if m.is_castle() {
            let (rf, rt) =
                if m.flag() == FLAG_KING_CASTLE { (m.to() + 1, m.to() - 1) } else { (m.to() - 2, m.to() + 1) };
            let rook = make_piece(us, ROOK);
            subs[ns] = (rook, rf);
            ns += 1;
            adds[na] = (rook, rt);
            na += 1;
        }

        for persp in [WHITE, BLACK] {
            let v = view(net, before, persp);
            if persp == us && piece_type(moved) == KING && view(net, after, persp) != v {
                out.acc[persp] = cache.refresh(net, after, persp);
                continue;
            }
            let src = &self.acc[persp].vals;
            let dst = &mut out.acc[persp].vals;
            let col = |(p, sq): (Piece, Square)| &net.feature_weights[feature(persp, v, p, sq)].vals;
            // One fused pass per case: dst = src + adds - subs.
            match (na, ns) {
                (1, 1) => {
                    let (a0, s0) = (col(adds[0]), col(subs[0]));
                    for i in 0..HIDDEN {
                        dst[i] = src[i] + a0[i] - s0[i];
                    }
                }
                (1, 2) => {
                    let (a0, s0, s1) = (col(adds[0]), col(subs[0]), col(subs[1]));
                    for i in 0..HIDDEN {
                        dst[i] = src[i] + a0[i] - s0[i] - s1[i];
                    }
                }
                _ => {
                    let (a0, a1, s0, s1) = (col(adds[0]), col(adds[1]), col(subs[0]), col(subs[1]));
                    for i in 0..HIDDEN {
                        dst[i] = src[i] + a0[i] + a1[i] - s0[i] - s1[i];
                    }
                }
            }
        }
    }
}

/// The accumulator of `perspective` for `b`, computed from scratch.
fn refresh(net: &Network, b: &Board, perspective: usize) -> Accumulator {
    let v = view(net, b, perspective);
    let mut acc = net.feature_bias;
    for sq in Bits(b.occupied()) {
        let col = &net.feature_weights[feature(perspective, v, b.piece_at(sq), sq)].vals;
        for (a, w) in acc.vals.iter_mut().zip(col) {
            *a += *w;
        }
    }
    acc
}

/// Accumulator cache for king-bucket changes (an idea known as "Finny
/// tables"): for each perspective, king bucket and mirroring, the
/// accumulator of the last position rebuilt there and that position's
/// pieces. A rebuild then only adds and removes the pieces that differ,
/// usually a handful instead of all of them.
pub struct RefreshCache {
    entries: [[CacheEntry; 2 * KING_BUCKETS]; 2],
}

#[derive(Clone, Copy)]
struct CacheEntry {
    acc: Accumulator,
    pieces: [Bitboard; 6],
    colors: [Bitboard; 2],
}

impl RefreshCache {
    /// Every entry starts as the empty board (bias only), which is exact.
    pub fn new(net: &Network) -> Box<RefreshCache> {
        let empty = CacheEntry { acc: net.feature_bias, pieces: [0; 6], colors: [0; 2] };
        Box::new(RefreshCache { entries: [[empty; 2 * KING_BUCKETS]; 2] })
    }

    fn refresh(&mut self, net: &Network, b: &Board, perspective: usize) -> Accumulator {
        let v = view(net, b, perspective);
        let e = &mut self.entries[perspective][v.0 / INPUTS * 2 + usize::from(v.1 != 0)];
        for color in [WHITE, BLACK] {
            for pt in 0..6 {
                let now = b.colors[color] & b.pieces[pt];
                let then = e.colors[color] & e.pieces[pt];
                let piece = make_piece(color, pt);
                for sq in Bits(now & !then) {
                    let col = &net.feature_weights[feature(perspective, v, piece, sq)].vals;
                    for (a, w) in e.acc.vals.iter_mut().zip(col) {
                        *a += *w;
                    }
                }
                for sq in Bits(then & !now) {
                    let col = &net.feature_weights[feature(perspective, v, piece, sq)].vals;
                    for (a, w) in e.acc.vals.iter_mut().zip(col) {
                        *a -= *w;
                    }
                }
            }
        }
        e.pieces = b.pieces;
        e.colors = b.colors;
        e.acc
    }
}

/// Squared clipped ReLU, dotted with the output weights. Written as
/// `(v * w) * v` with the first product in i16 so the compiler can use
/// the `madd` SIMD instruction; `v * w` fits in i16 because |w| < 128.
#[inline(always)]
fn screlu_dot(acc: &[i16; HIDDEN], weights: &[i16]) -> i32 {
    let mut sum = 0i32;
    for (&a, &w) in acc.iter().zip(weights) {
        let v = a.clamp(0, QA as i16);
        sum += i32::from(v.wrapping_mul(w)) * i32::from(v);
    }
    sum
}

#[inline(always)]
fn output_bucket(b: &Board) -> usize {
    (b.occupied().count_ones() as usize - 2) / 32usize.div_ceil(OUTPUT_BUCKETS)
}

/// Evaluation of `b` (whose accumulators are `pair`) for the side to move.
pub fn evaluate(net: &Network, pair: &AccPair, b: &Board) -> i32 {
    let bucket = output_bucket(b);
    let weights = &net.output_weights[bucket];
    let mut out = screlu_dot(&pair.acc[b.stm].vals, &weights[..HIDDEN])
        + screlu_dot(&pair.acc[b.stm ^ 1].vals, &weights[HIDDEN..]);
    out /= QA;
    out += i32::from(net.output_bias[bucket]);
    out * SCALE / (QA * QB)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::movegen::legal_moves;

    /// A random network in the bullet file layout, for testing.
    pub fn random_net_bytes(seed: u64) -> Vec<u8> {
        random_bytes(seed, NET_BYTES, INPUTS * HIDDEN + HIDDEN)
    }

    fn random_bytes(seed: u64, size: usize, out_start: usize) -> Vec<u8> {
        let mut s = seed;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let n = size / 2;
        let mut bytes = Vec::with_capacity(size + 64);
        for i in 0..n {
            // Keep output weights within |w| < 128 as the trainer's clipping does.
            let v: i16 = if i >= out_start { (next() % 255) as i16 - 127 } else { (next() % 101) as i16 - 50 };
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        bytes.resize(size.div_ceil(64) * 64, 0);
        bytes
    }

    /// Straightforward floating-point reference implementation.
    fn reference_eval(net: &Network, b: &Board) -> f64 {
        let mut hidden = [[0f64; HIDDEN]; 2];
        for persp in [WHITE, BLACK] {
            for i in 0..HIDDEN {
                hidden[persp][i] = f64::from(net.feature_bias.vals[i]) / QA as f64;
            }
            for sq in Bits(b.occupied()) {
                let f = feature(persp, view(net, b, persp), b.piece_at(sq), sq);
                for i in 0..HIDDEN {
                    hidden[persp][i] += f64::from(net.feature_weights[f].vals[i]) / QA as f64;
                }
            }
        }
        let bucket = (b.occupied().count_ones() as usize - 2) / 4;
        let mut out = f64::from(net.output_bias[bucket]) / (QA * QB) as f64;
        for (k, persp) in [b.stm, b.stm ^ 1].into_iter().enumerate() {
            for i in 0..HIDDEN {
                let v = hidden[persp][i].clamp(0.0, 1.0);
                out += v * v * f64::from(net.output_weights[bucket][k * HIDDEN + i]) / QB as f64;
            }
        }
        out * SCALE as f64
    }

    #[test]
    fn incremental_matches_refresh_and_reference() {
        let net = Network::from_bytes(&random_net_bytes(0x5eed)).unwrap();
        let fens = [
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "8/8/8/2k5/3Pp3/8/8/4K2R b K d3 0 1",
            "8/8/8/1k6/3Pp3/8/8/4K2R w K - 0 1",
            "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1",
            "r3k2r/8/8/8/8/8/8/4K3 b kq - 0 1",
            "8/8/3k4/8/8/4K3/8/8 w - - 0 1",
        ];
        for fen in fens {
            let b = Board::from_fen(fen).unwrap();
            let pair = AccPair::from_board(&net, &b);
            for &m in legal_moves(&b).iter() {
                let child = b.make_move(m);
                let inc = pair.update(&net, &b, &child, m);
                let fresh = AccPair::from_board(&net, &child);
                assert!(inc.acc[0].vals == fresh.acc[0].vals && inc.acc[1].vals == fresh.acc[1].vals, "{fen} {m}");
                let q = evaluate(&net, &inc, &child) as f64;
                let r = reference_eval(&net, &child);
                assert!((q - r).abs() <= 2.0 + r.abs() * 0.01, "quantised {q} vs reference {r}");
            }
        }
    }

    /// One cache reused across a long random game stays exact.
    #[test]
    fn refresh_cache_stays_exact() {
        let net = Network::from_bytes(&random_net_bytes(0xcafe)).unwrap();
        let mut cache = RefreshCache::new(&net);
        let mut b = Board::from_fen("r3k2r/pppq1ppp/2npbn2/4p3/4P3/2NPBN2/PPPQ1PPP/R3K2R w KQkq - 0 1").unwrap();
        let mut pair = AccPair::from_board(&net, &b);
        let mut seed = 12345u64;
        for _ in 0..300 {
            let moves = legal_moves(&b);
            if moves.is_empty() {
                break;
            }
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            // Prefer king moves so the cache is exercised often.
            let kings: Vec<Move> = moves.iter().copied().filter(|m| piece_type(b.piece_at(m.from())) == KING).collect();
            let pool: &[Move] = if !kings.is_empty() && seed % 3 != 0 { &kings } else { moves.as_slice() };
            let m = pool[(seed >> 33) as usize % pool.len()];
            let child = b.make_move(m);
            let mut next = pair;
            pair.update_into(&mut next, &net, &mut cache, &b, &child, m);
            let fresh = AccPair::from_board(&net, &child);
            assert!(next.acc[0].vals == fresh.acc[0].vals && next.acc[1].vals == fresh.acc[1].vals, "{m}");
            b = child;
            pair = next;
        }
    }

    /// A legacy (256 hidden, single output) network must evaluate exactly
    /// as the old engine did.
    #[test]
    fn legacy_network_converts_exactly() {
        let bytes = random_bytes(0xbeef, LEGACY_BYTES, INPUTS * LEGACY_HIDDEN + LEGACY_HIDDEN);
        let raw: Vec<i32> = bytes.chunks_exact(2).map(|c| i32::from(i16::from_le_bytes([c[0], c[1]]))).collect();
        let net = Network::from_bytes(&bytes).unwrap();
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
            "8/8/8/2k5/3Pp3/8/8/4K2R b K d3 0 1",
        ];
        for fen in fens {
            let b = Board::from_fen(fen).unwrap();
            // The old engine's integer arithmetic, straight from the file.
            let h = LEGACY_HIDDEN;
            let mut acc = [[0i32; LEGACY_HIDDEN]; 2];
            for persp in [WHITE, BLACK] {
                for i in 0..h {
                    acc[persp][i] = raw[INPUTS * h + i];
                }
                for sq in Bits(b.occupied()) {
                    let f = feature(persp, (0, 0), b.piece_at(sq), sq);
                    for i in 0..h {
                        acc[persp][i] += raw[f * h + i];
                    }
                }
            }
            let out_w = &raw[INPUTS * h + h..];
            let mut sum = 0;
            for (k, persp) in [b.stm, b.stm ^ 1].into_iter().enumerate() {
                for i in 0..h {
                    let v = acc[persp][i].clamp(0, QA);
                    sum += v * v * out_w[k * h + i];
                }
            }
            let expected = (sum / QA + out_w[2 * h]) * SCALE / (QA * QB);
            let pair = AccPair::from_board(&net, &b);
            assert_eq!(evaluate(&net, &pair, &b), expected, "{fen}");
        }
    }

    #[test]
    fn rejects_wrong_size() {
        assert!(Network::from_bytes(&[0u8; 1000]).is_err());
    }
}
