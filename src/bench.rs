use crate::eval;
use crate::movegen;
use crate::position::Move;
use crate::position::Position;
use crate::solver;

use std::io::{self, Write};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::vec;

pub const BENCHMARKS_PATH: &str = "./benchmarks/";

/// Generate a benchmark file with the given specifications
/// 1. `num_positions` is the number of positions in the benchmark
/// 2. `moves` is the bounds on the number of moves that need to played
///     for the position to be in the benchmark.
/// 3. `depth` gives a lower and upper bound on the depth needed to solve
///    the position.
///
/// The benchmark consists of lines with moves to be played.
pub fn generate_benchmark_file(
    abort: Arc<AtomicBool>,
    num_positions: usize,
    moves_range: Range<usize>,
    depth_range: Range<usize>,
) -> io::Result<()> {
    let mut positions = vec::Vec::with_capacity(num_positions);
    let mut counter = 0;
    while positions.len() < num_positions {
        counter += 1;
        println!("Generating position {}", positions.len());
        let mut solver = solver::Solver::new(abort.clone());
        let moves = generate_random_position(&mut solver, &moves_range, &depth_range, counter);
        if abort.load(std::sync::atomic::Ordering::Relaxed) {
            println!("Stopping benchmark generation.");
            break;
        }
        let moves = moves.unwrap();
        if !positions.contains(&moves) {
            positions.push(moves);
        }
    }
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
    let path = path.join(&file_name);
    println!("Saved bench to {:?} ({} positions)", path, positions.len());
    let mut file = std::fs::File::create(path)?;
    file.write(positions.join("\n").as_bytes())?;
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

    if solver.position.num_moves() > moves_range.end as u8 {
        // Searching way too deep.
        return None;
    }
    if solver.position.num_moves() < moves_range.start as u8 {
        if solver.position.game_over() {
            // We are in a game over state, but not deep enough yet.
            return None;
        }
    } else {
        let eval = solver.search(depth_range.end);
        let eval = eval::decode_eval(&solver.position, eval);
        match eval {
            eval::ExplainableEval::Undetermined(_) => (),
            eval::ExplainableEval::Win(moves) | eval::ExplainableEval::Loss(moves) => {
                if moves >= depth_range.start as isize {
                    // Position is solvable in given depth.
                    return Some(solver.position.serialize());
                } else {
                    // Position is too easily solvable.
                    return None;
                }
            }
        }
    }
    // Generate a new move 'randomly'.
    let mut moves = movegen::MoveGen::new(&solver.position).collect::<vec::Vec<Move>>();
    let mut move_i;

    loop {
        if moves.is_empty() {
            return None;
        }
        (move_i, seed) = next_rand(seed);
        let smove = moves[move_i % moves.len()];
        solver.position.make_move(smove);
        if let Some(result) = generate_random_position(solver, moves_range, depth_range, seed) {
            return Some(result);
        }
        // Didn't work, try another move.
        solver.position.unmake_move();
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
        let params: vec::Vec<usize> = file_name
            .split("_")
            .flat_map(|s| s.split("-").map(|n| n.parse::<usize>().unwrap()))
            .collect();
        assert!(params.len() == 4);
        let min_moves = params[0];
        let max_moves = params[1];
        let min_depth = params[2];
        let max_depth = params[3];
        let file = std::fs::read_to_string(file.path())?;
        let positions: vec::Vec<_> = file.lines().collect();
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

            thread_handlers.push(std::thread::spawn(move || {
                let mut solver = solver::Solver::new(abort);
                let mut total_nodes = 0;
                let mut total_time = 0;
                for position in thread_positions {
                    solver.position = Position::default();
                    let moves = position.split_whitespace().map(|s| s.to_string()).collect();
                    solver.position.parse_and_play_moves(moves).unwrap();
                    let now = std::time::Instant::now();
                    solver.search(max_depth.clone());
                    if solver.abort_search() {
                        break;
                    }
                    total_time += now.elapsed().as_micros();
                    total_nodes += solver.nodes();
                }

                return (total_nodes, total_time);
            }))
        }

        let mut total_nodes = 0;
        let mut total_time = 0;
        for handler in thread_handlers {
            let (nodes, time) = handler.join().unwrap();
            total_nodes += nodes;
            total_time += time;
        }
        println!("Finished benchmark:\n\
            Average time: {}s\n\
            Average number of nodes searched: {}\n\
            Average knps: {} knps\n",
        total_time as f64 / 1_000_000.0 / positions.len() as f64,
        total_nodes as f64 / positions.len() as f64,
        total_nodes as f64 * 1000. / total_time as f64
        );
    }
    Ok(())
}
