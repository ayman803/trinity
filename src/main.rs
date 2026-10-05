//! Trinity chess engine.

mod attacks;
mod bench;
mod board;
mod datagen;
mod eval;
mod movegen;
mod nnue;
mod search;
mod select;
mod tb;
mod tt;
mod types;
mod uci;
mod zobrist;

fn main() {
    // Run everything on a thread with a large stack: Windows gives the main
    // thread only 1 MB, which deep searches can exceed.
    let child =
        std::thread::Builder::new().stack_size(uci::STACK_SIZE).spawn(real_main).expect("failed to start main thread");
    let code = child.join().unwrap_or(1);
    std::process::exit(code);
}

fn real_main() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        // `trinity bench [depth]` — used by test frameworks; prints
        // "<nodes> nodes <nps> nps".
        Some("bench") => {
            let depth = args.get(1).and_then(|d| d.parse().ok()).unwrap_or(bench::DEFAULT_DEPTH);
            bench::run(depth);
        }
        // `trinity perft <depth> [fen]` — move generator check.
        Some("perft") => {
            let depth = args.get(1).and_then(|d| d.parse().ok()).unwrap_or(5);
            let fen = if args.len() > 2 { args[2..].join(" ") } else { board::START_FEN.to_string() };
            match board::Board::from_fen(&fen) {
                Ok(b) => {
                    let start = std::time::Instant::now();
                    let nodes = movegen::perft(&b, depth);
                    let secs = start.elapsed().as_secs_f64().max(1e-9);
                    println!("perft {depth}: {nodes} nodes in {secs:.2} s ({:.0} Mnps)", nodes as f64 / secs / 1e6);
                }
                Err(e) => {
                    eprintln!("{e}");
                    return 1;
                }
            }
        }
        Some("datagen") => match datagen::Config::from_args(&args[1..]) {
            Ok(cfg) => datagen::run(cfg),
            Err(e) => {
                eprintln!("{e}");
                eprintln!("usage: trinity datagen threads=N positions=N nodes=N out=DIR");
                return 1;
            }
        },
        // `trinity select <in.data> <out.data> [fraction] [threads]` — build
        // the "drill" training set (see select.rs).
        Some("select") if args.len() >= 3 => {
            let fraction = args.get(3).and_then(|f| f.parse().ok()).unwrap_or(0.2);
            let threads = args
                .get(4)
                .and_then(|t| t.parse().ok())
                .unwrap_or_else(|| std::thread::available_parallelism().map_or(4, |n| n.get()));
            if let Err(e) = select::run(&args[1], &args[2], fraction, threads) {
                eprintln!("{e}");
                return 1;
            }
        }
        _ => uci::Engine::new().run(),
    }
    0
}
