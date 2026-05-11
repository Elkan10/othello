use crate::board::{BBoard, Board, Move};

const SQUARE_WEIGHTS: [i16; 64] = [
    100, -20, 10,  5,  5, 10, -20, 100,
    -20, -40, -5, -5, -5, -5, -40, -20,
     10,  -5,  5,  1,  1,  5,  -5,  10,
      5,  -5,  1,  0,  0,  1,  -5,   5,
      5,  -5,  1,  0,  0,  1,  -5,   5,
     10,  -5,  5,  1,  1,  5,  -5,  10,
    -20, -40, -5, -5, -5, -5, -40, -20,
    100, -20, 10,  5,  5, 10, -20, 100,
];

pub fn square_weights(bboard: BBoard) -> i16 {
    let mut score = 0;

    for pos in bboard.into_iter() {
        score += SQUARE_WEIGHTS[pos.index() as usize];
    }

    score
}

pub fn frontier_discs(bboard: BBoard, empty: BBoard) -> i16 {
    let neighbors = (bboard << 1) | (bboard >> 1) | 
                    (bboard << 8) | (bboard >> 8) |
                    (bboard << 7) | (bboard >> 7) |
                    (bboard << 9) | (bboard >> 9);
    
    (neighbors & empty).count_ones() as i16
}

pub fn eval(board: Board) -> i16 {
    let (me, opp) = board.me_opp();
    let empty = !(me | opp);

    if empty.count_ones() <= 16 {
        return me.count_ones() as i16 - opp.count_ones() as i16;
    }

    let frontier = frontier_discs(me, empty) - frontier_discs(opp, empty);
    let pos = square_weights(me) - square_weights(opp);

    3 * pos + 5 * frontier
}

pub fn order(board: Board, mv: Move) -> i16 {
    let pos = match mv {
        Move::Play(pos) => SQUARE_WEIGHTS[pos.index() as usize],
        Move::Pass => return 0,
    };

    let (me, opp) = board.me_opp();
    let empty = !(me | opp);

    let front = frontier_discs(me, empty) - frontier_discs(opp, empty);

    pos + 10 * front
}
