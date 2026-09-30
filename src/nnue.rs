//! NNUE evaluation: a `(768 -> HIDDEN)x2 -> 1` network with SCReLU
//! activation, matching the layout that the bullet trainer writes
//! (see `trainer/src/main.rs`).
//!
//! The first layer ("accumulator") is updated incrementally as moves are
//! made, which is what makes NNUE fast enough for alpha-beta search.

use crate::board::Board;
use crate::types::*;

pub const HIDDEN: usize = 256;
pub const QA: i32 = 255;
pub const QB: i32 = 64;
pub const SCALE: i32 = 400;
const INPUTS: usize = 768;

#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct Accumulator {
    pub vals: [i16; HIDDEN],
}

#[repr(C)]
pub struct Network {
    /// One column of `HIDDEN` weights per input feature (quantised by QA).
    feature_weights: [Accumulator; INPUTS],
    feature_bias: Accumulator,
    /// First `HIDDEN` weights apply to the side to move, the rest to the
    /// other side (quantised by QB).
    output_weights: [i16; 2 * HIDDEN],
    output_bias: i16,
}

/// Size in bytes of a network file without bullet's trailing padding.
#[cfg_attr(not(trinity_net), allow(dead_code))]
pub const NET_BYTES: usize = 2 * (INPUTS * HIDDEN + HIDDEN + 2 * HIDDEN + 1);

impl Network {
    #[cfg_attr(not(trinity_net), allow(dead_code))]
    /// Parse a network from the raw little-endian i16 layout that bullet
    /// writes to `quantised.bin`. Trailing padding is ignored.
    pub fn from_bytes(bytes: &[u8]) -> Result<Box<Network>, String> {
        if bytes.len() < NET_BYTES {
            return Err(format!(
                "network file is {} bytes, expected at least {NET_BYTES} (hidden size {HIDDEN})",
                bytes.len()
            ));
        }
        if bytes.len() >= NET_BYTES + 64 {
            return Err(format!(
                "network file is {} bytes, expected {NET_BYTES} plus padding: hidden size mismatch?",
                bytes.len()
            ));
        }
        let mut vals = bytes[..NET_BYTES].chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]]));
        // Allocate zeroed on the heap: the struct is too large for the stack.
        let mut net: Box<Network> = unsafe {
            let layout = std::alloc::Layout::new::<Network>();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut Network;
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            Box::from_raw(ptr)
        };
        for col in net.feature_weights.iter_mut() {
            for v in col.vals.iter_mut() {
                *v = vals.next().unwrap();
            }
        }
        for v in net.feature_bias.vals.iter_mut() {
            *v = vals.next().unwrap();
        }
        for v in net.output_weights.iter_mut() {
            *v = vals.next().unwrap();
        }
        net.output_bias = vals.next().unwrap();
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

/// Input feature index of `piece` on `sq`, seen from `perspective`: our
/// pieces come first, and Black's view is flipped vertically, so both
/// sides share the same weights.
#[inline(always)]
fn feature(perspective: usize, piece: Piece, sq: Square) -> usize {
    let (color, pt) = (piece_color(piece), piece_type(piece));
    if perspective == WHITE { (color * 6 + pt) * 64 + sq } else { ((color ^ 1) * 6 + pt) * 64 + flip(sq) }
}

/// Accumulators for both perspectives, indexed by colour.
#[derive(Clone, Copy)]
pub struct AccPair {
    pub acc: [Accumulator; 2],
}

impl AccPair {
    pub fn from_board(net: &Network, b: &Board) -> AccPair {
        let mut pair = AccPair { acc: [net.feature_bias; 2] };
        for sq in Bits(b.occupied()) {
            let p = b.piece_at(sq);
            for persp in [WHITE, BLACK] {
                let col = &net.feature_weights[feature(persp, p, sq)].vals;
                for (a, w) in pair.acc[persp].vals.iter_mut().zip(col) {
                    *a += *w;
                }
            }
        }
        pair
    }

    /// Compute the accumulators after `m` (played on `before`) from the
    /// parent's accumulators by adding/removing only the changed features.
    #[cfg(test)]
    pub fn update(&self, net: &Network, before: &Board, m: Move) -> AccPair {
        let mut out = *self;
        self.update_into(&mut out, net, before, m);
        out
    }

    /// Like `update`, but writes into `out` (no temporary copy).
    #[inline(always)]
    pub fn update_into(&self, out: &mut AccPair, net: &Network, before: &Board, m: Move) {
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
            let src = &self.acc[persp].vals;
            let dst = &mut out.acc[persp].vals;
            let col = |(p, sq): (Piece, Square)| &net.feature_weights[feature(persp, p, sq)].vals;
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

pub fn evaluate(net: &Network, pair: &AccPair, stm: usize) -> i32 {
    let mut out = screlu_dot(&pair.acc[stm].vals, &net.output_weights[..HIDDEN])
        + screlu_dot(&pair.acc[stm ^ 1].vals, &net.output_weights[HIDDEN..]);
    out /= QA;
    out += i32::from(net.output_bias);
    out * SCALE / (QA * QB)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::movegen::legal_moves;

    /// A random network in the bullet file layout, for testing.
    pub fn random_net_bytes(seed: u64) -> Vec<u8> {
        let mut s = seed;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let n = NET_BYTES / 2;
        let out_start = INPUTS * HIDDEN + HIDDEN;
        let mut bytes = Vec::with_capacity(NET_BYTES + 64);
        for i in 0..n {
            // Keep output weights within |w| < 128 as the trainer's clipping does.
            let v: i16 = if i >= out_start { (next() % 255) as i16 - 127 } else { (next() % 101) as i16 - 50 };
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        bytes.resize(NET_BYTES.div_ceil(64) * 64, 0);
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
                let f = feature(persp, b.piece_at(sq), sq);
                for i in 0..HIDDEN {
                    hidden[persp][i] += f64::from(net.feature_weights[f].vals[i]) / QA as f64;
                }
            }
        }
        let mut out = f64::from(net.output_bias) / (QA * QB) as f64;
        for (k, persp) in [b.stm, b.stm ^ 1].into_iter().enumerate() {
            for i in 0..HIDDEN {
                let v = hidden[persp][i].clamp(0.0, 1.0);
                out += v * v * f64::from(net.output_weights[k * HIDDEN + i]) / QB as f64;
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
        ];
        for fen in fens {
            let b = Board::from_fen(fen).unwrap();
            let pair = AccPair::from_board(&net, &b);
            for &m in legal_moves(&b).iter() {
                let child = b.make_move(m);
                let inc = pair.update(&net, &b, m);
                let fresh = AccPair::from_board(&net, &child);
                assert!(inc.acc[0].vals == fresh.acc[0].vals && inc.acc[1].vals == fresh.acc[1].vals, "{fen} {m}");
                let q = evaluate(&net, &inc, child.stm) as f64;
                let r = reference_eval(&net, &child);
                assert!((q - r).abs() <= 2.0 + r.abs() * 0.01, "quantised {q} vs reference {r}");
            }
        }
    }

    #[test]
    fn rejects_wrong_size() {
        assert!(Network::from_bytes(&[0u8; 1000]).is_err());
    }
}
