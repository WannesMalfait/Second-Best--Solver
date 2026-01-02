use std::{num::ParseIntError, ops::Neg, str::FromStr};

use crate::position::{Color, Position};

type ScoreType = i16;
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score(ScoreType);

impl std::fmt::Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Score {
    pub const WIN: Score = Score(1000);
    pub const LOSS: Score = Score(-Self::WIN.0);
    const IS_WIN: Score = Score(Self::WIN.0 - 2 * Position::MAX_MOVES as ScoreType);
    const IS_LOSS: Score = Score(-Self::IS_WIN.0);

    pub fn is_win(self) -> bool {
        self >= Self::IS_WIN
    }
    pub fn is_loss(self) -> bool {
        self <= Self::IS_LOSS
    }
    /// Either a win or a loss
    pub fn is_mate(self) -> bool {
        self.is_win() || self.is_loss()
    }

    /// Score for a loss in `ply` moves
    pub fn loss_in(ply: usize) -> Self {
        Score(Self::LOSS.0 + ply as ScoreType)
    }

    /// Score for a win in `ply` moves
    pub fn win_in(ply: usize) -> Self {
        Score(Self::WIN.0 - ply as ScoreType)
    }

    pub fn draw() -> Score {
        Score(0)
    }

    pub fn middle(a: Score, b: Score) -> Self {
        Score(a.0 + (b.0 - a.0) / 2)
    }

    /// Increase relative ply of mates by 1.
    pub fn increase_ply(self) -> Self {
        if self.is_loss() {
            Score(self.0 + 1)
        } else if self.is_win() {
            Score(self.0 - 1)
        } else {
            self
        }
    }

    /// Decrease relative ply of mates by 1.
    pub fn decrease_ply(self) -> Self {
        if self.is_loss() && self != Self::LOSS {
            Score(self.0 - 1)
        } else if self.is_win() && self != Self::WIN {
            Score(self.0 + 1)
        } else {
            self
        }
    }

    /// Turn the evaluation into a more digestible enum.
    pub fn decode_eval(self) -> ExplainableEval {
        if self.is_loss() {
            ExplainableEval::Loss((self.0 - Self::LOSS.0) as usize)
        } else if self.is_win() {
            ExplainableEval::Win((Self::WIN.0 - self.0) as usize)
        } else {
            ExplainableEval::Undetermined(self.0)
        }
    }

    pub fn half(self) -> Score {
        Score(self.0 / 2)
    }
}

impl Neg for Score {
    type Output = Score;

    fn neg(self) -> Self::Output {
        Score(-self.0)
    }
}

impl FromStr for Score {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Score(s.parse()?))
    }
}

pub enum ExplainableEval {
    /// A win, with how many moves needed to get there.
    Win(usize),
    /// A loss, with how many moves needed to get there.
    Loss(usize),
    /// Position is not yet solved, best score at the searched depth.
    Undetermined(ScoreType),
}

/// Return a static evaluation of the position.
pub fn static_eval(pos: &Position) -> Score {
    // For now just count how many stacks are controlled by each player.
    let mut score = 0;
    score += pos.controlled_stacks(Position::US).count_ones() as ScoreType;
    score -= pos.controlled_stacks(Position::THEM).count_ones() as ScoreType;
    // Since the bitboards store two copies of the board,
    // we need to divide by 2.
    score /= 2;
    if pos.has_alignment(Position::THEM) {
        // We don't check for us having an alignment, because that would already be a win.
        score -= 10;
    }
    Score(score)
}

/// Explain an evaluation in a human readable way.
pub fn explain_eval(side: Color, eval: Score) -> String {
    match eval.decode_eval() {
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
