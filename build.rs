//! Build script: finds "magic numbers" for sliding-piece attack lookup and
//! writes the resulting tables into OUT_DIR, so the engine never has to
//! compute them at startup.
//!
//! Squares are numbered a1 = 0, b1 = 1, ..., h8 = 63.

use std::env;
use std::fs;
use std::path::Path;

const ROOK_DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const BISHOP_DIRS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

/// Attacks from `sq` along `dirs`, stopping at (and including) blockers.
fn slow_attacks(sq: usize, occ: u64, dirs: &[(i32, i32)]) -> u64 {
    let mut result = 0;
    let (f0, r0) = ((sq % 8) as i32, (sq / 8) as i32);
    for &(df, dr) in dirs {
        let (mut f, mut r) = (f0 + df, r0 + dr);
        while (0..8).contains(&f) && (0..8).contains(&r) {
            let bit = 1u64 << (r * 8 + f);
            result |= bit;
            if occ & bit != 0 {
                break;
            }
            f += df;
            r += dr;
        }
    }
    result
}

/// Relevant-occupancy mask: the squares whose occupancy can change the
/// attack set (board edges in the direction of travel are irrelevant).
fn relevant_mask(sq: usize, dirs: &[(i32, i32)]) -> u64 {
    let mut result = 0;
    let (f0, r0) = ((sq % 8) as i32, (sq / 8) as i32);
    for &(df, dr) in dirs {
        let (mut f, mut r) = (f0 + df, r0 + dr);
        loop {
            let (nf, nr) = (f + df, r + dr);
            if !(0..8).contains(&nf) || !(0..8).contains(&nr) {
                break;
            }
            result |= 1u64 << (r * 8 + f);
            f = nf;
            r = nr;
        }
    }
    result
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn sparse(&mut self) -> u64 {
        self.next() & self.next() & self.next()
    }
}

/// Enumerate every subset of `mask` (Carry-Rippler trick).
fn subsets(mask: u64) -> Vec<u64> {
    let mut out = Vec::with_capacity(1 << mask.count_ones());
    let mut sub = 0u64;
    loop {
        out.push(sub);
        sub = sub.wrapping_sub(mask) & mask;
        if sub == 0 {
            break;
        }
    }
    out
}

struct Found {
    mask: u64,
    magic: u64,
    shift: u32,
    offset: usize,
}

fn find_magics(dirs: &[(i32, i32)], rng: &mut Rng, table: &mut Vec<u64>) -> Vec<Found> {
    let mut found = Vec::with_capacity(64);
    for sq in 0..64 {
        let mask = relevant_mask(sq, dirs);
        let bits = mask.count_ones();
        let shift = 64 - bits;
        let occs = subsets(mask);
        let atts: Vec<u64> = occs.iter().map(|&o| slow_attacks(sq, o, dirs)).collect();
        let size = 1usize << bits;
        let mut slots = vec![0u64; size];
        let mut used = vec![0u32; size];
        let mut attempt = 0u32;
        let magic = 'search: loop {
            attempt += 1;
            let magic = rng.sparse();
            if (mask.wrapping_mul(magic) >> 56).count_ones() < 6 {
                continue;
            }
            for (&o, &a) in occs.iter().zip(&atts) {
                let idx = (o.wrapping_mul(magic) >> shift) as usize;
                if used[idx] != attempt {
                    used[idx] = attempt;
                    slots[idx] = a;
                } else if slots[idx] != a {
                    continue 'search;
                }
            }
            break magic;
        };
        let offset = table.len();
        table.resize(offset + size, 0);
        for (&o, &a) in occs.iter().zip(&atts) {
            let idx = (o.wrapping_mul(magic) >> shift) as usize;
            table[offset + idx] = a;
        }
        found.push(Found { mask, magic, shift, offset });
    }
    found
}

fn write_entries(out: &mut String, name: &str, entries: &[Found]) {
    out.push_str(&format!("pub(crate) const {name}: [Magic; 64] = [\n"));
    for e in entries {
        out.push_str(&format!(
            "    Magic {{ mask: {:#018x}, magic: {:#018x}, shift: {}, offset: {} }},\n",
            e.mask, e.magic, e.shift, e.offset
        ));
    }
    out.push_str("];\n");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=EVALFILE");
    println!("cargo::rustc-check-cfg=cfg(trinity_net)");
    println!("cargo::rustc-check-cfg=cfg(trinity_hidden_1024)");

    let out_dir = env::var("OUT_DIR").unwrap();
    let mut rng = Rng(0x7E1A_17C0_FFEE_1234);
    let mut table = Vec::new();
    let rooks = find_magics(&ROOK_DIRS, &mut rng, &mut table);
    let bishops = find_magics(&BISHOP_DIRS, &mut rng, &mut table);

    let mut src = String::new();
    src.push_str(&format!("pub(crate) const SLIDER_TABLE_LEN: usize = {};\n", table.len()));
    write_entries(&mut src, "ROOK_MAGICS", &rooks);
    write_entries(&mut src, "BISHOP_MAGICS", &bishops);
    fs::write(Path::new(&out_dir).join("magics.rs"), src).unwrap();

    let bytes: Vec<u8> = table.iter().flat_map(|x| x.to_le_bytes()).collect();
    fs::write(Path::new(&out_dir).join("slider_attacks.bin"), bytes).unwrap();

    // Neural network: embed `EVALFILE` (default: `nets/default.nnue`) if it
    // exists. Without one the engine uses its hand-written evaluation.
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let net_path = env::var("EVALFILE")
        .map(|p| Path::new(&p).to_path_buf())
        .unwrap_or_else(|_| Path::new(&manifest).join("nets").join("default.nnue"));
    println!("cargo:rerun-if-changed={}", net_path.display());
    if net_path.is_file() {
        fs::copy(&net_path, Path::new(&out_dir).join("net.bin")).unwrap();
        println!("cargo:rustc-cfg=trinity_net");
        // The hidden layer size is fixed at compile time; pick it from the
        // size of the network file (1024 neurons, 8 king buckets, 8 output
        // buckets: see src/nnue.rs).
        let wide = 2 * (8 * 768 * 1024 + 1024 + 8 * (2 * 1024 + 1));
        let len = fs::metadata(&net_path).unwrap().len() as usize;
        if (wide..wide + 64).contains(&len) {
            println!("cargo:rustc-cfg=trinity_hidden_1024");
        }
    }
}
