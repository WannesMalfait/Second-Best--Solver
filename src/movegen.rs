use crate::position;
use position::Bitboard;
use position::BitboardMove;
use position::Position;

pub struct MoveGen {
    /// Possible "to" spots.
    free_to_spots: Bitboard,
    /// Possible "from" spots. These are the spots on top of stacks we control.
    possible_from_spots: Bitboard,
    /// Potential move that we are not allowed to play anymore.
    banned_move: Option<Bitboard>,
    /// Are we in the second phase of the game?
    second_phase: bool,
    /// Current stack we are generating moves to.
    stack_i: usize,
    /// Current stage we are in to generate moves in the second phase.
    adjacent_stage: Adjacent,
    /// The pv-move from a previous iteration.
    pv_move: Option<BitboardMove>,
    played_pv: bool,
    /// Did we generate "Second Best!" already?
    did_second_best: bool,
}

/// Used to keep track of which moves we generated already.
#[derive(Debug)]
enum Adjacent {
    Left,
    Right,
    Opposite,
}

impl MoveGen {
    pub fn new(pos: &Position, pv_move: Option<BitboardMove>) -> Self {
        Self {
            free_to_spots: pos.free_spots(),
            possible_from_spots: pos.from_spots(true),
            banned_move: pos.banned_move(),
            second_phase: pos.is_second_phase(),
            stack_i: 0,
            adjacent_stage: Adjacent::Left,
            pv_move,
            played_pv: false,
            did_second_best: !pos.can_second_best(),
        }
    }
}

impl Iterator for MoveGen {
    type Item = BitboardMove;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.played_pv {
            self.played_pv = true;
            if let Some(pv_move) = self.pv_move {
                if pv_move == BitboardMove::SecondBest {
                    self.did_second_best = true;
                }
                return Some(pv_move);
            }
        }
        if !self.did_second_best {
            self.did_second_best = true;
            return Some(BitboardMove::SecondBest);
        }
        if self.stack_i == Position::NUM_STACKS {
            // All the stacks have been done.
            return None;
        }
        if !self.second_phase {
            while self.stack_i < Position::NUM_STACKS {
                let candidate = Position::column_mask(self.stack_i) & self.free_to_spots;
                self.stack_i += 1;
                if candidate != 0 && self.banned_move != Some(candidate) {
                    let candidate = Some(BitboardMove::StoneMove(candidate));
                    if candidate != self.pv_move {
                        return candidate;
                    }
                }
            }
        } else {
            while self.stack_i < Position::NUM_STACKS {
                let to = Position::column_mask(self.stack_i) & self.free_to_spots;
                if to == 0 {
                    // This stack was not free.
                    self.stack_i += 1;
                    continue;
                }
                let from = match self.adjacent_stage {
                    Adjacent::Left => {
                        self.adjacent_stage = Adjacent::Right;
                        Position::column_mask(self.stack_i + Position::LEFT)
                            & self.possible_from_spots
                    }
                    Adjacent::Right => {
                        self.adjacent_stage = Adjacent::Opposite;
                        Position::column_mask(self.stack_i + Position::RIGHT)
                            & self.possible_from_spots
                    }
                    Adjacent::Opposite => {
                        self.adjacent_stage = Adjacent::Left;
                        self.stack_i += 1;
                        Position::column_mask(self.stack_i - 1 + Position::OPPOSITE)
                            & self.possible_from_spots
                    }
                };
                if from != 0 {
                    let candidate = to | from;
                    if self.banned_move == Some(candidate) {
                        continue;
                    }
                    let candidate = Some(BitboardMove::StoneMove(candidate));
                    if candidate != self.pv_move {
                        return candidate;
                    }
                }
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
        pos.make_phase_one_move(0);
        pos.second_best();
        println!("{}", pos.can_second_best());
        let moves = MoveGen::new(&pos, None);
        assert_eq!(moves.count(), 7);
        let moves = MoveGen::new(&pos, None);
        for bmove in moves {
            let pmove = bmove.to_player_move(&pos);
            pos.try_make_move(pmove).unwrap();
            pos.unmake_stone_move();
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
            let pmove = smove.to_player_move(&pos);
            pos.try_make_move(pmove).unwrap();
            if !pos.has_alignment(false) {
                let moves = MoveGen::new(&pos, None);
                for smove in moves {
                    let pmove = smove.to_player_move(&pos);
                    pos.try_make_move(pmove).unwrap();
                    if !pos.has_alignment(false) {
                        let moves = MoveGen::new(&pos, None);
                        for smove in moves {
                            let pmove = smove.to_player_move(&pos);
                            pos.try_make_move(pmove).unwrap();
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
