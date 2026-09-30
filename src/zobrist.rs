//! Zobrist hashing keys, generated at compile time from a fixed seed.

pub struct Keys {
    pub pieces: [[u64; 64]; 12],
    pub castling: [u64; 16],
    pub ep_file: [u64; 8],
    pub black_to_move: u64,
}

const fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

const fn generate() -> Keys {
    let mut s = 0x7121_17A0_5EED_0001u64;
    let mut keys = Keys { pieces: [[0; 64]; 12], castling: [0; 16], ep_file: [0; 8], black_to_move: 0 };
    let mut p = 0;
    while p < 12 {
        let mut sq = 0;
        while sq < 64 {
            keys.pieces[p][sq] = splitmix(&mut s);
            sq += 1;
        }
        p += 1;
    }
    // Castling keys are XOR-combinations of four independent per-right keys,
    // so a hash can be updated by XORing out the old rights and in the new.
    let rights = [splitmix(&mut s), splitmix(&mut s), splitmix(&mut s), splitmix(&mut s)];
    let mut c = 0;
    while c < 16 {
        let mut k = 0;
        let mut bit = 0;
        while bit < 4 {
            if c & (1 << bit) != 0 {
                k ^= rights[bit];
            }
            bit += 1;
        }
        keys.castling[c] = k;
        c += 1;
    }
    let mut f = 0;
    while f < 8 {
        keys.ep_file[f] = splitmix(&mut s);
        f += 1;
    }
    keys.black_to_move = splitmix(&mut s);
    keys
}

pub static KEYS: Keys = generate();
