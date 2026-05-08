use std::{collections::HashMap, io::{self, BufRead}, str::FromStr};

use crate::board::{Board, Move};

pub fn play() {
    let mut board = Board::start();
    let stdin = io::stdin();

    println!("{}", board);
    for line in stdin.lock().lines() {
        let mv = Move::from_str(&line.unwrap()).unwrap();
        let legal = board.legal_moves();

        if legal.contains(mv) {
            board = board.make_move(mv);
            println!("{}", board);
        } else {
            println!("Illegal move!");
        }

        if let Some(winner) = board.win() {
            println!("Game Ended, winner: {}", winner);
            break
        }
    }
}

pub fn play_veng() {
    let mut board = Board::start();
    let mut known = HashMap::new();
    let stdin = io::stdin();

    println!("{}", board);
    for line in stdin.lock().lines() {
        let mv = Move::from_str(&line.unwrap()).unwrap();
        let legal = board.legal_moves();

        if legal.contains(mv) {
            board = board.make_move(mv);
            println!("{}", board);
        } else {
            println!("Illegal move!");
            break
        }

        let mv = best_move(&mut known, board, 10);
        board = board.make_move(mv);
        println!("{}", board);

        if let Some(winner) = board.win() {
            println!("Game Ended, winner: {}", winner);
            break
        }
    }

}

type Known = HashMap<Board, i8>;


fn best_move(known: &mut Known, board: Board, depth: u8) -> Move {
    let moves = board.legal_moves();
    moves.into_iter().max_by_key(|mv| -negamax(known, board.make_move(*mv), depth, i8::MIN + 1, i8::MAX)).unwrap()
}

fn negamax(known: &mut Known, board: Board, depth: u8, mut alpha: i8, beta: i8) -> i8 {
    if board.win().is_some() || depth == 0 {
        return (2 * (board.is_black_turn() as i8) - 1) * board.black_count() as i8 - board.white_count() as i8;
    }

    let legal = board.legal_moves();
    
    let children = legal.into_iter().map(|mv| board.make_move(mv));

    let mut value = 0;
    for child in children {
        if let Some(v) = known.get(&child.canonical()) {
            value = value.max(*v);
        } else {
            let v = -negamax(known, child, depth - 1, -beta, -alpha);
            known.insert(child.canonical(), v);

            value = value.max(v);
        }

        alpha = alpha.max(value);
        if alpha >= beta {
            break
        }
    }

    value
}
