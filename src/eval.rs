use crate::position::Position;

pub const WIN: isize = 1000;
pub const LOSS: isize = -WIN;

/// Return a static evaluation of the position.
pub fn static_eval(pos: &Position) -> isize {
    // For now just count how many stacks are controlled by each player.
    let mut score = 0;
    for (stack_i, stack) in pos.board().iter().enumerate() {
        let height = pos.stack_height(stack_i as u8);
        if height == 0 {
            continue;
        }
        let top_of_stack = stack[height as usize - 1];
        match top_of_stack {
            None => continue,
            Some(color) => {
                if color == pos.current_player() {
                    score += height as isize;
                } else {
                    score -= height as isize;
                }
            }
        }
    }
    if pos.player_has_alignment(pos.current_player()) {
        score += 10;
    }
    if pos.player_has_alignment(pos.current_player().switch()) {
        score -= 10;
    }
    score
}

#[inline]
pub fn loss_score(pos: &Position) -> isize {
    LOSS + pos.num_moves() as isize
}
