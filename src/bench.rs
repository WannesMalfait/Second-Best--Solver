use crate::eval;
use crate::movegen;
use crate::position::Move;
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
/// 2. `min_moves` is the minimum number of moves that need to played
///     for the position to be in the benchmark.
/// 3. `depth` gives a lower and upper bound on the depth needed to solve
///    the position.
///
/// The benchmark consists of lines with moves to be played.
pub fn generate_benchmark_file(
    abort: Arc<AtomicBool>,
    num_positions: usize,
    min_moves: usize,
    depth: Range<usize>,
) -> io::Result<()> {
    let mut positions = vec::Vec::with_capacity(num_positions);
    let mut counter = 0;
    while positions.len() < num_positions {
        counter += 1;
        println!("Generating position {}", positions.len());
        let mut solver = solver::Solver::new(abort.clone());
        let moves = generate_random_position(&mut solver, min_moves, &depth, counter);
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
    let file_name = format!("bench_{min_moves}_{}-{}", depth.start, depth.end);
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
    min_moves: usize,
    depth: &Range<usize>,
    mut seed: usize,
) -> Option<String> {
    if solver.abort_search() {
        return None;
    }

    if solver.position.num_moves() > min_moves as u8 + 40 {
        // Searching way too deep.
        return None;
    }
    if solver.position.num_moves() < min_moves as u8 {
        if solver.position.game_over() {
            // We are in a game over state, but not deep enough yet.
            return None;
        }
    } else {
        let eval = solver.search(depth.end);
        let eval = eval::decode_eval(&solver.position, eval);
        match eval {
            eval::ExplainableEval::Undetermined(_) => (),
            eval::ExplainableEval::Win(moves) | eval::ExplainableEval::Loss(moves) => {
                if moves >= depth.start as isize {
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
        if let Some(result) = generate_random_position(solver, min_moves, depth, seed) {
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
