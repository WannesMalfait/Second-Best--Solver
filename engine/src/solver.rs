use crate::eval;
use crate::movegen;
use crate::position::GameStatus;
use crate::position::Position;
use crate::transposition_table::EntryType;
use crate::transposition_table::TranspositionTable;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time;

pub struct Solver {
    pub position: Position,
    nodes: usize,
    abort: Arc<AtomicBool>,
    /// If true, don't print anything to stdout.
    quiet: bool,
    ttable: TranspositionTable,
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            position: Position::default(),
            nodes: 0,
            abort: Arc::new(AtomicBool::new(false)),
            quiet: true,
            ttable: TranspositionTable::default(),
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
    fn negamax(&mut self, depth: usize, mut alpha: isize, beta: isize) -> isize {
        // Don't check this every node, but often enough.
        if self.nodes.is_multiple_of(1024) && self.abort_search() {
            // Have to stop the search now.
            return 0;
        }

        self.nodes += 1;
        match self.position.game_status() {
            GameStatus::WeLost => return eval::loss_score(self.position.ply() as isize),
            GameStatus::WeWon => return eval::win_score(self.position.ply() as isize),
            GameStatus::Draw => return 0,
            GameStatus::OnGoing => {}
        }
        if depth == 0 {
            // Return a static evaluation of the position.
            let eval = eval::static_eval(&self.position);
            return eval;
        }

        // Set the best score to the minimal value at first.
        // We already checked that we aren't lost now, so worst case we lose next ply.
        let mut best_score = eval::loss_score(self.position.ply() as isize + 1);
        if best_score >= beta {
            return best_score;
        }

        // Look up in the transposition table.
        let mut tt_move = None;
        if let Some(entry) = self.ttable.get(&self.position) {
            // If we are searching deeper then we can't trust the transposition table.
            if entry.depth() >= depth {
                let score = entry.score(self.position.ply() as isize);
                if score < eval::IS_WIN && score > eval::IS_LOSS {
                    // Don't look at mate evals for now.
                    // TODO: figure out what's wrong with mate evals.
                    match entry.entry_type() {
                        EntryType::Exact => return score,
                        EntryType::LowerBound => {
                            if score >= beta {
                                return score;
                            }
                        }
                        EntryType::UpperBound => {
                            if score <= alpha {
                                return score;
                            }
                        }
                    }
                }
            }
            // Still probably a good candidate to explore first.
            tt_move = Some(entry.best_move(&self.position))
        }

        // Look at the child nodes:
        let moves = movegen::MoveGen::new(&self.position, tt_move);
        let original_alpha = alpha;
        let mut best_move = None;
        for bmove in moves {
            if best_move.is_none() {
                best_move = Some(bmove);
            }
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
                best_score = eval;
                best_move = Some(bmove);
                if best_score > alpha {
                    alpha = best_score;
                    if alpha >= beta {
                        break;
                    }
                }
            }
        }
        // Store in Transposition Table
        self.ttable.store(
            &self.position,
            best_score,
            best_move.unwrap(),
            if best_score <= original_alpha {
                // There might be an even worse score, but we did a cut-off.
                EntryType::UpperBound
            } else if best_score >= beta {
                // There might be an event better score, but we did a cut-off.
                EntryType::LowerBound
            } else {
                // No cut-off.
                EntryType::Exact
            },
            depth,
            self.position.ply(),
        );

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
