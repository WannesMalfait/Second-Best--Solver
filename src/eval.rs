use crate::position::Position;

pub const WIN: isize = 1000;
pub const LOSS: isize = -WIN;

pub enum ExplainableEval {
    /// A win, with how many moves needed to get there.
    Win(isize),
    /// A loss, with how many moves needed to get there.
    Loss(isize),
    /// Position is not yet solved, best score at the searched depth.
    Undetermined(isize),
}

/// Return a static evaluation of the position.
pub fn static_eval(pos: &Position) -> isize {
    // For now just count how many stacks are controlled by each player.
    let mut score = 0;
    for (stack_i, stack) in pos.board().iter().enumerate() {
        let height = pos.stack_height(stack_i as u8);
        if height == 0 {
            continue;
        }
        let top_of_stack = stack[height as usize - 1];
        match top_of_stack {
            None => continue,
            Some(color) => {
                if color == pos.current_player() {
                    score += height as isize;
                } else {
                    score -= height as isize;
                }
            }
        }
    }
    if pos.player_has_alignment(pos.current_player()) {
        score += 10;
    }
    if pos.player_has_alignment(pos.current_player().switch()) {
        score -= 10;
    }
    score
}

/// The evaluation of a loss in the given position.
#[inline]
pub fn loss_score(pos: &Position) -> isize {
    LOSS + pos.num_moves() as isize
}

/// Turn the evaluation into a more digestible enum.
pub fn decode_eval(pos: &Position, eval: isize) -> ExplainableEval {
    if eval < LOSS + Position::MAX_MOVES as isize {
        ExplainableEval::Loss(eval - LOSS - pos.num_moves() as isize)
    } else if eval > WIN - Position::MAX_MOVES as isize {
        ExplainableEval::Win(WIN - eval - pos.num_moves() as isize)
    } else {
        ExplainableEval::Undetermined(eval)
    }
}

/// Explain an evaluation in a human readable way.
pub fn explain_eval(pos: &Position, eval: isize) -> String {
    match decode_eval(pos, eval) {
        ExplainableEval::Win(moves) => format!(
            "Position is winning:\n{} can win in {} move(s)",
            pos.current_player(),
            moves
        ),
        ExplainableEval::Loss(moves) => format!(
            "Position is lost:\n{} can win in {} move(s)",
            pos.current_player().switch(),
            moves
        ),
        ExplainableEval::Undetermined(eval) => format!(
            "Result of the position is undetermined.\nBest score for ({}) is {} (Higher is better)",
            pos.current_player(),
            eval,
        ),
    }
}
