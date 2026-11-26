//! Trinity chess engine.

mod attacks;
mod bench;
mod board;
mod datagen;
mod eval;
mod forensics;
mod movegen;
mod nnue;
mod search;
mod select;
mod similarity;
mod tb;
mod timeuse;
mod tt;
mod tune;
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

/// TEST BUILD ONLY (branch tb-on-test, never merged). The approved test
/// script passes no SyzygyPath, so this build switches the tablebases on by
/// itself: the folder `syzygy` three levels above the program, which is
/// where the scripts keep it (<folder>\sprt\engines\<this file>, tables in
/// <folder>\syzygy). It measures "tables on" against main's "tables off".
/// If the tables cannot be loaded the engine refuses to play, so the test
/// can never silently compare two equal engines. `bench` is not affected.
fn default_tables() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot find my own path: {e}"))?;
    let dir = exe
        .parent()
        .and_then(|d| d.parent())
        .and_then(|d| d.parent())
        .map(|d| d.join("syzygy"))
        .ok_or_else(|| "no folder three levels above the program".to_string())?;
    if !dir.is_dir() {
        return Err(format!("no tablebase folder at {}", dir.display()));
    }
    let n = tb::set_path(&dir.to_string_lossy())?;
    if n == 0 || tb::max_pieces() < 5 {
        return Err(format!("only {n} tables (up to {} pieces) in {}", tb::max_pieces(), dir.display()));
    }
    println!("info string tb-on-test: {n} tables loaded (up to {} pieces)", tb::max_pieces());
    Ok(())
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
        // `trinity similarity <positions> <movetime ms> <name=path>...`
        Some("similarity") => {
            let count = args.get(1).and_then(|c| c.parse().ok()).unwrap_or(1000);
            let movetime = args.get(2).and_then(|t| t.parse().ok()).unwrap_or(100);
            if let Err(e) = similarity::run(count, movetime, &args[3.min(args.len())..]) {
                eprintln!("{e}");
                return 1;
            }
        }
        // `trinity forensics <max losses> <workers> <pgn files...>`
        Some("forensics") => {
            let max = args.get(1).and_then(|c| c.parse().ok()).unwrap_or(200);
            let workers = args.get(2).and_then(|t| t.parse().ok()).unwrap_or(4);
            if let Err(e) = forensics::run(max, workers, &args[3.min(args.len())..]) {
                eprintln!("{e}");
                return 1;
            }
        }
        // `trinity timeuse <pgn files...>`
        Some("timeuse") => {
            if let Err(e) = timeuse::run(&args[1..]) {
                eprintln!("{e}");
                return 1;
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
        _ => {
            // TEST BUILD ONLY (branch tb-on-test, never merged; Scout, 26
            // November): play with the tablebases on without being told.
            if let Err(e) = default_tables() {
                println!("info string tb-on-test: {e}");
                eprintln!("tb-on-test: {e}");
                return 1;
            }
            uci::Engine::new().run()
        }
    }
    0
}
