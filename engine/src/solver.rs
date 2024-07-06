use crate::eval;
use crate::eval::explain_eval;
use crate::movegen;
use crate::position::GameStatus;
use crate::position::Position;
use crate::transposition_table::EntryType;
use crate::transposition_table::TranspositionTable;
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
    fn negamax(&mut self, depth: usize, mut alpha: isize, mut beta: isize) -> isize {
        // Don't check this every node, but often enough.
        if self.nodes % 1024 == 0 && self.abort_search() {
            // Have to stop the search now.
            return 0;
        }

        self.nodes += 1;
        match self.position.game_status() {
            GameStatus::WeLost => return eval::loss_score(self.position.ply() as isize),
            GameStatus::WeWon => return eval::win_score(self.position.ply() as isize),
            GameStatus::OnGoing => {}
        }
        if depth == 0 {
            // Return a static evaluation of the position.
            let eval = eval::static_eval(&self.position);
            return eval;
        }

        let initial_alpha = alpha;
        let initial_beta = beta;

        let mut best_move = None;
        if let Some(tt_entry) = self.t_table.get(&self.position) {
            best_move = Some(tt_entry.best_move(&self.position));
            // If we find the entry in a direct way, the score can be used.
            if tt_entry.ply() >= self.position.ply() {
                let score = tt_entry.score(self.position.ply() as isize);
                match tt_entry.entry_type() {
                    EntryType::Undetermined => (),
                    EntryType::Exact => {
                        return score;
                    }
                    EntryType::UpperBound => {
                        // if beta > score {
                        //     beta = score;
                        //     if score <= alpha {
                        //         return score;
                        //     }
                        // }
                    }
                    EntryType::LowerBound => {
                        // if alpha < score {
                        //     alpha = score;
                        //     if score >= beta {
                        //         return score;
                        //     }
                        // }
                    }
                }
            }
        }

        // Set the best score to the minimal value at first.
        // We already checked that we aren't lost now, so worst case we lose next ply.
        let mut best_score = eval::loss_score(self.position.ply() as isize + 1);
        if best_score >= beta {
            println!("beta cutoff {best_score} >= {beta} (initial = {initial_beta})");
            println!(
                "{}",
                explain_eval(
                    self.position.current_player(),
                    best_score,
                    self.position.ply() as isize,
                )
            );
            self.position.show();
            println!("ply {}", self.position.ply());
            return best_score;
        }

        // Look at the child nodes:
        let moves = movegen::MoveGen::new(&self.position, best_move);
        for bmove in moves {
            if cfg!(debug_assertions) {
                // Validate moves in debug builds.
                self.position
                    .try_make_move(bmove.to_player_move(&self.position))
                    .unwrap();
            } else {
                self.position.make_move(bmove);
            }
            let next_depth = if matches!(bmove, crate::position::BitboardMove::SecondBest) {
                //  Search lines where we "Second Best!" a little longer.
                depth
            } else {
                depth - 1
            };
            let eval = -self.negamax(next_depth, -beta, -alpha);

            self.position.unmake_move();
            if eval > best_score {
                best_move = Some(bmove);
                best_score = eval;
                if best_score > alpha {
                    alpha = best_score;
                    if alpha >= beta {
                        break;
                    }
                }
            }
        }
        if let Some(best_move) = best_move {
            let entry_type = match eval::decode_eval(best_score, self.position.ply() as isize) {
                eval::ExplainableEval::Undetermined(_) => EntryType::Undetermined,
                eval::ExplainableEval::Win(_) | eval::ExplainableEval::Loss(_) => match () {
                    _ if best_score >= initial_beta => EntryType::LowerBound,
                    _ if best_score <= initial_alpha => EntryType::UpperBound,
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
            let new_eval = self.negamax(depth, eval::LOSS, eval::WIN);
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
                print!("pv");
                let mut keys = vec![];
                while let Some(tt_entry) = self.t_table.get(&self.position) {
                    let key = TranspositionTable::key(&self.position);
                    if keys.contains(&key) {
                        // Prevent from being stuck in a loop.
                        break;
                    }
                    keys.push(key);
                    print!(" {}", tt_entry.best_move_for_printing());
                    self.position.make_move(tt_entry.best_move(&self.position));
                }
                // Set position back to original state.
                for _ in 0..keys.len() {
                    self.position.unmake_move();
                }
                println!();
            }
            match eval::decode_eval(eval, self.position.ply() as isize) {
                eval::ExplainableEval::Win(_) | eval::ExplainableEval::Loss(_) => {
                    break;
                }
                _ => continue,
            }
        }
        eval
    }
}
