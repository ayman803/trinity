//! Search settings that the SPSA tuner may adjust (see
//! `.github/workflows/spsa.yml`). In a normal build each one is a constant.
//! Built with `--features tune`, each is also a UCI option, so the tuner can
//! play games with different values.
//!
//! Every entry: name, default, minimum, maximum, and the SPSA step size
//! (`c_end`: how far a value is moved when testing it).

macro_rules! tunables {
    ($($name:ident = $default:expr, $min:expr, $max:expr, $step:expr;)*) => {
        #[cfg(feature = "tune")]
        mod values {
            use std::sync::atomic::AtomicI32;
            $(pub static $name: AtomicI32 = AtomicI32::new($default);)*
        }

        $(
            #[inline(always)]
            #[allow(non_snake_case)]
            pub fn $name() -> i32 {
                #[cfg(feature = "tune")]
                {
                    values::$name.load(std::sync::atomic::Ordering::Relaxed)
                }
                #[cfg(not(feature = "tune"))]
                {
                    $default
                }
            }
        )*

        /// (name, default, min, max, step) of every tunable setting.
        #[cfg_attr(not(feature = "tune"), allow(dead_code))]
        pub const LIST: &[(&str, i32, i32, i32, f64)] = &[$((stringify!($name), $default, $min, $max, $step as f64)),*];

        /// Set a value by its UCI option name; false if there is no such setting.
        #[cfg(feature = "tune")]
        pub fn set(name: &str, value: i32) -> bool {
            $(
                if name.eq_ignore_ascii_case(stringify!($name)) {
                    values::$name.store(value.clamp($min, $max), std::sync::atomic::Ordering::Relaxed);
                    return true;
                }
            )*
            false
        }
    };
}

tunables! {
    // Reverse futility pruning margin per ply.
    RFP_MARGIN = 81, 30, 150, 8;
    // Razoring: margin = base + mul * depth.
    RAZOR_BASE = 209, 50, 400, 20;
    RAZOR_MUL = 266, 100, 450, 20;
    // Null move: one more ply of reduction per this much eval above beta.
    NMP_EVAL_DIV = 200, 80, 400, 20;
    // Late move pruning: quiet moves allowed = (base + mul * depth^2) / 100,
    // halved when not improving.
    LMP_BASE = 349, 100, 800, 40;
    LMP_MUL = 101, 50, 200, 8;
    // Futility pruning: margin = base + mul * depth.
    FUT_BASE = 91, 30, 200, 10;
    FUT_MUL = 102, 50, 200, 8;
    // SEE pruning thresholds: quiet moves -x * depth^2, captures -x * depth.
    SEE_QUIET = 29, 10, 80, 4;
    SEE_NOISY = 89, 40, 180, 8;
    // Singular extension margin, in 16ths of a centipawn per ply.
    SE_MUL = 16, 8, 48, 2;
    // Late move reductions: base and divisor (in hundredths) of the
    // ln(depth) * ln(moves) formula, and the history that cancels a ply.
    LMR_NOISY_BASE = 19, 0, 80, 5;
    LMR_NOISY_DIV = 337, 200, 500, 15;
    LMR_QUIET_BASE = 80, 30, 150, 6;
    LMR_QUIET_DIV = 225, 150, 350, 10;
    LMR_HIST_DIV = 8006, 4000, 16000, 600;
    // Quiescence search futility margin.
    QS_FUTILITY = 162, 50, 300, 12;
    // First aspiration window half-width.
    ASP_DELTA = 20, 8, 50, 2;
    // History bonus: min(mul * depth - sub, max).
    HIST_MUL = 172, 80, 300, 10;
    HIST_SUB = 79, 0, 200, 10;
    HIST_MAX = 1698, 800, 3000, 100;
}
