use crate::{
    eval::{self, decode_eval, ExplainableEval},
    position::{BitboardMove, PlayerMove, Position},
};

/// A compact storage of a move in 8 bits.
/// The bits are decomposed as follows:
/// 1000_0000 -- Determines if the move is "Second Best!"
/// 0111_0000 -- The column of the to spot.
/// 0000_1111 -- The column of the "from" spot.
/// If the "from" spot is None, then 8 (0b0000_1000) is
/// stored as the from spot.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TTMove(u8);

impl TTMove {
    const FROM_BITS: u8 = 0b0000_1111;
    const TO_BITS: u8 = 0b0111_0000;
    const SECOND_BEST_BIT: u8 = 0b1000_0000;

    fn from(&self) -> Option<usize> {
        let from_stack = (self.0 & Self::FROM_BITS) as usize;
        if from_stack == 8 {
            return None;
        }
        Some(from_stack)
    }

    fn to(&self) -> usize {
        ((self.0 & Self::TO_BITS) >> 4) as usize
    }

    fn is_second_best(&self) -> bool {
        (self.0 & Self::SECOND_BEST_BIT) != 0
    }

    fn from_bitboard_move(pos: &Position, bmove: BitboardMove) -> Self {
        match bmove {
            BitboardMove::SecondBest => Self(Self::SECOND_BEST_BIT),
            BitboardMove::StoneMove(smove) => {
                let from_spot = smove & pos.from_spots(true);
                let to_spot = smove & pos.free_spots();
                // Add a sentinel bit to ensure that `from_stack` becomes
                // 8 if `from_spot` was empty.
                let from_spot = from_spot | 1 << 32;
                let from_stack = (from_spot.trailing_zeros() / 4) as u8;
                let to_stack = (to_spot.trailing_zeros() / 4) as u8;
                Self(from_stack | (to_stack << 4))
            }
        }
    }

    fn to_bitboard_move(self, pos: &Position) -> BitboardMove {
        if self.is_second_best() {
            return BitboardMove::SecondBest;
        }
        BitboardMove::StoneMove({
            let smove = pos.free_spots() & Position::column_mask(self.to());
            match self.from() {
                Some(from) => smove | pos.from_spots(true) & Position::column_mask(from),
                None => smove,
            }
        })
    }

    fn to_player_move(self) -> PlayerMove {
        if self.is_second_best() {
            return PlayerMove::SecondBest;
        }
        PlayerMove::StoneMove {
            from: self.from(),
            to: self.to(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum EntryType {
    #[default]
    Undetermined,
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    score: i16,
    best_move: TTMove,
    entry_type: EntryType,
    ply: u8,
}

impl Entry {
    fn new(
        pos: &Position,
        score: i16,
        best_move: BitboardMove,
        entry_type: EntryType,
        ply: u8,
    ) -> Self {
        Self {
            score,
            best_move: TTMove::from_bitboard_move(pos, best_move),
            entry_type,
            ply,
        }
    }

    /// The score for mate evals depends on the ply.
    pub fn score(&self, ply: isize) -> isize {
        if self.score as isize >= eval::IS_WIN {
            self.score as isize - ply
        } else if self.score as isize <= eval::IS_LOSS {
            self.score as isize + ply
        } else {
            self.score as isize
        }
    }
    pub fn best_move(&self, pos: &Position) -> BitboardMove {
        self.best_move.to_bitboard_move(pos)
    }

    pub fn best_move_for_printing(&self) -> PlayerMove {
        self.best_move.to_player_move()
    }

    pub fn entry_type(&self) -> EntryType {
        self.entry_type
    }

    pub fn ply(&self) -> usize {
        self.ply as usize
    }
}

type Key = u64;
type PartialKey = u32;

/// Simple implementation of a transposition table.
/// The idea is that multiple move orders can lead
/// to the same position. By caching positions in
/// the table, we avoid making the same calculations
/// multiple times. Additionaly, we can use results
/// from a previous iteration of the iterative
/// deepening loop to get a quicker result.
pub struct TranspositionTable {
    entries: Box<[Entry]>,
    /// We store the keys in the table as well,
    /// to be able to detect collisions.
    /// We only store a truncated key as that is
    /// enough to ensure we don't retreive a wrong
    /// entry from the table.
    /// This works because of the Chinese remainder
    /// theorem: since 2^32 and the size of the
    /// transposition table are coprime, there is
    /// a unique key less than 2^32*size such that
    /// key % 2^32 = key % size.
    keys: Box<[PartialKey]>,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self {
            entries: (0..Self::SIZE).map(|_| Entry::default()).collect(),
            // Ensure that the initial keys stored are not valid.
            keys: (0..Self::SIZE)
                .map(|_| Self::SIZE as PartialKey + 1)
                .collect(),
        }
    }
}

/// The following are functions to find the next prime factor at compile time

const fn med(min: u64, max: u64) -> u64 {
    (min + max) / 2
}

/// Checks if `n` has a a divisor between `min` (inclusive) and `max` (exclusive)
const fn has_factor(n: u64, min: u64, max: u64) -> bool {
    if min * min > n {
        false
    }
    // do not search for factor above sqrt(n)
    else if min + 1 >= max {
        n % min == 0
    } else {
        has_factor(n, min, med(min, max)) || has_factor(n, med(min, max), max)
    }
}

/// Return the next prime number greater or equal to `n`.
/// `n` must be >= 2
const fn next_prime(n: u64) -> u64 {
    if has_factor(n, 2, n) {
        next_prime(n + 1)
    } else {
        n
    }
}

impl TranspositionTable {
    const SIZE: usize = next_prime(1 << 23) as usize;

    #[inline(always)]
    fn index(&self, key: Key) -> usize {
        // Make the keys a bit more spread out.
        key as usize % Self::SIZE
    }

    /// Get a unique key for the given position.
    /// The reason the key is unique, is that we can
    /// reconstruct the position of both player's stones
    /// from the key:
    /// - First note that `our_spots` and `free_spots`
    ///   are disjoint from each other.
    /// - So we can find the free spots by looking at the
    ///   top set bits on each stack.
    /// - Once we know free_spots, we can get played_spots
    ///   by subtracting [`Self::BOTTOM`].
    /// - We can also get `our_spots` by removing `free_spots`
    ///   from the key.
    ///
    /// Because of "Second Best!" we also need to know the last
    /// move, and whether or not "Second Best!" is possible.
    pub fn key(pos: &Position) -> Key {
        let u32mask = 0xFFFF_FFFF;
        let last_move_info = match pos.last_stone_move() {
            Some(smove) => smove & !u32mask,
            None => 0,
        };
        let second_best_info = match pos.can_second_best() {
            true => last_move_info,
            false => last_move_info | (1 << 35),
        };
        let pos_info = (pos.our_spots() | pos.free_spots()) & u32mask;
        second_best_info | pos_info
    }

    /// Store a score and move in the transposition table.
    /// The position is needed to efficiently encode the move, and
    /// to calculate the key.
    pub fn store(
        &mut self,
        pos: &Position,
        score: isize,
        best_move: BitboardMove,
        entry_type: EntryType,
        ply: usize,
    ) {
        let key = Self::key(pos);
        let index = self.index(key);
        let score = match decode_eval(score, pos.ply() as isize) {
            ExplainableEval::Undetermined(_) => score,
            ExplainableEval::Win(_) => score + pos.ply() as isize,
            ExplainableEval::Loss(_) => score - pos.ply() as isize,
        };
        self.entries[index] = Entry::new(pos, score as i16, best_move, entry_type, ply as u8);
        self.keys[index] = key as PartialKey;
    }

    /// Try to get a stored score from the transposition table.
    /// If the position was not yet in the table, `None` is returned.
    pub fn get(&self, pos: &Position) -> Option<Entry> {
        let key = Self::key(pos);
        let index = self.index(key);
        if self.keys[index] == key as PartialKey {
            return Some(self.entries[index]);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::*;

    #[test]
    fn tt_move() {
        let mut pos = Position::default();
        let bmove = BitboardMove::StoneMove(pos.stone_move(None, 0));
        assert_eq!(
            TTMove::from_bitboard_move(&pos, bmove).to_bitboard_move(&pos),
            bmove
        );
        pos.make_move(bmove);
        assert_eq!(
            TTMove::from_bitboard_move(&pos, BitboardMove::SecondBest).to_bitboard_move(&pos),
            BitboardMove::SecondBest
        );
        for to in [1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 7, 0] {
            let bmove = BitboardMove::StoneMove(pos.stone_move(None, to));
            assert_eq!(
                TTMove::from_bitboard_move(&pos, bmove).to_bitboard_move(&pos),
                bmove
            );
            pos.make_move(bmove);
        }
        pos.show();
        for (from, to) in [(1, 2), (0, 1), (3, 5)] {
            pos.show();
            let bmove = BitboardMove::StoneMove(pos.stone_move(Some(from), to));
            assert_eq!(
                TTMove::from_bitboard_move(&pos, bmove).to_bitboard_move(&pos),
                bmove
            );
            pos.make_move(bmove);
        }
    }

    #[test]
    fn tt_entries() {
        let mut pos = Position::default();
        let mut tt = TranspositionTable::default();
        for to in 0..8 {
            let bmove = BitboardMove::StoneMove(pos.stone_move(None, to));

            tt.store(&pos, 0, bmove, EntryType::Exact, 0);
            assert_eq!(tt.get(&pos).unwrap().best_move(&pos), bmove);
            pos.make_move(bmove);
        }
        for _ in 0..8 {
            pos.unmake_move();
        }
        pos.show();
        for to in 0..8 {
            let bmove = BitboardMove::StoneMove(pos.stone_move(None, to));

            pos.make_move(bmove);
            pos.second_best();
            let bmove = BitboardMove::StoneMove(pos.stone_move(None, (1 + to) % 8));
            tt.store(&pos, 0, bmove, EntryType::Exact, 0);
            assert_eq!(tt.get(&pos).unwrap().best_move(&pos), bmove);
            pos.make_move(bmove);
        }
        for _ in 0..(8 * 3) {
            pos.unmake_move();
        }
        pos.show();
        for to in 0..8 {
            pos.show();
            dbg!(pos.can_second_best());
            let bmove = BitboardMove::StoneMove(pos.stone_move(None, to));
            assert_eq!(tt.get(&pos).unwrap().best_move(&pos), bmove);
            pos.make_move(bmove);
        }
    }

    #[test]
    fn second_best() {
        let mut pos = Position::default();
        let mut tt = TranspositionTable::default();
        pos.make_phase_one_move(1);
        pos.make_phase_one_move(2);
        tt.store(
            &pos,
            0,
            BitboardMove::SecondBest,
            EntryType::Undetermined,
            0,
        );
        pos.unmake_move();
        pos.unmake_move();
        pos.make_phase_one_move(2);
        pos.make_phase_one_move(1);
        // Even though the position is the "same", calling "Second Best!" would
        // have a different effect, and hence the score might be different.
        assert_eq!(tt.get(&pos), None);
    }
}
