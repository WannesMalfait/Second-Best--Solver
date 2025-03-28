use crate::position::{Color, Position};

pub const WIN: isize = 1000;
pub const IS_WIN: isize = WIN - 2 * Position::MAX_MOVES as isize;
pub const LOSS: isize = -WIN;
pub const IS_LOSS: isize = -IS_WIN;

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
    score += pos.controlled_stacks(true).count_ones() as isize;
    score -= pos.controlled_stacks(false).count_ones() as isize;
    // Since the bitboards store two copies of the board,
    // we need to divide by 2.
    score /= 2;
    if pos.has_alignment(Position::THEM) {
        // We don't check for us having an alignment, because that would already be a win.
        score -= 10;
    }
    score
}

/// The evaluation of a loss at the given ply.
#[inline]
pub fn loss_score(ply: isize) -> isize {
    LOSS + ply
}

/// The evaluation of a win at the given ply.
#[inline]
pub fn win_score(ply: isize) -> isize {
    WIN - ply
}

/// Turn the evaluation into a more digestible enum.
pub fn decode_eval(eval: isize, ply: isize) -> ExplainableEval {
    if eval < IS_LOSS {
        ExplainableEval::Loss(eval - LOSS - ply)
    } else if eval > IS_WIN {
        ExplainableEval::Win(WIN - eval - ply)
    } else {
        ExplainableEval::Undetermined(eval)
    }
}

/// Explain an evaluation in a human readable way.
pub fn explain_eval(side: Color, eval: isize, ply: isize) -> String {
    match decode_eval(eval, ply) {
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
