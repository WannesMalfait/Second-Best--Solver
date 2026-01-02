use engine::eval;
use engine::eval::ExplainableEval;
use engine::movegen;
use engine::position::BitboardMove;
use engine::position::GameStatus;
use engine::position::Position;
use engine::solver;

use std::io::{self, Write};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::vec::Vec;

pub const BENCHMARKS_PATH: &str = "./benchmarks/";

/// Generate a benchmark file with the given specifications
/// 1. `num_positions` is the number of positions in the benchmark
/// 2. `num_threads` each thread runs a separate solver generating positions
/// 3. `moves` is the bounds on the number of moves that need to played
///    for the position to be in the benchmark.
/// 4. `depth` gives a lower and upper bound on the depth needed to solve
///    the position.
///
/// The benchmark consists of lines with moves to be played.
pub fn generate_benchmark_file(
    abort: Arc<AtomicBool>,
    num_positions: usize,
    num_threads: usize,
    moves_range: Range<usize>,
    depth_range: Range<usize>,
) -> io::Result<()> {
    let counter = Arc::new(AtomicUsize::new(1));
    let generated_positions = Arc::new(Mutex::new(Vec::with_capacity(num_positions)));
    let num_generated_positions = Arc::new(AtomicUsize::new(0));
    let mut thread_handlers = vec![];
    for thread_id in 0..num_threads {
        let abort = abort.clone();
        let counter = counter.clone();
        let num_generated_positions = num_generated_positions.clone();
        let generated_positions = generated_positions.clone();
        let moves_range = moves_range.clone();
        let depth_range = depth_range.clone();
        let main_thread = thread_id == 0;

        thread_handlers.push(
            std::thread::Builder::new()
                .name(thread_id.to_string())
                .stack_size(5_000_000)
                .spawn(move || {
                    while num_generated_positions.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                        < num_positions
                    {
                        let seed = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        if main_thread {
                            // Some thread is generating this position right now.
                            print!(
                                "\rGenerating position {}",
                                num_generated_positions.load(std::sync::atomic::Ordering::Relaxed)
                            );
                            io::stdout().flush().unwrap();
                        }
                        let mut solver = solver::Solver::new(abort.clone());
                        let moves =
                            generate_random_position(&mut solver, &moves_range, &depth_range, seed);
                        if abort.load(std::sync::atomic::Ordering::Relaxed) {
                            if main_thread {
                                println!("\nStopping benchmark generation.");
                            }
                            break;
                        }
                        let moves = moves.unwrap();
                        let mut generated_positions = generated_positions.lock().unwrap();
                        if !generated_positions.contains(&moves) {
                            generated_positions.push(moves);
                        } else {
                            num_generated_positions
                                .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                        }
                    }
                }),
        );
    }

    for handler in thread_handlers {
        handler?.join().unwrap();
    }
    let mut positions = generated_positions.lock().unwrap();
    // Ensure that the output is stable.
    positions.sort();
    println!();
    if positions.is_empty() {
        // Don't create the file if nothing was generated.
        println!("No benchmarks generated.");
        return Ok(());
    }
    let file_name = format!(
        "bench_{}-{}_{}-{}",
        moves_range.start, moves_range.end, depth_range.start, depth_range.end
    );
    let path = PathBuf::from(BENCHMARKS_PATH);
    std::fs::create_dir_all(&path)?;
    let path = path.join(file_name);
    println!("Saved bench to {:?} ({} positions)", path, positions.len());
    let mut file = std::fs::File::create(path)?;
    file.write_all(positions.join("\n").as_bytes())?;
    Ok(())
}

/// Recursive utility function to generate a random position
/// satisfying the given parameters.
fn generate_random_position(
    solver: &mut solver::Solver,
    moves_range: &Range<usize>,
    depth_range: &Range<usize>,
    mut seed: usize,
) -> Option<String> {
    if solver.abort_search() {
        return None;
    }

    if solver.position.num_turns() > moves_range.end {
        // Searching way too deep.
        return None;
    }
    if solver.position.num_turns() < moves_range.start {
        if solver.position.game_status() != GameStatus::OnGoing {
            // We are in a game over state, but not deep enough yet.
            return None;
        }
    } else {
        let eval = solver.search(depth_range.end);
        let eval = eval.decode_eval();
        match eval {
            eval::ExplainableEval::Undetermined(_) => (),
            eval::ExplainableEval::Win(moves) | eval::ExplainableEval::Loss(moves) => {
                if depth_range.start <= moves && depth_range.end >= moves {
                    // Position is solvable in given depth.
                    return Some(format!("{moves};") + &solver.position.clone().serialize());
                } else {
                    // Position is too easily solvable.
                    return None;
                }
            }
        }
    }
    // Generate a new move 'randomly'.
    let mut moves = movegen::MoveGen::new(&solver.position, None).collect::<Vec<_>>();
    let mut move_i;

    loop {
        if moves.is_empty() {
            return None;
        }
        (move_i, seed) = next_rand(seed);
        let smove = moves[move_i % moves.len()];
        let smove = match smove {
            BitboardMove::SecondBest => {
                moves.swap_remove(move_i % moves.len());
                continue;
            }
            BitboardMove::StoneMove(smove) => smove,
        };
        solver.position.make_stone_move(smove);
        if let Some(result) = generate_random_position(solver, moves_range, depth_range, seed) {
            return Some(result);
        }
        // Didn't work, try another move.
        solver.position.unmake_stone_move();
        moves.swap_remove(move_i % moves.len());
    }
}

/// Generate a pseudo-random number for the given seed.
/// The generated number and a new seed are returned.
fn next_rand(seed: usize) -> (usize, usize) {
    let a = 1103515245;
    let c = 12345;
    let m = 1 << 31;
    let seed = (a * seed + c) % m;
    (seed >> 4, seed)
}

/// Run all the benchmarks and print statistics.
/// To make the benchmark run faster, the work can be spread
/// over multiple threads. Each position is still assigned to
/// a unique thread.
pub fn run_benchmarks(abort: Arc<AtomicBool>, num_threads: usize) -> io::Result<()> {
    let files = std::fs::read_dir(BENCHMARKS_PATH)?;
    for file in files {
        let file = file?;
        let file_name = file.file_name().into_string().unwrap();
        if !file_name.starts_with("bench") {
            continue;
        }
        let file_name = file_name.strip_prefix("bench_").unwrap();
        let params: Vec<usize> = file_name
            .split('_')
            .flat_map(|s| s.split('-').map(|n| n.parse::<usize>().unwrap()))
            .collect();
        assert!(params.len() == 4);
        let min_moves = params[0];
        let max_moves = params[1];
        let min_depth = params[2];
        let max_depth = params[3];
        let file = std::fs::read_to_string(file.path())?;
        let positions: Vec<_> = file.lines().collect();
        println!(
            "\nStarting benchmark with {} positions.\n\
            number of moves: {min_moves}..{max_moves}\n\
            solution depth: {min_depth}..{max_depth}\n",
            positions.len()
        );
        let mut thread_handlers = vec![];
        for thread_id in 0..num_threads {
            let mut thread_positions = vec![];
            for position_id in (thread_id..(positions).len()).step_by(num_threads) {
                thread_positions.push(positions[position_id].to_string());
            }
            let abort = abort.clone();
            let main_thread = thread_id == 0;

            thread_handlers.push(
                std::thread::Builder::new()
                    .name(thread_id.to_string())
                    .stack_size(5_000_000)
                    .spawn(move || {
                        let mut solver = solver::Solver::new(abort);
                        let mut total_nodes = 0;
                        let mut total_time = 0;
                        for (i, position) in thread_positions.iter().enumerate() {
                            if main_thread {
                                print!(
                                    "\rRunning benchmark: {:.2}%",
                                    (i as f64 + 1.0) / thread_positions.len() as f64 * 100.
                                );
                                io::stdout().flush().unwrap();
                            }
                            solver.reset();
                            solver.position = Position::default();
                            let (num_moves_sol, moves) = position.split_once(';').unwrap();
                            let num_moves_sol = num_moves_sol.parse().unwrap();
                            let moves = moves.split_whitespace().map(|s| s.to_string()).collect();
                            solver.position.parse_and_play_moves(moves).unwrap();
                            let now = std::time::Instant::now();
                            // Add extra depth, in case the solver needs it.
                            let eval = solver.search(max_depth);
                            // Sanity check to make sure we actually solved the position.
                            match eval.decode_eval() {
                                ExplainableEval::Win(num_moves)
                                | ExplainableEval::Loss(num_moves) => {
                                    if num_moves != num_moves_sol {
                                        println!("\nFailed position {position}\nExpected to solve in {num_moves_sol} but solved in {num_moves}");
                                        break;
                                    }
                                }
                                ExplainableEval::Undetermined(_) => {
                                    println!("\nFailed to solve position {position}, ran at depth {max_depth}");
                                    break;
                                }
                            }
                            if solver.abort_search() {
                                break;
                            }
                            total_time += now.elapsed().as_micros();
                            total_nodes += solver.nodes();
                        }
                        if main_thread {
                            // Add a newline after the progress print
                            println!("\nWaiting for all threads to finish...\n");
                        }

                        (total_nodes, total_time)
                    }),
            )
        }

        let mut total_nodes = 0;
        let mut total_time = 0;
        for handler in thread_handlers {
            let (nodes, time) = handler?.join().unwrap();
            total_nodes += nodes;
            total_time += time;
        }
        println!(
            "Finished benchmark:\n\
            Average time: {:.4}s\n\
            Average number of nodes searched: {:.2}\n\
            Average knps: {:.2} knps\n",
            total_time as f64 / 1_000_000.0 / positions.len() as f64,
            total_nodes as f64 / positions.len() as f64,
            total_nodes as f64 * 1000. / total_time as f64
        );
    }
    Ok(())
}
