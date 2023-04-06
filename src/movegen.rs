use crate::position;
use position::Bitboard;
use position::BitboardMove;
use position::Position;

pub struct MoveGen {
    /// Spots which give us a vertical alignment.
    alignment_spots: Bitboard,
    /// Possible "to" spots not yet controlled by us.
    good_to_spots: Bitboard,
    /// Spots already controlled by us.
    bad_to_spots: Bitboard,
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
    /// If we can play "Second Best"
    can_second_best: bool,
    /// The stage of move generation we are in.
    stage: Stage,
}

#[derive(PartialEq, Eq)]
enum Stage {
    PvMove,
    SecondBest,
    VerticalAlignments,
    GoodToMoves,
    BadToMoves,
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
        let banned_move = pos.banned_move();
        let second_phase = pos.is_second_phase();
        let mut free_to_spots = pos.free_spots();
        let mut alignment_spots = pos.vertical_alignment_spots();
        if !second_phase {
            if let Some(banned_move) = banned_move {
                // Remove the banned move from the possible moves.
                free_to_spots &= !banned_move;
                alignment_spots &= !banned_move;
            }
        }
        let possible_from_spots = pos.from_spots(true);
        // Ensure we don't make the same moves twice.
        free_to_spots &= !alignment_spots;
        // A 1 on the top of every stack we control.
        let bad_spots = possible_from_spots << 1;
        let good_to_spots = free_to_spots & !bad_spots;
        let bad_to_spots = free_to_spots & bad_spots;
        Self {
            alignment_spots,
            good_to_spots,
            bad_to_spots,
            possible_from_spots,
            banned_move,
            second_phase,
            stack_i: 0,
            adjacent_stage: Adjacent::Left,
            can_second_best: pos.can_second_best(),
            pv_move,
            stage: Stage::PvMove,
        }
    }
}

impl Iterator for MoveGen {
    type Item = BitboardMove;
    fn next(&mut self) -> Option<Self::Item> {
        if self.stage == Stage::PvMove {
            self.stage = Stage::SecondBest;
            if let Some(pv_move) = self.pv_move {
                match pv_move {
                    BitboardMove::SecondBest => self.stage = Stage::GoodToMoves,
                    BitboardMove::StoneMove(smove) => {
                        if !self.second_phase {
                            // Remove the pv_move from the possible moves.
                            self.alignment_spots &= !smove;
                            self.good_to_spots &= !smove;
                            self.bad_to_spots &= !smove;
                        }
                        // If in the second phase, there could be multiple moves
                        // with the same "to" spot.
                    }
                }
                return Some(pv_move);
            }
        }
        if self.stage == Stage::SecondBest {
            self.stage = Stage::VerticalAlignments;
            if self.can_second_best {
                return Some(BitboardMove::SecondBest);
            }
        }
        if self.stage == Stage::VerticalAlignments {
            if self.alignment_spots != 0 {
                if let Some(bmove) = self.next_stone_move(self.alignment_spots) {
                    return Some(bmove);
                }
            }
            self.stack_i = 0;
            self.stage = Stage::GoodToMoves;
        }
        if self.stage == Stage::GoodToMoves {
            if self.good_to_spots != 0 {
                if let Some(bmove) = self.next_stone_move(self.good_to_spots) {
                    return Some(bmove);
                }
            }
            self.stack_i = 0;
            self.stage = Stage::BadToMoves;
        }
        if self.stage == Stage::BadToMoves {
            return self.next_stone_move(self.bad_to_spots);
        }
        None
    }
}

impl MoveGen {
    fn next_stone_move(&mut self, to_spots: Bitboard) -> Option<BitboardMove> {
        if !self.second_phase {
            while self.stack_i < Position::NUM_STACKS {
                let candidate = Position::column_mask(self.stack_i) & to_spots;
                self.stack_i += 1;
                if candidate != 0 {
                    // Checking for pv move and banned move is already handled.
                    return Some(BitboardMove::StoneMove(candidate));
                }
            }
        } else {
            while self.stack_i < Position::NUM_STACKS {
                let to = Position::column_mask(self.stack_i) & to_spots;
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
        let mut nodes = 0;
        let moves = MoveGen::new(&pos, None);
        for smove in moves {
            nodes += 1;
            let pmove = smove.to_player_move(&pos);
            pos.try_make_move(pmove).unwrap();
            if !pos.has_alignment(true) {
                let moves = MoveGen::new(&pos, None);
                for smove in moves {
                    nodes += 1;
                    let pmove = smove.to_player_move(&pos);
                    pos.try_make_move(pmove).unwrap();
                    if !pos.has_alignment(true) {
                        let moves = MoveGen::new(&pos, None);
                        for smove in moves {
                            nodes += 1;
                            let pmove = smove.to_player_move(&pos);
                            pos.try_make_move(pmove).unwrap();
                            if !pos.has_alignment(true) {
                                let moves = MoveGen::new(&pos, None);
                                for smove in moves {
                                    nodes += 1;
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
            pos.unmake_move();
        }
        assert_eq!(nodes, 2770);
    }
}
