use std::{io::{self, BufRead}, str::FromStr, time::Instant};

use crate::{board::{Board, Move}, eval::{eval, order}};

pub fn play_engveng() {
    let mut board = Board::start();
    let mut known = Known::new(64);
    while board.win().is_none() {
        let mv = iter_deep(&mut known, board, 10);
        board = board.make_move(mv);
    }
}


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
    let mut known = Known::new(64);

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

        let mv = iter_deep(&mut known, board, 10);
        board = board.make_move(mv);
        println!("{}", board);

        if let Some(winner) = board.win() {
            println!("Game Ended, winner: {}", winner);
            break
        }
    }

}

#[derive(Clone, Copy)]
#[derive(Default)]
enum TTFlag {
    UpperBound, 
    LowerBound,
    #[default]
    Exact,
}


#[derive(Clone, Copy, Default)]
struct TTEntry {
    hash: u64,
    depth: u8,
    value: i16,
    flag: TTFlag,
    mv: Move,
}

struct Known {
    entries: Vec<TTEntry>,
    size: usize,
    node_count: u32,
}

impl Known {
    fn new(mb: usize) -> Known {
        let size = (mb * 1024 * 1024) / std::mem::size_of::<TTEntry>();
        let size = size.next_power_of_two() >> 1;
        Known {
            entries: vec![TTEntry::default(); size],
            size,
            node_count: 0,
        }
    }

    fn get(&self, hash: u64) -> Option<&TTEntry> {
        let entry = &self.entries[self.index(hash)];
        if entry.hash == hash {
            Some(entry)
        } else {
            None
        }
    }

    fn index(&self, hash: u64) -> usize {
        (hash as usize) & (self.size - 1)
    }

    fn insert(&mut self, hash: u64, entry: TTEntry) {
        let i = self.index(hash);
        self.entries[i] = entry;
    }
}

fn iter_deep(known: &mut Known, board: Board, depth: u8) -> Move {
    let mut mv = Move::Pass;

    for depth in 0..=depth {
        mv = best_move(known, board, depth)
    }

    mv
}

fn best_move(known: &mut Known, board: Board, depth: u8) -> Move {
    let moves = board.legal_moves();
    known.node_count = 0;

    let start = Instant::now();
    let mv = moves.into_iter().max_by_key(|mv| -negamax(known, board.make_move(*mv), depth, 0, i16::MIN + 1, i16::MAX)).unwrap();
    let elapsed = start.elapsed().as_secs_f64();

    println!("d={}: {} nodes in {:.2}s = {:.0} nps", depth, known.node_count, elapsed, known.node_count as f64 / elapsed);
    mv
}


const MAX_MOVES: usize = 64;

fn negamax(known: &mut Known, board: Board, depth: u8, depth_up: u8, mut alpha: i16, mut beta: i16) -> i16 {
    known.node_count += 1;

    let alpha_orig = alpha;

    let color = 2 * (board.is_black_turn() as i16) - 1;

    if board.win().is_some() || depth == 0 {
        return eval(board);
    }

    let mut value = i16::MIN;
    let mut tt_move = None;
    
    if let Some(entry) = known.get(board.hash()) {
        if entry.depth >= depth {
            let entry_val = color * entry.value;

            match entry.flag {
                TTFlag::Exact => return entry_val,
                TTFlag::LowerBound => alpha = alpha.max(entry_val),
                TTFlag::UpperBound => beta = beta.min(entry_val),
            }

            if alpha >= beta {
                return entry_val;
            }
        }

        tt_move = Some(entry.mv);
        value = -negamax(known, board.make_move(entry.mv), depth - 1, depth_up + 1, -beta, -alpha);
        alpha = alpha.max(value);

        if alpha >= beta {
            return value;
        }
    }

    let mut legal = board.legal_moves();

    let mut best = tt_move.unwrap_or(Move::Pass);

    while !legal.is_empty() {
        let mv = legal.pick(board);

        if let Some(ttmv) = tt_move && ttmv == mv {
            continue;
        }

        let v = -negamax(known, board.make_move(mv), depth - 1, depth_up + 1, -beta, -alpha);
        if v >= value {
            best = mv;
            value = v;
        }

        alpha = alpha.max(value);
        if alpha >= beta {
            break
        }
    }

    let flag = if value <= alpha_orig {
        TTFlag::UpperBound
    } else if value >= beta {
        TTFlag::LowerBound
    } else {
        TTFlag::Exact
    };

    known.insert(board.hash(), TTEntry { hash: board.hash(), depth, value: color * value, flag, mv: best });

    value
}
