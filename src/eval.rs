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
    todo!("Write evaluation function");
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
            side.other(),
            moves
        ),
        ExplainableEval::Undetermined(eval) => format!(
            "Result of the position is undetermined.\nBest score for ({}) is {} (Higher is better)",
            side, eval,
        ),
    }
}
