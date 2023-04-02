use crate::eval;
use crate::movegen;
use crate::position::Position;
use std::cmp::max;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time;

#[derive(Default)]
pub struct Solver {
    pub position: Position,
    nodes: usize,
    abort: Arc<AtomicBool>,
}

impl Solver {
    pub fn new(abort: Arc<AtomicBool>) -> Self {
        Solver {
            position: Position::default(),
            nodes: 0,
            abort,
        }
    }

    /// Do an alpha beta negamax search on the current position.
    /// Returns the score of the current position.
    fn negamax(&mut self, depth: usize, mut alpha: isize, beta: isize) -> isize {
        // Don't check this every node, but often often enough.
        if self.nodes % 1024 == 0 && self.abort.load(Ordering::Relaxed) {
            // Have to stop the search now.
            return 0;
        }

        self.nodes += 1;
        if self.position.game_over() {
            // The position is lost, so return the lowest possible score.
            return eval::loss_score(&self.position);
        }

        if depth == 0 {
            // Return a static evaluation of the position.
            let eval = eval::static_eval(&self.position);
            return eval;
        }

        // Set the best score to the minimal value at first.
        let mut best_score = eval::loss_score(&self.position);

        // Look at the child nodes:

        // First try to do "Second Best!"
        if self.position.can_second_best() {
            self.position.second_best();
            best_score = max(best_score, -self.negamax(depth, -beta, -alpha));
            alpha = max(alpha, best_score);
            self.position.undo_second_best();
            if alpha >= beta {
                return best_score;
            }
        }
        // Now try the other possible moves.
        let moves = movegen::MoveGen::new(&self.position);
        for smove in moves {
            self.position.make_move(smove);
            best_score = max(best_score, -self.negamax(depth - 1, -beta, -alpha));
            self.position.unmake_move();
            alpha = max(alpha, best_score);
            if alpha >= beta {
                break;
            }
        }
        best_score
    }

    pub fn search(&mut self, depth: usize) -> isize {
        self.nodes = 0;
        self.abort.store(false, Ordering::Relaxed);
        let start = time::Instant::now();
        let score = self.negamax(depth, eval::LOSS, eval::WIN);
        if self.abort.load(Ordering::Relaxed) {
            println!("Search aborted.\nScore is unreliable.\n");
            return score;
        }
        let elapsed = start.elapsed();
        let nodes = self.nodes;
        let knps = self.nodes as u128 / (1 + elapsed.as_millis());
        println!("Searched {nodes} nodes in {:?} ({knps} kn/s)", elapsed);
        score
    }
}
