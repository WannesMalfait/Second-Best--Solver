use crate::eval;
use crate::eval::Score;
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
    fn negamax(&mut self, depth: usize, mut alpha: Score, beta: Score) -> Score {
        // Don't check this every node, but often enough.
        if self.nodes.is_multiple_of(1024) && self.abort_search() {
            // Have to stop the search now.
            return Score::draw();
        }

        self.nodes += 1;
        match self.position.game_status() {
            GameStatus::WeLost => return Score::loss_in(0),
            GameStatus::WeWon => return Score::win_in(0),
            GameStatus::Draw => return Score::draw(),
            GameStatus::OnGoing => {}
        }
        if depth == 0 {
            // Return a static evaluation of the position.
            let eval = eval::static_eval(&self.position);
            return eval;
        }

        // Set the best score to the minimal value at first.
        // We already checked that we aren't lost now, so worst case we lose next ply.
        let mut best_score = Score::loss_in(1);
        if best_score >= beta {
            return best_score;
        }

        // Look up in the transposition table.
        let mut tt_move = None;
        if let Some(entry) = self.ttable.get(&self.position) {
            // If we are searching deeper then we can't trust the transposition table.
            if entry.depth() >= depth {
                let score = entry.score();
                if !score.is_mate() {
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
            // TODO: experiment with selectively increasing search.
            // Doing so can cause the solver to find slower mates, so it should be carefully implemented.
            let next_depth = depth - 1;

            // Ensure that the ply is kept track of correctly for mate evals.
            let eval = -self.negamax(next_depth, -beta, -alpha).increase_ply();

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

    pub fn search(&mut self, depth: usize) -> Score {
        self.initialize_for_search();
        let mut eval = Score::default();
        let start = time::Instant::now();
        for depth in 1..=depth {
            let new_eval = self.negamax(depth, Score::LOSS, Score::WIN);
            if self.abort_search() {
                return eval;
            }
            eval = new_eval;
            if !self.quiet {
                let elapsed = start.elapsed();
                let nodes = self.nodes;
                let knps = self.nodes as u128 / (1 + elapsed.as_millis());
                println!(
                    "info depth {depth} score {eval:?} nodes {nodes} knps {knps} ({:?} total time)",
                    elapsed
                );
                print!("pv");
                let mut pv_keys = vec![TranspositionTable::key(&self.position)];
                let mut pv_pos = self.position.clone();
                while let Some(entry) = self.ttable.get(&pv_pos) {
                    let best = entry.best_move_for_printing();
                    print!(" {best}");
                    pv_pos.try_make_move(best).unwrap();
                    if pv_keys.contains(&TranspositionTable::key(&pv_pos)) {
                        break;
                    };
                    pv_keys.push(TranspositionTable::key(&pv_pos));
                }
                println!();
            }
            if eval.is_mate() {
                break;
            }
        }
        eval
    }
}
