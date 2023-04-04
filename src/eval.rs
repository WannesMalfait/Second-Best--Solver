use crate::position::{Color, Position};

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
    if pos.player_has_alignment(pos.current_player().switch()) {
        score -= 10;
    } else if pos.player_has_alignment(pos.current_player()) {
        // Only get a bonus if our opponent isn't winning already.
        score += 10;
    }
    score
}

/// The evaluation of a loss in a position with `num_moves` moves.
#[inline]
pub fn loss_score(num_moves: isize) -> isize {
    LOSS + num_moves
}

/// Turn the evaluation into a more digestible enum.
pub fn decode_eval(num_moves: isize, eval: isize) -> ExplainableEval {
    if eval < LOSS + Position::MAX_MOVES as isize {
        ExplainableEval::Loss(eval - LOSS - num_moves)
    } else if eval > WIN - Position::MAX_MOVES as isize {
        ExplainableEval::Win(WIN - eval - num_moves)
    } else {
        ExplainableEval::Undetermined(eval)
    }
}

/// Explain an evaluation in a human readable way.
pub fn explain_eval(num_moves: isize, side: Color, eval: isize) -> String {
    match decode_eval(num_moves, eval) {
        ExplainableEval::Win(moves) => format!(
            "Position is winning:\n{} can win in {} move(s)",
            side, moves
        ),
        ExplainableEval::Loss(moves) => format!(
            "Position is lost:\n{} can win in {} move(s)",
            side.switch(),
            moves
        ),
        ExplainableEval::Undetermined(eval) => format!(
            "Result of the position is undetermined.\nBest score for ({}) is {} (Higher is better)",
            side, eval,
        ),
    }
}
