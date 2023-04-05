use crate::position;
use crate::position::Move;
use position::Position;

pub struct MoveGen {
    /// Stacks with one of our stones on top.
    our_stacks: [bool; Position::NUM_STACKS as usize],
    /// Stacks that are already full.
    full_stacks: [bool; Position::NUM_STACKS as usize],
    /// Potential move that we are not allowed to play anymore.
    banned_move: Option<position::Move>,
    /// Are we in the second phase of the game?
    second_phase: bool,
    /// Current stack we are generating moves to.
    stack_i: u8,
    /// Current stage we are in to generate moves in the second phase.
    adjacent_stage: Adjacent,
    /// If the opponnent has an alignment.
    has_alignment: bool,
    /// The pv-move from a previous iteration.
    pv_move: Option<GenericMove>,
    played_pv: bool,
    /// Did we generate "Second Best!" already?
    did_second_best: bool,
}

/// Used to keep track of which moves we generated already.
enum Adjacent {
    Left,
    Right,
    Opposite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericMove {
    SecondBest,
    Move(Move),
}

impl MoveGen {
    pub fn new(pos: &Position, pv_move: Option<GenericMove>) -> Self {
        let mut our_stacks = [false; Position::NUM_STACKS as usize];
        let mut full_stacks = [false; Position::NUM_STACKS as usize];
        for (stack_i, stack) in pos.board().iter().enumerate() {
            let height = pos.stack_height(stack_i as u8);
            full_stacks[stack_i] = height == Position::STACK_HEIGHT;
            if height == 0 {
                continue;
            }
            our_stacks[stack_i] = stack[height as usize - 1] == Some(pos.current_player());
        }
        Self {
            our_stacks,
            full_stacks,
            banned_move: pos.banned_move(),
            second_phase: pos.is_second_phase(),
            stack_i: 0,
            adjacent_stage: Adjacent::Left,
            has_alignment: pos.player_has_alignment(pos.current_player().switch()),
            pv_move,
            played_pv: false,
            did_second_best: !pos.can_second_best(),
        }
    }
}

impl Iterator for MoveGen {
    type Item = GenericMove;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.played_pv && !self.has_alignment {
            self.played_pv = true;
            if let Some(pv_move) = self.pv_move {
                if pv_move == GenericMove::SecondBest {
                    self.did_second_best = true;
                }
                return Some(pv_move);
            }
        }
        if !self.did_second_best {
            self.did_second_best = true;
            return Some(GenericMove::SecondBest);
        }
        if self.has_alignment {
            // No legal moves except "Second Best!".
            return None;
        }
        if self.stack_i == Position::NUM_STACKS {
            // We have gone over all the possible locations already.
            return None;
        }
        if !self.second_phase {
            let banned_to = self.banned_move.map(|smove| smove.to);
            let pv_to = match self.pv_move {
                None => None,
                Some(GenericMove::SecondBest) => None,
                Some(GenericMove::Move(smove)) => Some(smove.to),
            };
            // In the first phase of the game, we just need to find a valid "to" stack.
            while self.stack_i < Position::NUM_STACKS {
                if !self.full_stacks[self.stack_i as usize]
                    && banned_to != Some(self.stack_i)
                    && pv_to != Some(self.stack_i)
                {
                    // Make sure that next time we start at the correct stack.
                    self.stack_i += 1;
                    return Some(GenericMove::Move(position::Move {
                        from: None,
                        to: self.stack_i - 1,
                    }));
                }
                self.stack_i += 1;
            }
            return None;
        }

        let pv = match self.pv_move {
            None => None,
            Some(GenericMove::SecondBest) => None,
            Some(GenericMove::Move(smove)) => Some(smove),
        };
        // We are in the second phase
        // stack_i is the index of the stack we are moving *to*.
        // each time we check if we can take from the left, right or opposite stack.
        while self.stack_i < Position::NUM_STACKS {
            // Can put something on this stack?
            if self.full_stacks[self.stack_i as usize] {
                self.stack_i += 1;
                continue;
            }
            // From where do we try to take?
            let from = match self.adjacent_stage {
                Adjacent::Left => (Position::LEFT + self.stack_i) % Position::NUM_STACKS,
                Adjacent::Right => (Position::RIGHT + self.stack_i) % Position::NUM_STACKS,
                Adjacent::Opposite => (Position::OPPOSITE + self.stack_i) % Position::NUM_STACKS,
            };
            let to = self.stack_i;
            self.adjacent_stage = match self.adjacent_stage {
                Adjacent::Left => Adjacent::Right,
                Adjacent::Right => Adjacent::Opposite,
                Adjacent::Opposite => {
                    // This was the last option for this stack. Move on to the next stack.
                    self.stack_i += 1;
                    Adjacent::Left
                }
            };
            let smove = position::Move {
                from: Some(from),
                to,
            };
            if self.our_stacks[from as usize]
                && Some(smove) != self.banned_move
                && Some(smove) != pv
            {
                // We can make this move.
                return Some(GenericMove::Move(position::Move {
                    from: Some(from),
                    to,
                }));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position;

    #[test]
    fn starting_pos() {
        let mut pos = position::Position::default();
        pos.make_move(position::Move { from: None, to: 0 });
        pos.second_best();
        println!("{}", pos.can_second_best());
        let moves = MoveGen::new(&pos, None);
        assert_eq!(moves.count(), 7);
        let moves = MoveGen::new(&pos, None);
        for smove in moves {
            let smove = match smove {
                GenericMove::SecondBest => continue,
                GenericMove::Move(smove) => smove,
            };
            println!("{smove}");
            pos.try_make_move(smove).unwrap();
            pos.unmake_move();
        }
    }

    #[test]
    fn second_phase() {
        let mut pos = position::Position::default();
        pos.parse_and_play_moves(
            "0 0 1 1 2 3 2 3 4 4 0 1 6 6 6 7"
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();
        assert!(pos.is_second_phase());
        let moves = MoveGen::new(&pos, None);
        for smove in moves {
            let smove = match smove {
                GenericMove::SecondBest => continue,
                GenericMove::Move(smove) => smove,
            };
            pos.try_make_move(smove).unwrap();
            if !pos.player_has_alignment(pos.current_player().switch()) {
                let moves = MoveGen::new(&pos, None);
                for smove in moves {
                    let smove = match smove {
                        GenericMove::SecondBest => continue,
                        GenericMove::Move(smove) => smove,
                    };
                    pos.try_make_move(smove).unwrap();
                    if !pos.player_has_alignment(pos.current_player().switch()) {
                        let moves = MoveGen::new(&pos, None);
                        for smove in moves {
                            let smove = match smove {
                                GenericMove::SecondBest => continue,
                                GenericMove::Move(smove) => smove,
                            };
                            pos.try_make_move(smove).unwrap();
                            pos.unmake_move();
                        }
                    }
                    pos.unmake_move();
                }
            }
            pos.unmake_move();
        }
    }
}
