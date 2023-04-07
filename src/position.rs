use std::fmt::Display;
/// A bitboard is a way to efficiently store board state.
/// The board has 8 stacks with a maximal height of 3.
/// In total there are hence 8 * 3 = 24 spots.
/// So, technically we just need a u32. However, since the
/// board is in a circle, we will place two copies of the
/// board next to each other, to be able to check alignments
/// much more easily.
///
/// We store the state of the board in two different Bitboards:
/// - played_spots: with a bit set for every spot occupied by
///   any player
/// - our_spots: with a bit set for every spot occupied by the
///   current player to move.
///
/// Example:
/// play 1 0 4 7 2 4 2 3:
///         .
///   .     O     .
///     .   X   .
///       O   X
/// . . .       X . .
///       O   X
///     .   O   .
///   .     .     .
///         .
/// It is black's (X) turn to move.
///
/// This will be stored in the following bitboards:
/// played_spots:
/// ................
/// ................
/// ....1.......1...
/// 111111.1111111.1
///
/// 0123456701234567 -- column index
///
/// our_spots: (We are X)
/// ................
/// ................
/// ................
/// .1111....1111...
///
/// 0123456701234567 -- column index
pub type Bitboard = u64;

#[derive(Clone)]
pub struct Position {
    // /// State of the board, with `NUM_STACKS` stacks of stones of height `STACK_HEIGHT`.
    // board: [[Stone; Self::STACK_HEIGHT as usize]; Self::NUM_STACKS as usize],
    // /// Number of stone moves played since the beginning of the game.
    // moves: u8,
    // /// The player whose turn it is.
    // current_player: Color,
    // /// Moves played throughout the game.
    // move_history: [Option<Move>; Self::MAX_MOVES as usize],
    // /// Moves which were prohibited, since "Second Best!" was called on it.
    // banned_moves: [Option<Move>; Self::MAX_MOVES as usize],
    // /// The heights of the different stacks. Stored for more efficient lookups.
    // stack_heights: [u8; Self::NUM_STACKS as usize],
    /// Bitboard containing all the played spots by either player.
    played_spots: Bitboard,
    /// Bitboard containing all the played spots by the current player.
    our_spots: Bitboard,
    /// Number of moves played throughout the game.
    num_moves: usize,
    /// History of all the stone moves played in the game.
    move_history: [Option<Bitboard>; Self::MAX_MOVES],
    /// History of all the moves which were banned.
    banned_moves: [Option<Bitboard>; Self::MAX_MOVES],
}

impl Position {
    pub const NUM_STACKS: usize = 8;
    pub const STACK_HEIGHT: usize = 3;
    pub const STONES_PER_PLAYER: usize = 8;
    pub const MAX_MOVES: usize = 255;
    // Offset to get to the right of the current stack.
    pub const RIGHT: usize = 1;
    // Offset to get to the left of the current stack.
    pub const LEFT: usize = Self::NUM_STACKS - 1;
    // Offset to get to the opposite of the current stack.
    pub const OPPOSITE: usize = Self::NUM_STACKS / 2;

    /// Bitboard with the bottom row set to ones.
    const BOTTOM: Bitboard = Self::bottom(0, 0);
    /// Bitboard with a 1 on the bottom of the first four columns.
    const BOTTOM_FOUR: Bitboard = 1
        | (1 << (Self::STACK_HEIGHT + 1))
        | (1 << ((Self::STACK_HEIGHT + 1) * 2))
        | (1 << ((Self::STACK_HEIGHT + 1) * 3));
    const COLUMN_MASKS: [Bitboard; Self::NUM_STACKS * 2] = Self::gen_column_masks(0);

    /// Create a bitboard with the bottom row set to ones
    /// recursively at compile time.
    const fn bottom(mut bb: Bitboard, col: usize) -> Bitboard {
        if col == Self::NUM_STACKS * 2 {
            return bb;
        }
        bb |= 1 << ((Self::STACK_HEIGHT + 1) * col);
        Self::bottom(bb, col + 1)
    }

    const fn gen_column_masks(col: usize) -> [Bitboard; Self::NUM_STACKS * 2] {
        let mut other_masks = if col == Self::NUM_STACKS - 1 {
            [0; Self::NUM_STACKS * 2]
        } else {
            Self::gen_column_masks(col + 1)
        };
        // Substracting 1 from 1000 gives 0111.
        // We can then shift that to the correct column.
        let mask = ((1 << Self::STACK_HEIGHT) - 1) << ((Self::STACK_HEIGHT + 1) * col)
        // We have two copies of the column in our bitboards.
        | (((1 << Self::STACK_HEIGHT) - 1)
            << ((Self::STACK_HEIGHT + 1) * (col + Self::NUM_STACKS)));
        other_masks[col] = mask;
        other_masks[col + Self::NUM_STACKS] = mask;
        other_masks
    }
}

impl Default for Position {
    fn default() -> Self {
        Position {
            played_spots: 0,
            our_spots: 0,
            num_moves: 0,
            move_history: [None; Self::MAX_MOVES],
            banned_moves: [None; Self::MAX_MOVES],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn other(&self) -> Self {
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

#[derive(Debug, PartialEq, Eq)]
pub enum GameStatus {
    /// Current player has won.
    Win,
    /// Current player has lost.
    Loss,
    /// The game is not yet over.
    OnGoing,
}

/// A type of move a player can make.
/// In case of a stone move, the Bitboard holds the
/// following information:
/// The spot where the move is going to, and the
/// spot the move is coming from are set on the bitboard.
///
/// To determine what is the "from" spot, and what is the
/// "to" spot, you need to know the state of the board.
/// The "to" spot will be free on the board, and the "from"
/// spot will be occupied on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitboardMove {
    SecondBest,
    StoneMove(Bitboard),
}

/// Move structure which is useful for interacting with the player,
/// but not very efficient for working with the bitboards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerMove {
    SecondBest,
    StoneMove { from: Option<usize>, to: usize },
}

impl Display for PlayerMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SecondBest => write!(f, "!"),
            Self::StoneMove { from, to } => match from {
                Some(from) => write!(f, "{from}-{to}"),
                None => write!(f, "{to}"),
            },
        }
    }
}

impl PlayerMove {
    /// Convert a player move to a Bitboard move.
    /// Assumes that the player move is valid.
    pub fn to_bitboard_move(self, pos: &Position) -> BitboardMove {
        match self {
            Self::SecondBest => BitboardMove::SecondBest,
            Self::StoneMove { from, to } => BitboardMove::StoneMove(pos.stone_move(from, to)),
        }
    }

    pub fn from(smove: String) -> Result<Self, MoveFailed> {
        if smove == "!" {
            return Ok(Self::SecondBest);
        }
        let numbers: Vec<&str> = smove.split('-').collect();
        match numbers.len() {
            1 => {
                let to = match numbers[0].parse() {
                    Ok(n) => n,
                    Err(_) => return Err(MoveFailed::ParseError),
                };
                Ok(Self::StoneMove { from: None, to })
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
                Ok(Self::StoneMove {
                    from: Some(from),
                    to,
                })
            }
            _ => Err(MoveFailed::ParseError),
        }
    }
}

impl BitboardMove {
    /// The column that the single bit set in the bitboard is in.
    fn column_of_bit(bb: Bitboard) -> usize {
        for col in 0..Position::NUM_STACKS {
            if bb & Position::column_mask(col) != 0 {
                return col;
            }
        }
        unreachable!("Some bit should have been set!")
    }

    pub fn to_player_move(self, pos: &Position) -> PlayerMove {
        match self {
            Self::SecondBest => PlayerMove::SecondBest,
            Self::StoneMove(smove) => {
                let from_spot = smove & pos.top_spots();
                let from = if from_spot == 0 {
                    None
                } else {
                    Some(Self::column_of_bit(from_spot))
                };
                let to_spot = smove & pos.free_spots();
                let to = Self::column_of_bit(to_spot);
                PlayerMove::StoneMove { from, to }
            }
        }
    }

    pub fn to_string(self, pos: &Position) -> String {
        self.to_player_move(pos).to_string()
    }
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
    pub fn print_bb(bb: Bitboard) {
        for row in (0..(Self::STACK_HEIGHT + 1)).rev() {
            for col in 0..(Self::NUM_STACKS * 2) {
                let mask = 1 << (col * (Self::STACK_HEIGHT + 1) + row);
                if 0 == (bb & mask) {
                    print!(".");
                } else {
                    print!("x");
                }
            }
            println!();
        }
        println!();
    }

    fn stone_move(&self, from: Option<usize>, to: usize) -> Bitboard {
        let bb = self.phase_one_stone_move(to);
        if let Some(from) = from {
            bb | (Position::column_mask(from) & (self.top_spots()))
        } else {
            bb
        }
    }

    #[inline(always)]
    fn phase_one_stone_move(&self, to: usize) -> Bitboard {
        Position::column_mask(to) & self.free_spots()
    }

    /// The empty spots just on top of the stack.
    #[inline(always)]
    pub fn free_spots(&self) -> Bitboard {
        // Why does this work?
        // Take one of the columns in the bitboard.
        // Laying it out horizontally we get something
        // like this: 0011.
        // By adding 0001 to this, we will make all the
        // bits flow over, and we end up with 0100,
        // as we wanted.
        Self::BOTTOM + self.played_spots
    }

    /// The top spots of every stack.
    #[inline(always)]
    fn top_spots(&self) -> Bitboard {
        // Shifting left drags the columns down.
        // And-ing with self.played_spots ensures we don't overflow
        // into the next column.
        // The xor gives us the bits which did not have a bit above it.
        self.played_spots ^ ((self.played_spots >> 1) & self.played_spots)
    }

    /// A mask with all the spots marked in the given column.
    #[inline(always)]
    pub fn column_mask(col: usize) -> Bitboard {
        Self::COLUMN_MASKS[col]
    }

    /// A mask with a one at the bottom of the given column.
    #[inline(always)]
    fn column_bottom_mask(col: usize) -> Bitboard {
        1 << ((Self::STACK_HEIGHT + 1) * col)
    }

    /// Bitboard with a 1 on the bottom of each stack
    /// controlled by the given player.
    #[inline(always)]
    pub fn controlled_stacks(&self, us: bool) -> Bitboard {
        let player_stones = if us {
            self.our_spots
        } else {
            self.played_spots ^ self.our_spots
        };
        // Only look at stones on top of their stack.
        let player_stones = player_stones & self.top_spots();
        // Shift everything to the bottom row.
        (player_stones & Self::BOTTOM)
            | ((player_stones >> 1) & Self::BOTTOM)
            | ((player_stones >> 2) & Self::BOTTOM)
    }

    /// Bitboard with columns set to 1 if the given player
    /// controls that column.
    #[inline(always)]
    pub fn controlled_columns(&self, us: bool) -> Bitboard {
        let controlled_stacks = self.controlled_stacks(us);
        (Self::BOTTOM << Self::STACK_HEIGHT) - controlled_stacks
    }

    /// Bitboard with a 1 set on the top of every stack controlled
    /// by the given player.
    #[inline(always)]
    pub fn from_spots(&self, us: bool) -> Bitboard {
        self.top_spots() & self.controlled_columns(us)
    }

    /// Bitboard with a 1 set on the bottom of the stacks which are still free.
    #[inline(always)]
    pub fn free_columns(&self) -> Bitboard {
        Self::BOTTOM & !(self.played_spots >> 2)
    }

    /// Bitboard with a 1 set on every free spot that would give us an alignment.
    #[inline(always)]
    pub fn vertical_alignment_spots(&self) -> Bitboard {
        // The free spots where we have two stones beneath it.
        (self.our_spots << 1) & (self.our_spots << 2) & self.free_spots()
    }

    /// The current player to move.
    pub fn current_player(&self) -> Color {
        match self.num_moves % 2 {
            0 => Color::Black,
            1 => Color::White,
            _ => unreachable!(),
        }
    }

    /// The banned move in the current position if any.
    #[inline(always)]
    pub fn banned_move(&self) -> Option<Bitboard> {
        self.banned_moves[self.num_moves + 1]
    }

    /// The number of moves that have been played in the current position.
    #[inline(always)]
    pub fn num_moves(&self) -> usize {
        self.num_moves
    }

    /// Are we in the second phase of the game, where stones are no longer placed, but moved.
    #[inline(always)]
    pub fn is_second_phase(&self) -> bool {
        self.num_moves >= 2 * Self::STONES_PER_PLAYER
    }

    /// Check if the "to" spot is adjacent or opposite to the "from" spot.
    #[inline(always)]
    fn valid_adjacent(from: usize, to: usize) -> bool {
        (from + Self::RIGHT) % Self::NUM_STACKS == to
            || (from + Self::OPPOSITE) % Self::NUM_STACKS == to
            || (from + Self::LEFT) % Self::NUM_STACKS == to
    }

    /// Is the given move banned due to a "Second Best!" call?
    #[inline(always)]
    fn is_move_banned(&self, smove: Bitboard) -> bool {
        self.banned_moves[self.num_moves + 1] == Some(smove)
    }

    /// Make the move if it is valid, otherwise return why it wasn't valid.
    pub fn try_make_move(&mut self, pmove: PlayerMove) -> Result<(), MoveFailed> {
        let (from, to) = match pmove {
            PlayerMove::SecondBest => {
                if !self.can_second_best() {
                    return Err(MoveFailed::InvalidSecondBest);
                } else {
                    self.second_best();
                    return Ok(());
                }
            }
            PlayerMove::StoneMove { from, to } => (from, to),
        };
        if self.has_alignment(false) {
            // We can't "Second Best!" and our opponent has an alignment.
            return Err(MoveFailed::PositionWinning);
        }

        if self.is_second_phase() {
            let Some(from) = from else {
                return Err(MoveFailed::MissingFromSpot);
            };
            // Check for out of bounds indices.
            if from >= Self::NUM_STACKS {
                return Err(MoveFailed::InvalidFromSpot);
            }
            if to >= Self::NUM_STACKS {
                return Err(MoveFailed::InvalidToSpot);
            }

            if to == from {
                return Err(MoveFailed::SameFromAndTo);
            }

            // We are moving a stone from the "from" spot so:
            // 1. There should be a stone at that spot
            // 2. The stone at that spot should be of the current
            //    player's color.
            if 0 == (Self::column_mask(from) & self.top_spots() & self.our_spots) {
                return Err(MoveFailed::InvalidFromSpot);
            }

            // We are moving a stone to the "to" spot, so:
            // 1. The "to" spot should be "adjacent" to the "from" spot
            // 2. The stack at the "to" spot should not be full.
            if !Self::valid_adjacent(from, to) || 0 == (self.free_spots() & Self::column_mask(to)) {
                return Err(MoveFailed::InvalidToSpot);
            }

            let smove = self.stone_move(Some(from), to);

            if self.is_move_banned(smove) {
                return Err(MoveFailed::MoveBanned);
            }

            self.make_stone_move(smove);
            return Ok(());
        }

        // There should be no "from" spot.
        if from.is_some() {
            return Err(MoveFailed::InvalidFromSpot);
        }

        // Check for out of bounds indices.
        if to >= Self::NUM_STACKS {
            return Err(MoveFailed::InvalidToSpot);
        }

        // We are moving a stone to the "to" spot, so:
        // the stack at the "to" spot should not be full.
        if 0 == (self.free_spots() & Self::column_mask(to)) {
            return Err(MoveFailed::InvalidToSpot);
        }
        let smove = self.stone_move(None, to);
        if self.is_move_banned(smove) {
            return Err(MoveFailed::MoveBanned);
        }
        self.make_stone_move(smove);
        Ok(())
    }

    /// Make a move in the given position.
    /// No checks are done whether the move is valid.
    /// For a safe version of this method, use [`try_make_move`].
    pub fn make_move(&mut self, gmove: BitboardMove) {
        match gmove {
            BitboardMove::SecondBest => self.second_best(),
            BitboardMove::StoneMove(smove) => self.make_stone_move(smove),
        }
    }

    // Unmake the last stone move or "Second Best!".
    pub fn unmake_move(&mut self) {
        if self.banned_move().is_some() {
            self.undo_second_best();
        } else {
            self.unmake_stone_move();
        }
    }

    /// Convenience function for testing mostly.
    pub fn make_phase_one_move(&mut self, to: usize) {
        self.make_stone_move(self.phase_one_stone_move(to))
    }

    pub fn make_stone_move(&mut self, smove: Bitboard) {
        // The opponents spots are the played spots where we didn't play.
        self.our_spots ^= self.played_spots;
        self.played_spots ^= smove;
        self.num_moves += 1;
        self.move_history[self.num_moves] = Some(smove);
    }

    /// Unmake the last move played.
    /// Returns the move that was undone.
    pub fn unmake_stone_move(&mut self) -> Bitboard {
        if self.num_moves == 0 {
            unreachable!("Logic error in the program");
        }
        let Some(last_move) = self.move_history[self.num_moves] else {
            unreachable!("There should be a move, because we have played that many moves.")
        };
        self.move_history[self.num_moves] = None;
        self.banned_moves[self.num_moves + 1] = None;
        self.num_moves -= 1;
        // xor-ing a second time undoes the first xor.
        self.played_spots ^= last_move;
        self.our_spots ^= self.played_spots;
        last_move
    }

    /// Check if "Second Best!" can be called this move.
    /// 1. There should be at least one move played.
    /// 2. "Second Best!" should not have been called yet this turn.
    /// 3. "Second Best!" should not have been called previous turn.
    #[inline(always)]
    pub fn can_second_best(&self) -> bool {
        self.num_moves > 0
            && self.banned_moves[self.num_moves].is_none()
            && self.banned_moves[self.num_moves + 1].is_none()
    }

    /// Opponent called "Second Best!"
    /// This should only be called if `can_second_best()` is true.
    pub fn second_best(&mut self) {
        let last_move = self.unmake_stone_move();
        self.banned_moves[self.num_moves + 1] = Some(last_move);
    }

    /// Undo a "Second Best!" call.
    /// Should only be called if there is a banned move in the current position.
    pub fn undo_second_best(&mut self) {
        let banned_move = self.banned_move().unwrap();
        self.banned_moves[self.num_moves + 1] = None;
        self.make_stone_move(banned_move);
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
            self.try_make_move(PlayerMove::from(smove)?)?;
        }
        Ok(())
    }

    #[inline(always)]
    fn stone_move_to_string(&self, smove: Bitboard) -> String {
        BitboardMove::StoneMove(smove)
            .to_player_move(self)
            .to_string()
    }

    /// Serialize the position into a string of moves.
    /// An "inverse" to `parse_and_play_moves`.
    pub fn serialize(mut self) -> String {
        let mut moves = std::vec::Vec::<String>::new();
        for move_i in (0..self.num_moves).rev() {
            let smove = self.move_history[move_i + 1].unwrap();
            self.unmake_stone_move();
            let s = self.stone_move_to_string(smove);
            moves.push(s);
            if let Some(banned_move) = self.banned_moves[move_i + 1] {
                let s = format!("{} !", self.stone_move_to_string(banned_move));
                moves.push(s);
            }
        }
        moves.reverse();
        moves.join(" ")
    }

    /// Check if the given player has an alignment on the board:
    /// Either:
    /// 1. There is a stack with three stones of the player's color.
    /// 2. There are 4 stones of the players colors next to each other
    ///    on the top of the stacks.
    pub fn has_alignment(&self, us: bool) -> bool {
        let player_stones = if us {
            self.our_spots
        } else {
            self.our_spots ^ self.played_spots
        };
        // Check for alignment in the columns:
        if (player_stones & (player_stones << 1) & (player_stones << 2)) != 0 {
            return true;
        }

        // Check for alignment on top of the stacks.
        // Step 1. "flatten" the top of the stacks to the bottom row.
        let top_of_stacks = self.controlled_stacks(us);
        // Step 2. Check for horizontal alignment.
        let mut bottom_four_mask = Self::BOTTOM_FOUR;
        for _ in 0..(Self::NUM_STACKS + 4) {
            if bottom_four_mask == (bottom_four_mask & top_of_stacks) {
                // All four stones are ours.
                return true;
            }
            bottom_four_mask <<= Self::STACK_HEIGHT + 1;
        }
        false
    }

    /// Returns true if the current player is lost.
    pub fn game_over(&self) -> bool {
        if self.can_second_best() {
            return false;
        }
        if self.has_alignment(false) {
            // Opponent has an alignment, and we can't "Second Best!".
            return true;
        }
        // From now on we just check if we are lost (this turn), this can
        // only happen if we have no legal moves.
        if !self.is_second_phase() {
            return false;
        }
        // Free columns are those where the top spot is not played.
        let free_columns = self.free_columns();
        let our_columns = self.controlled_stacks(true);
        if our_columns == 0 {
            // No controlled columns means no moves.
            return true;
        }

        let left = Self::column_bottom_mask(Self::LEFT);
        let right = Self::column_bottom_mask(Self::RIGHT);
        let opposite = Self::column_mask(Self::OPPOSITE);
        let mut possible_to = left | right | opposite;
        for from in 0..Self::NUM_STACKS {
            let from_mask = Self::column_bottom_mask(from);
            if (from_mask & our_columns) == 0 {
                // This from spot is not controlled by us.
                continue;
            }
            if (possible_to & free_columns) != 0 {
                // Found a possible move.
                return false;
            }
            // Move to the next column.
            possible_to <<= Self::STACK_HEIGHT + 1;
        }

        // No legal move, so the game is over.
        true
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
                    let mask = (1 << coordinate.1) << (coordinate.0 * (Self::STACK_HEIGHT + 1));
                    let c = if 0 != (mask & self.our_spots) {
                        self.current_player().to_string()
                    } else if 0 != (mask & self.played_spots) {
                        self.current_player().other().to_string()
                    } else {
                        ".".to_string()
                    };
                    s += &c;
                }
                s += " ";
            }
            s += "\n";
        }
        println!("{s}");
        if self.game_over() {
            println!("Game over, ({}) has won!", self.current_player().other());
            return;
        }
        println!(
            "It is {} turn to move",
            match self.current_player() {
                Color::Black => "black's (X)",
                Color::White => "white's (O)",
            }
        );
        if let Some(banned_move) = self.banned_move() {
            let banned_move = BitboardMove::StoneMove(banned_move).to_player_move(self);
            println!("Banned move: {}", banned_move);
        }
        if self.has_alignment(false) {
            println!("{} has an alignment", self.current_player().other());
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
            pos.try_make_move(PlayerMove::StoneMove {
                from: None,
                to: Position::NUM_STACKS
            }),
            Err(MoveFailed::InvalidToSpot)
        );
        for _ in 0..Position::STACK_HEIGHT {
            assert_eq!(
                pos.try_make_move(PlayerMove::StoneMove { from: None, to: 0 }),
                Ok(())
            );
        }
        assert_eq!(
            pos.try_make_move(PlayerMove::StoneMove { from: None, to: 0 }),
            Err(MoveFailed::InvalidToSpot)
        );
        assert_eq!(pos.num_moves, Position::STACK_HEIGHT);
        // Use make_stone_move the stones.
        'outer: for stack in 1..Position::NUM_STACKS {
            for _ in 0..Position::STACK_HEIGHT {
                if pos.is_second_phase() {
                    break 'outer;
                }
                assert_eq!(
                    pos.try_make_move(PlayerMove::StoneMove {
                        from: None,
                        to: stack
                    }),
                    Ok(())
                );
            }
        }
        // Now in second phase.
        assert_eq!(
            pos.try_make_move(PlayerMove::StoneMove {
                from: None,
                to: Position::NUM_STACKS - 1
            }),
            Err(MoveFailed::MissingFromSpot)
        );
        assert_eq!(
            pos.try_make_move(PlayerMove::StoneMove {
                from: Some(Position::NUM_STACKS),
                to: Position::NUM_STACKS - 1
            }),
            Err(MoveFailed::InvalidFromSpot)
        );
        assert_eq!(
            pos.try_make_move(PlayerMove::StoneMove {
                from: Some(0),
                to: Position::NUM_STACKS - 2
            }),
            Err(MoveFailed::InvalidToSpot)
        );
        assert_eq!(
            pos.try_make_move(PlayerMove::StoneMove {
                from: Some(0),
                to: Position::NUM_STACKS - 1
            }),
            Ok(()),
        );
        pos.second_best();
        assert_eq!(
            pos.try_make_move(PlayerMove::StoneMove {
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
        pos.make_phase_one_move(0);
        assert!(pos.can_second_best());
        pos.second_best();
        assert_eq!(pos.num_moves, 0);
        assert_eq!(
            pos.try_make_move(PlayerMove::StoneMove { from: None, to: 0 }),
            Err(MoveFailed::MoveBanned)
        );
        pos.make_phase_one_move(1);
        assert!(!pos.can_second_best());

        pos.make_phase_one_move(7);
        pos.make_phase_one_move(7);
        assert!(pos.can_second_best());
        pos.second_best();
        // Can't "Second Best!" a "Second Best!"
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
        assert!(!pos.has_alignment(true));
        pos.parse_and_play_moves("0 0 0".split_whitespace().map(|s| s.to_string()).collect())
            .unwrap();
        assert!(!pos.has_alignment(true));
        pos.parse_and_play_moves(
            "1 2 1 2 1"
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();
        assert!(pos.has_alignment(false));
        pos.second_best();
        pos.parse_and_play_moves(
            "2 1 3 7 4 6"
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();
        pos.show();
        assert!(pos.has_alignment(false));
    }

    #[test]
    fn game_over() {
        let mut pos = Position::default();
        assert!(!pos.game_over());
        pos.parse_and_play_moves(
            "0 1 0 1 0"
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();
        pos.show();
        // Can still call second best.
        assert!(!pos.game_over());
        pos.second_best();
        pos.parse_and_play_moves(
            "1 0 ! 7 7 ! 0"
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();
        // Can't second best.
        assert!(pos.game_over());
        pos.unmake_stone_move();
        pos.show();

        pos.parse_and_play_moves(
            "4 7 7 3 5 3 3"
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();
        pos.show();
        assert!(!pos.game_over());
        pos.make_phase_one_move(5);
        pos.make_phase_one_move(5);
        pos.make_phase_one_move(4);
        assert!(!pos.game_over());
        pos.show();
        pos.try_make_move(PlayerMove::StoneMove {
            from: Some(7),
            to: 0,
        })
        .unwrap();
        pos.show();
        assert!(!pos.game_over());
        pos.second_best();
        pos.try_make_move(PlayerMove::StoneMove {
            from: Some(0),
            to: 4,
        })
        .unwrap();
        pos.show();
        // No legal moves.
        assert!(pos.game_over());
    }

    #[test]
    fn unmake_move() {
        let mut pos = Position::default();
        pos.make_phase_one_move(0);
        pos.make_phase_one_move(0);
        pos.make_phase_one_move(1);
        pos.make_phase_one_move(1);
        pos.show();
        assert!(pos.can_second_best());
        pos.second_best();
        pos.show();
        assert!(!pos.can_second_best());
        pos.make_phase_one_move(0);
        pos.show();
        assert!(!pos.can_second_best());
        pos.unmake_stone_move();
        pos.show();
        assert!(!pos.can_second_best());
        pos.make_phase_one_move(3);
        pos.show();
        assert!(!pos.can_second_best());
        pos.unmake_stone_move();
        pos.show();
        assert!(!pos.can_second_best());
        pos.unmake_stone_move();
        assert!(pos.can_second_best());
        pos.show();
        pos.make_phase_one_move(5);
        pos.show();
        assert!(pos.can_second_best());
    }

    #[test]
    fn undo_second_best() {
        let mut pos = Position::default();
        pos.make_phase_one_move(0);
        pos.make_phase_one_move(0);
        pos.make_phase_one_move(0);
        pos.show();
        pos.second_best();
        assert!(!pos.can_second_best());
        pos.undo_second_best();
        pos.show();
        assert!(pos.can_second_best());
    }

    #[test]
    fn serialize() {
        let mut pos = Position::default();
        pos.make_phase_one_move(0);
        pos.make_phase_one_move(0);
        pos.make_phase_one_move(0);
        pos.second_best();
        pos.make_phase_one_move(1);
        pos.make_phase_one_move(1);
        pos.make_phase_one_move(1);
        let moves = pos.clone().serialize();
        println!("{moves}");
        let mut pos2 = Position::default();
        pos2.parse_and_play_moves(moves.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap();
        assert_eq!(pos.num_moves, pos2.num_moves);
        assert_eq!(pos.our_spots, pos2.our_spots);
        assert_eq!(pos.played_spots, pos2.played_spots);
        assert_eq!(pos.banned_moves, pos2.banned_moves);
        assert_eq!(pos.move_history, pos2.move_history);

        let mut pos = Position::default();
        let input_moves =
            "3 1 1 0 6 2 3 7 6 6 7 0 5 7 0 2 5-4 7-3 0-1 3-4 3-4 0-7 4-0 4-3 4-5 7-0 7-3 6-7 ! 6-5 6-7";
        pos.parse_and_play_moves(
            input_moves
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        )
        .unwrap();
        let moves = pos.clone().serialize();
        println!("{moves}");
        assert_eq!(moves, input_moves);
    }
}
