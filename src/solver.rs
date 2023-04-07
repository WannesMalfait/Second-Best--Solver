use crate::eval;
use crate::movegen;
use crate::position::BitboardMove;
use crate::position::Position;
use std::cmp::max;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time;

pub struct Solver {
    pub position: Position,
    nodes: usize,
    abort: Arc<AtomicBool>,
    /// If true, don't print anything to stdout.
    quiet: bool,
    pv_table: [PV; Position::MAX_MOVES + 1],
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            position: Position::default(),
            nodes: 0,
            abort: Arc::new(AtomicBool::new(false)),
            quiet: true,
            pv_table: [PV::default(); Position::MAX_MOVES + 1],
        }
    }
}

/// Stores the Principal Variation at a given ply.
#[derive(Debug, Clone, Copy)]
struct PV {
    // Should technically be two times as big, but 128 moves
    // would be a very big pv.
    pv: [Option<BitboardMove>; Position::MAX_MOVES + 1],
    pv_len: usize,
}

impl Default for PV {
    fn default() -> Self {
        Self {
            pv: [None; Position::MAX_MOVES + 1],
            pv_len: 0,
        }
    }
}

impl PV {
    pub fn update_pv(&mut self, best_move: BitboardMove, child_pv: &[Option<BitboardMove>]) {
        self.pv[0] = Some(best_move);
        for (pv, &child) in self.pv[1..].iter_mut().zip(child_pv) {
            *pv = child;
        }
        self.pv_len = child_pv.len() + 1;
    }
}

impl Solver {
    pub fn new(abort: Arc<AtomicBool>) -> Self {
        Solver {
            abort,
            ..Default::default()
        }
    }

    pub fn nodes(&self) -> usize {
        self.nodes
    }

    fn update_pv(&mut self, pv_index: usize, best_move: BitboardMove) {
        let child = self.pv_table[pv_index + 1];
        self.pv_table[pv_index].update_pv(best_move, &child.pv[..child.pv_len]);
    }

    fn update_pv_depth_one(&mut self, pv_index: usize, best_move: BitboardMove) {
        self.pv_table[pv_index].pv[0] = Some(best_move);
        self.pv_table[pv_index].pv_len = 1;
    }

    fn get_pv_move(&self, pv_index: usize) -> Option<BitboardMove> {
        if pv_index < self.pv_table[0].pv_len {
            self.pv_table[0].pv[pv_index]
        } else {
            None
        }
    }

    /// Do an alpha beta negamax search on the current position.
    /// Returns the score of the current position.
    fn negamax(
        &mut self,
        leftmost: bool,
        depth: usize,
        pv_index: usize,
        mut alpha: isize,
        beta: isize,
    ) -> isize {
        // Don't check this every node, but often often enough.
        if self.nodes % 1024 == 0 && self.abort_search() {
            // Have to stop the search now.
            return 0;
        }

        self.nodes += 1;
        if self.position.game_over() {
            return eval::loss_score(self.position.num_moves() as isize);
        }
        if depth == 0 {
            // Return a static evaluation of the position.
            let eval = eval::static_eval(&self.position);
            return eval;
        }

        // Set the best score to the minimal value at first.
        // Worst case is that we lose next move.
        let mut best_score = eval::loss_score(self.position.num_moves() as isize + 1);

        let pv_move = match leftmost {
            true => self.get_pv_move(pv_index),
            false => None,
        };

        // Look at the child nodes:
        let moves = movegen::MoveGen::new(&self.position, pv_move);
        for (i, bmove) in moves.enumerate() {
            match bmove {
                BitboardMove::SecondBest => {
                    self.position.second_best();
                    best_score = max(
                        best_score,
                        -self.negamax(leftmost && i == 0, depth, pv_index + 1, -beta, -alpha),
                    );
                    self.position.undo_second_best();
                }
                BitboardMove::StoneMove(smove) => {
                    // Enable for testing purposes.
                    // self.position
                    //     .try_make_move(bmove.to_player_move(&self.position))
                    //     .unwrap();
                    self.position.make_stone_move(smove);
                    best_score = max(
                        best_score,
                        -self.negamax(leftmost && i == 0, depth - 1, pv_index + 1, -beta, -alpha),
                    );
                    self.position.unmake_stone_move();
                }
            }
            if best_score > alpha {
                alpha = best_score;
                if depth == 1 && bmove != BitboardMove::SecondBest {
                    // Special case here to make sure we don't accidentally pick up
                    // a tail from a previous iteration.
                    self.update_pv_depth_one(pv_index, bmove);
                } else {
                    self.update_pv(pv_index, bmove);
                }
                if alpha >= beta {
                    break;
                }
            }
        }
        best_score
    }

    /// Returns whether the search is being aborted.
    pub fn abort_search(&self) -> bool {
        self.abort.load(Ordering::Relaxed)
    }

    pub fn be_quiet(&mut self) {
        self.quiet = true
    }

    pub fn be_noisy(&mut self) {
        self.quiet = false
    }

    pub fn search(&mut self, depth: usize) -> isize {
        self.nodes = 0;
        self.pv_table = [PV::default(); Position::MAX_MOVES + 1];
        let mut eval = 0;
        let start = time::Instant::now();
        for depth in 1..=depth {
            let new_eval = self.negamax(true, depth, 0, eval::LOSS, eval::WIN);
            if self.abort_search() {
                return eval;
            }
            eval = new_eval;
            if !self.quiet {
                let elapsed = start.elapsed();
                let nodes = self.nodes;
                let knps = self.nodes as u128 / (1 + elapsed.as_millis());
                println!(
                    "info depth {depth} score {eval} nodes {nodes} knps {knps} ({:?} total time)",
                    elapsed
                );

                print!("best move");
                if let Some(pv_move) = self.get_pv_move(0) {
                    print!(" {}", pv_move.to_string(&self.position));
                }
                // Skip for now, as there seems to be some bug.

                // print!("pv");
                // let mut pv_i = 0;
                // while let Some(pv_move) = self.get_pv_move(pv_i) {
                //     pv_i += 1;
                //     // self.position.show();
                //     print!(" {}", pv_move.to_string(&self.position));
                //     self.position.make_move(pv_move);
                // }
                // // Set position back to original state.
                // for _ in 0..pv_i {
                //     self.position.unmake_move();
                // }
                println!();
            }
            match eval::decode_eval(self.position.num_moves() as isize, eval) {
                eval::ExplainableEval::Win(_) | eval::ExplainableEval::Loss(_) => {
                    break;
                }
                _ => continue,
            }
        }
        eval
    }
}
