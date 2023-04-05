use crate::position;
use position::Bitboard;
use position::BitboardMove;
use position::Position;

pub struct MoveGen {
    /// Potential move that we are not allowed to play anymore.
    banned_move: Option<Bitboard>,
    /// Are we in the second phase of the game?
    second_phase: bool,
    /// Current stack we are generating moves to.
    stack_i: u8,
    /// Current stage we are in to generate moves in the second phase.
    adjacent_stage: Adjacent,
    /// If the opponnent has an alignment.
    has_alignment: bool,
    /// The pv-move from a previous iteration.
    pv_move: Option<BitboardMove>,
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

impl MoveGen {
    pub fn new(pos: &Position, pv_move: Option<BitboardMove>) -> Self {
        Self {
            banned_move: pos.banned_move(),
            second_phase: pos.is_second_phase(),
            stack_i: 0,
            adjacent_stage: Adjacent::Left,
            has_alignment: pos.has_alignment(false),
            pv_move,
            played_pv: false,
            did_second_best: !pos.can_second_best(),
        }
    }
}

impl Iterator for MoveGen {
    type Item = BitboardMove;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
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
