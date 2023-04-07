use crate::eval;
use crate::movegen;
use crate::position::BitboardMove;
use crate::position::Position;
use crate::transposition_table::EntryType;
use crate::transposition_table::TranspositionTable;
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
    t_table: TranspositionTable,
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            position: Position::default(),
            nodes: 0,
            abort: Arc::new(AtomicBool::new(false)),
            quiet: true,
            t_table: TranspositionTable::default(),
        }
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

    /// Do an alpha beta negamax search on the current position.
    /// Returns the score of the current position.
    fn negamax(&mut self, depth: usize, ply: usize, mut alpha: isize, beta: isize) -> isize {
        // Don't check this every node, but often often enough.
        if self.nodes % 1024 == 0 && self.abort_search() {
            // Have to stop the search now.
            return 0;
        }

        self.nodes += 1;
        if self.position.game_over() {
            return eval::loss_score(ply as isize);
        }
        if depth == 0 {
            // Return a static evaluation of the position.
            let eval = eval::static_eval(&self.position);
            return eval;
        }

        let initial_alpha = alpha;

        let mut best_move = None;
        if let Some(tt_entry) = self.t_table.get(&self.position) {
            best_move = Some(tt_entry.best_move(&self.position));
            // TODO: Figure out how to correctly use stored scores.
            // let score = tt_entry.score();
            // match tt_entry.entry_type() {
            //     EntryType::Undetermined => (),
            //     EntryType::Exact => {}
            //     EntryType::UpperBound => {
            //         // if score >= beta {
            //         //     return score;
            //         // }
            //     }
            //     EntryType::LowerBound => {
            //         if alpha < score {
            //             alpha = score;
            //             if score >= beta {
            //                 return score;
            //             }
            //         }
            //     }
            // }
        }

        // Set the best score to the minimal value at first.
        // Worst case is that we lose next move.
        let mut best_score = eval::loss_score(ply as isize + 1);

        // Look at the child nodes:
        let moves = movegen::MoveGen::new(&self.position, best_move);
        for bmove in moves {
            match bmove {
                BitboardMove::SecondBest => {
                    self.position.second_best();
                    best_score = max(best_score, -self.negamax(depth, ply + 1, -beta, -alpha));
                    self.position.undo_second_best();
                }
                BitboardMove::StoneMove(smove) => {
                    // Enable for testing purposes.
                    // self.position
                    //     .try_make_move(bmove.to_player_move(&self.position))
                    //     .unwrap();
                    self.position.make_stone_move(smove);
                    best_score = max(best_score, -self.negamax(depth - 1, ply + 1, -beta, -alpha));
                    self.position.unmake_stone_move();
                }
            }
            if best_score > alpha {
                alpha = best_score;
                best_move = Some(bmove);
                if alpha >= beta {
                    break;
                }
            }
        }
        if let Some(best_move) = best_move {
            let entry_type = match eval::decode_eval(best_score) {
                eval::ExplainableEval::Undetermined(_) => EntryType::Undetermined,
                _ => match () {
                    _ if best_score <= initial_alpha => EntryType::UpperBound,
                    _ if best_score >= beta => EntryType::LowerBound,
                    _ => EntryType::Exact,
                },
            };
            self.t_table
                .store(&self.position, best_score, best_move, entry_type);
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

    fn initialize_for_search(&mut self) {
        self.nodes = 0;
    }

    pub fn search(&mut self, depth: usize) -> isize {
        self.initialize_for_search();
        let mut eval = 0;
        let start = time::Instant::now();
        for depth in 1..=depth {
            let new_eval = self.negamax(depth, 0, eval::LOSS, eval::WIN);
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

                // Skip for now, as there seems to be some bug.

                print!("pv");
                let mut pv_len = 0;
                while let Some(tt_entry) = self.t_table.get(&self.position) {
                    if pv_len > 40 {
                        // Prevent from being stuck in a loop.
                        break;
                    }
                    pv_len += 1;
                    print!(" {}", tt_entry.best_move_for_printing());
                    self.position.make_move(tt_entry.best_move(&self.position));
                }
                // Set position back to original state.
                for _ in 0..pv_len {
                    self.position.unmake_move();
                }
                println!();
            }
            match eval::decode_eval(eval) {
                eval::ExplainableEval::Win(_) | eval::ExplainableEval::Loss(_) => {
                    break;
                }
                _ => continue,
            }
        }
        eval
    }
}
