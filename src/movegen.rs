use crate::position;
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
}

/// Used to keep track of which moves we generated already.
enum Adjacent {
    Left,
    Right,
    Opposite,
}

impl MoveGen {
    fn new(pos: Position) -> Self {
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
        }
    }
}

impl Iterator for MoveGen {
    type Item = position::Move;
    fn next(&mut self) -> Option<Self::Item> {
        if self.stack_i == Position::NUM_STACKS {
            // We have gone over all the possible locations already.
            return None;
        }
        if !self.second_phase {
            let banned_to = match self.banned_move {
                Some(smove) => Some(smove.to),
                None => None,
            };
            // In the first phase of the game, we just need to find a valid "to" stack.
            while self.stack_i < Position::NUM_STACKS {
                if !self.full_stacks[self.stack_i as usize] && banned_to != Some(self.stack_i) {
                    // Make sure that next time we start at the correct stack.
                    self.stack_i += 1;
                    return Some(position::Move {
                        from: None,
                        to: self.stack_i - 1,
                    });
                }
                self.stack_i += 1;
            }
            return None;
        }

        let banned_to = match self.banned_move {
            Some(smove) => Some(smove.to),
            None => None,
        };
        let banned_from = match self.banned_move {
            Some(smove) => smove.from,
            None => None,
        };
        // We are in the second phase
        // stack_i is the index of the stack we are moving *to*.
        // each time we check if we can take from the left, right or opposite stack.
        while self.stack_i < Position::NUM_STACKS {
            // Can put something on this stack?
            if self.full_stacks[self.stack_i as usize] || banned_to == Some(self.stack_i) {
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
            if self.our_stacks[from as usize] && banned_from != Some(from) {
                // We can make this move.
                return Some(position::Move {
                    from: Some(from),
                    to,
                });
            }
        }
        None
    }
}