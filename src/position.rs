use std::fmt::Display;

pub struct Position {
    /// State of the board, with `NUM_STACKS` stacks of stones of height `STACK_HEIGHT`.
    board: [[Stone; Self::STACK_HEIGHT as usize]; Self::NUM_STACKS as usize],
    /// Number of stone moves played since the beginning of the game.
    moves: u8,
    /// The player whose turn it is.
    current_player: Color,
    /// Moves played throughout the game.
    move_history: [Option<Move>; Self::MAX_MOVES as usize],
    /// Moves which were prohibited, since "Second Best!" was called on it.
    banned_moves: [Option<Move>; Self::MAX_MOVES as usize],
}

impl Position {
    pub const NUM_STACKS: u8 = 8;
    pub const STACK_HEIGHT: u8 = 3;
    pub const STONES_PER_PLAYER: u8 = 8;
    const MAX_MOVES: u8 = 255;
}

impl Default for Position {
    fn default() -> Self {
        Position {
            board: [[None; Self::STACK_HEIGHT as usize]; Self::NUM_STACKS as usize],
            moves: 0,
            current_player: Color::Black,
            move_history: [None; Self::MAX_MOVES as usize],
            banned_moves: [None; Self::MAX_MOVES as usize],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Color {
    Black,
    White,
}

impl Color {
    fn switch(&self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Color::Black => "X",
                Color::White => "O",
            }
        )
    }
}
type Stone = Option<Color>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move {
    from: Option<u8>,
    to: u8,
}

#[derive(PartialEq, Eq, Debug)]
pub enum MoveFailed {
    /// We are in the second phase of the game, but no from spot was given.
    MissingFromSpot,
    /// The starting spot is not one of the current players stones, or out of bounds.
    InvalidFromSpot,
    /// The resulting spot is not one of the valid spot to move to, or is out of bounds.
    InvalidToSpot,
    /// The move to be played was banned by a "Second Best!" call.
    MoveBanned,
    /// Second Best! was called when it is not allowed.
    InvalidSecondBest,
    /// The "from" and "to" spot are the same.
    SameFromAndTo,
    /// An error while parsing the move from a string.
    ParseError,
    /// The position is winning for the other player, so no move except calling "Second Best!" is possible.
    PositionWinning,
}

impl Position {
    /// Are we in the second phase of the game, where stones are no longer placed, but moved.
    #[inline]
    fn is_second_phase(&self) -> bool {
        self.moves >= 2 * Self::STONES_PER_PLAYER
    }

    fn stack_height(&self, stack: u8) -> u8 {
        let mut num_stones = 0;
        for stone in self.board[stack as usize] {
            if stone.is_none() {
                break;
            }
            num_stones += 1;
        }
        num_stones
    }

    /// Check if the "to" spot is adjacent or opposite to the "from" spot.
    fn valid_adjacent(from: u8, to: u8) -> bool {
        // We add Self::NUM_STACKS before subtracting 1
        // to prevent an underflow.
        (from + 1) % Self::NUM_STACKS == to
            || (from + Self::NUM_STACKS / 2) % Self::NUM_STACKS == to
            || (from + Self::NUM_STACKS - 1) % Self::NUM_STACKS == to
    }

    /// Is the given move banned due to a "Second Best!" call?
    #[inline]
    fn is_move_banned(&self, smove: Move) -> bool {
        self.banned_moves[self.moves as usize + 1] == Some(smove)
    }

    /// Make the move if it is valid, otherwise return why it wasn't valid.
    pub fn try_make_move(&mut self, smove: Move) -> Result<(), MoveFailed> {
        if self.is_move_banned(smove) {
            return Err(MoveFailed::MoveBanned);
        }
        if self.player_has_alignment(self.current_player.switch()) {
            return Err(MoveFailed::PositionWinning);
        }
        if self.is_second_phase() {
            let Some(from) = smove.from else {
                return Err(MoveFailed::MissingFromSpot);
            };
            // Check for out of bounds indices.
            if from >= Self::NUM_STACKS {
                return Err(MoveFailed::InvalidFromSpot);
            }
            if smove.to >= Self::NUM_STACKS {
                return Err(MoveFailed::InvalidToSpot);
            }

            if smove.to == from {
                return Err(MoveFailed::SameFromAndTo);
            }

            // We are moving a stone from the "from" spot so:
            // 1. There should be a stone at that spot
            // 2. The stone at that spot should be of the current
            //    player's color.
            let from_height = self.stack_height(from);
            if from_height == 0
                || self.board[from as usize][from_height as usize - 1] != Some(self.current_player)
            {
                return Err(MoveFailed::InvalidFromSpot);
            }

            // We are moving a stone to the "to" spot, so:
            // 1. The "to" spot should be "adjacent" to the "from" spot
            // 2. The stack at the "to" spot should not be full.
            let to_height = self.stack_height(smove.to);
            if !Self::valid_adjacent(from, smove.to) || to_height >= Self::STACK_HEIGHT {
                return Err(MoveFailed::InvalidToSpot);
            }
            self.make_phase_two_move(from, from_height, smove.to, to_height);
            return Ok(());
        }

        // There should be no "from" spot.
        if smove.from.is_some() {
            return Err(MoveFailed::InvalidFromSpot);
        }

        // Check for out of bounds indices.
        if smove.to >= Self::NUM_STACKS {
            return Err(MoveFailed::InvalidToSpot);
        }

        // We are moving a stone to the "to" spot, so:
        // the stack at the "to" spot should not be full.
        let to_height = self.stack_height(smove.to);
        if to_height >= Self::STACK_HEIGHT {
            return Err(MoveFailed::InvalidToSpot);
        }
        self.make_phase_one_move(smove.to, to_height);
        Ok(())
    }

    /// Make a move in the second phase of the game (moving a stone to an adjacent stack).
    /// The move should be valid.
    fn make_phase_two_move(&mut self, from: u8, from_height: u8, to: u8, to_height: u8) {
        // Remove stone from the "from" stack.
        self.board[from as usize][from_height as usize - 1] = None;
        // Place stone on top of the "to" stack.
        self.board[to as usize][to_height as usize] = Some(self.current_player);
        // Update board information.
        self.moves += 1;
        self.current_player = self.current_player.switch();
        self.move_history[self.moves as usize] = Some(Move {
            from: Some(from),
            to,
        });
    }

    /// Make a move in the first phase of the game (placing a stone on one of the stacks).
    /// The move should be valid.
    fn make_phase_one_move(&mut self, to: u8, to_height: u8) {
        // Place stone on top of the "to" stack.
        self.board[to as usize][to_height as usize] = Some(self.current_player);
        // Update board information.
        self.moves += 1;
        self.current_player = self.current_player.switch();
        self.move_history[self.moves as usize] = Some(Move { from: None, to });
    }

    /// Unmake the last move played.
    /// Returns the move that was undone.
    pub fn unmake_move(&mut self) -> Move {
        if self.moves == 0 {
            unreachable!("Logic error in the program");
        }
        let Some(last_move) = self.move_history[self.moves as usize] else {
            unreachable!("There should be a move, because we have played that many moves.")
        };
        self.move_history[self.moves as usize] = None;
        self.banned_moves[self.moves as usize] = None;
        self.moves -= 1;
        self.current_player = self.current_player.switch();
        let to = last_move.to;
        let to_height = self.stack_height(to);
        // Remove the stone placed on this stack.
        self.board[to as usize][to_height as usize - 1] = None;
        if let Some(from) = last_move.from {
            let from_height = self.stack_height(from);
            // Place back the stone on the stack it came from.
            self.board[from as usize][from_height as usize] = Some(self.current_player);
        }
        last_move
    }

    /// Check if "Second Best!" can be called this move.
    /// 1. There should be at least one move played.
    /// 2. "Second Best!" should not have been called yet.
    #[inline]
    pub fn can_second_best(&self) -> bool {
        self.moves > 0 && self.banned_moves[self.moves as usize].is_none()
    }

    /// Opponent called "Second Best!"
    /// This should only be called if `can_second_best()` is true.
    pub fn second_best(&mut self) {
        let last_move = self.unmake_move();
        self.banned_moves[self.moves as usize + 1] = Some(last_move);
    }

    /// Parses the moves and plays them one by one.
    /// Returns on the first error.
    ///
    /// Each element of `moves` should be a single move:
    ///    
    /// A move can be either:
    /// 1. A single number representing a stack index, e.g. 1
    /// 2. A "!" representing a second best call
    /// 3. Two numbers representing a from and to spot for a move in the second part of the game,
    ///    separated by a "-", e.g. 0-2
    pub fn parse_and_play_moves(&mut self, moves: Vec<String>) -> Result<(), MoveFailed> {
        for smove in moves {
            if smove == "!" {
                if !self.can_second_best() {
                    return Err(MoveFailed::InvalidSecondBest);
                }
                self.second_best();
                continue;
            }
            let numbers: Vec<&str> = smove.split('-').collect();
            match numbers.len() {
                1 => {
                    let to = match numbers[0].parse() {
                        Ok(n) => n,
                        Err(_) => return Err(MoveFailed::ParseError),
                    };
                    self.try_make_move(Move { from: None, to })?;
                }
                2 => {
                    let from = match numbers[0].parse() {
                        Ok(n) => n,
                        Err(_) => return Err(MoveFailed::ParseError),
                    };
                    let to = match numbers[1].parse() {
                        Ok(n) => n,
                        Err(_) => return Err(MoveFailed::ParseError),
                    };
                    self.try_make_move(Move {
                        from: Some(from),
                        to,
                    })?;
                }
                _ => return Err(MoveFailed::ParseError),
            }
        }
        Ok(())
    }

    /// Check if the given player has an alignment on the board:
    /// Either:
    /// 1. There is a stack with three stones of the player's color.
    /// 2. There are 4 stones of the players colors next to each other
    ///    on the top of the stacks.
    fn player_has_alignment(&self, player: Color) -> bool {
        let mut top_of_stacks = [None; Self::NUM_STACKS as usize];
        for (stack_i, stack) in self.board.iter().enumerate() {
            let mut all_ours = true;
            for &stone in stack {
                if stone.is_none() {
                    all_ours = false;
                    break;
                }
                top_of_stacks[stack_i] = stone;
                if stone != Some(player) {
                    all_ours = false;
                }
            }
            if all_ours {
                return true;
            }
        }
        // Check for a sequence of 4 in the stack tops.
        let mut consecutive = 0;
        for i in 0..(Self::NUM_STACKS + 3) {
            if top_of_stacks[(i % Self::NUM_STACKS) as usize] == Some(player) {
                consecutive += 1;
                if consecutive == 4 {
                    return true;
                }
            } else {
                consecutive = 0;
            }
        }
        false
    }

    /// Display the current state of the board.
    pub fn show(&self) {
        //     .
        // .   .   .
        //  .  .  .
        //   .   .
        //...     ...
        //   .   .
        //  .  .  .
        // .   .   .
        //     .
        let indices = [
            [
                (8, 8),
                (8, 8),
                (8, 8),
                (8, 8),
                (4, 2),
                (8, 8),
                (8, 8),
                (8, 8),
                (8, 8),
            ],
            [
                (8, 8),
                (5, 2),
                (8, 8),
                (8, 8),
                (4, 1),
                (8, 8),
                (8, 8),
                (3, 2),
                (8, 8),
            ],
            [
                (8, 8),
                (8, 8),
                (5, 1),
                (8, 8),
                (4, 0),
                (8, 8),
                (3, 1),
                (8, 8),
                (8, 8),
            ],
            [
                (8, 8),
                (8, 8),
                (8, 8),
                (5, 0),
                (8, 8),
                (3, 0),
                (8, 8),
                (8, 8),
                (8, 8),
            ],
            [
                (6, 2),
                (6, 1),
                (6, 0),
                (8, 8),
                (8, 8),
                (8, 8),
                (2, 0),
                (2, 1),
                (2, 2),
            ],
            [
                (8, 8),
                (8, 8),
                (8, 8),
                (7, 0),
                (8, 8),
                (1, 0),
                (8, 8),
                (8, 8),
                (8, 8),
            ],
            [
                (8, 8),
                (8, 8),
                (7, 1),
                (8, 8),
                (0, 0),
                (8, 8),
                (1, 1),
                (8, 8),
                (8, 8),
            ],
            [
                (8, 8),
                (7, 2),
                (8, 8),
                (8, 8),
                (0, 1),
                (8, 8),
                (8, 8),
                (1, 2),
                (8, 8),
            ],
            [
                (8, 8),
                (8, 8),
                (8, 8),
                (8, 8),
                (0, 2),
                (8, 8),
                (8, 8),
                (8, 8),
                (8, 8),
            ],
        ];
        let mut s = "".to_string();
        for row in indices {
            for coordinate in row {
                if coordinate == (8, 8) {
                    s += " ";
                } else {
                    let c = match self.board[coordinate.0][coordinate.1] {
                        None => ".".to_string(),
                        Some(c) => c.to_string(),
                    };
                    s += &c;
                }
                s += " ";
            }
            s += "\n";
        }
        println!("{s}");
        println!(
            "It is {} turn to move",
            match self.current_player {
                Color::Black => "black's (X)",
                Color::White => "white's (O)",
            }
        );
        if let Some(banned_move) = self.banned_moves[self.moves as usize + 1] {
            println!(
                "Banned move: {}",
                match banned_move.from {
                    None => format!("{}", banned_move.to),
                    Some(from) => format!("{}-{}", from, banned_move.to),
                }
            )
        }
        if self.player_has_alignment(self.current_player.switch()) {
            println!("{} has an alignment", self.current_player.switch());
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn invalid_moves() {
        let mut pos = Position::default();
        assert_eq!(
            pos.try_make_move(Move {
                from: None,
                to: Position::NUM_STACKS
            }),
            Err(MoveFailed::InvalidToSpot)
        );
        for _ in 0..Position::STACK_HEIGHT {
            assert_eq!(pos.try_make_move(Move { from: None, to: 0 }), Ok(()));
        }
        assert_eq!(
            pos.try_make_move(Move { from: None, to: 0 }),
            Err(MoveFailed::InvalidToSpot)
        );
        assert_eq!(pos.moves, Position::STACK_HEIGHT);
        // Use up all the stones.
        'outer: for stack in 1..Position::NUM_STACKS {
            for _ in 0..Position::STACK_HEIGHT {
                if pos.is_second_phase() {
                    break 'outer;
                }
                assert_eq!(
                    pos.try_make_move(Move {
                        from: None,
                        to: stack
                    }),
                    Ok(())
                );
            }
        }
        // Now in second phase.
        assert_eq!(
            pos.try_make_move(Move {
                from: None,
                to: Position::NUM_STACKS - 1
            }),
            Err(MoveFailed::MissingFromSpot)
        );
        assert_eq!(
            pos.try_make_move(Move {
                from: Some(Position::NUM_STACKS),
                to: Position::NUM_STACKS - 1
            }),
            Err(MoveFailed::InvalidFromSpot)
        );
        assert_eq!(
            pos.try_make_move(Move {
                from: Some(0),
                to: Position::NUM_STACKS - 2
            }),
            Err(MoveFailed::InvalidToSpot)
        );
        assert_eq!(
            pos.try_make_move(Move {
                from: Some(0),
                to: Position::NUM_STACKS - 1
            }),
            Ok(()),
        );
        pos.second_best();
        assert_eq!(
            pos.try_make_move(Move {
                from: Some(0),
                to: Position::NUM_STACKS - 1
            }),
            Err(MoveFailed::MoveBanned),
        );
    }

    #[test]
    fn second_best() {
        let mut pos = Position::default();
        assert!(!pos.can_second_best());
        pos.make_phase_one_move(0, 0);
        assert!(pos.can_second_best());
        pos.second_best();
        assert_eq!(pos.moves, 0);
        assert_eq!(
            pos.try_make_move(Move { from: None, to: 0 }),
            Err(MoveFailed::MoveBanned)
        );
        pos.make_phase_one_move(1, 0);
        assert!(!pos.can_second_best());
    }

    #[test]
    fn parsing_moves() {
        let mut pos = Position::default();
        assert_eq!(
            pos.parse_and_play_moves(vec!["".to_string()]),
            Err(MoveFailed::ParseError)
        );

        assert_eq!(pos.parse_and_play_moves(vec!["0".to_string()]), Ok(()));
        assert_eq!(
            pos.parse_and_play_moves(vec!["-0".to_string()]),
            Err(MoveFailed::ParseError)
        );
        assert_eq!(
            pos.parse_and_play_moves(vec!["1-0".to_string()]),
            Err(MoveFailed::InvalidFromSpot)
        );
        assert_eq!(
            pos.parse_and_play_moves(vec!["21".to_string()]),
            Err(MoveFailed::InvalidToSpot)
        );

        assert_eq!(pos.parse_and_play_moves(vec!["!".to_string()]), Ok(()));
    }

    #[test]
    fn alignments() {
        let mut pos = Position::default();
        assert!(!pos.player_has_alignment(pos.current_player));
        pos.parse_and_play_moves("0 0 0".split_whitespace().map(|s| s.to_string()).collect())
            .unwrap();
        assert!(!pos.player_has_alignment(pos.current_player));
        pos.parse_and_play_moves(
            "1 2 1 2 1"
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();
        assert!(pos.player_has_alignment(pos.current_player.switch()));
        pos.second_best();
        pos.parse_and_play_moves(
            "2 1 3 7 4 6"
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();
        pos.show();
        assert!(pos.player_has_alignment(pos.current_player.switch()));
    }
}
