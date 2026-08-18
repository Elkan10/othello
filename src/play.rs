use std::{io::{self, BufRead}, str::FromStr, sync::atomic::{AtomicU64, Ordering}, time::Instant};

use crate::{board::{Board, Move}, eval::eval};

pub fn play_engveng() {
    let mut board = Board::start();
    let state = TTState::new(64);
    while board.win().is_none() {
        let mv = iter_deep(&state, board, 10);
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
    let mut known = TTState::new(64);

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
#[repr(u8)]
enum TTFlag {
    UpperBound = 0, 
    LowerBound = 1,
    #[default]
    Exact = 2,
}

impl TTFlag {
    fn from_u64(n: u64) -> TTFlag {
        match n {
            0 => TTFlag::UpperBound,
            1 => TTFlag::LowerBound,
            2 => TTFlag::Exact,
            _ => panic!("tried to create TTFlag from value > 2"),
        }
    }
}


#[derive(Clone, Copy, Default)]
struct TTEntry {
    hash_check: u16,
    depth: u8,
    flag: TTFlag,
    value: i16,
    mv: Move,
}

impl TTEntry {
    /// Encodes as 0bHHHHHHHHHHHHHHHHDDDDDDDDFFVVVVVVVVVVVVVVVV00000000000000MMMMMMMM
    fn pack(self) -> u64 {
        let hash_check = self.hash_check as u64;
        let depth = self.depth as u64;
        let flag = self.flag as u64;
        let value = (self.value as u16) as u64;
        let mv = self.mv.encode() as u64;

        (hash_check << 48) | (depth << 40) | (flag << 38) | (value << 22) | mv
    }

    fn unpack(n: u64) -> TTEntry {
        let hash_check = (n >> 48) as u16;
        let depth = ((n >> 40) & 0xFF) as u8;
        let flag = TTFlag::from_u64((n >> 38) & 0b11);
        let value = (((n >> 22) & 0xFFFF) as u16) as i16;
        let mv = Move::decode((n & 0xFF) as u8);

        TTEntry { hash_check, depth, flag, value, mv }
    }
}

#[derive(Default)]
struct TTSlot {
    packed: AtomicU64,
}

struct TTState {
    entries: Vec<TTSlot>,
    size: usize,
}

impl TTState {
    fn new(mb: usize) -> TTState {
        let size = (mb * 1024 * 1024) / std::mem::size_of::<TTEntry>();
        let size = size.next_power_of_two() >> 1;
        TTState {
            entries: (0..size).map(|_| TTSlot::default()).collect(),
            size,
        }
    }

    fn get(&self, hash: u64) -> Option<TTEntry> {
        let entry = &self.entries[self.index(hash)];
        let entry = TTEntry::unpack(entry.packed.load(Ordering::Relaxed));

        if entry.hash_check == (hash >> 48) as u16 {
            Some(entry)
        } else {
            None
        }
    }

    fn index(&self, hash: u64) -> usize {
        (hash as usize) & (self.size - 1)
    }

    fn insert(&self, hash: u64, entry: TTEntry) {
        let i = self.index(hash);
        self.entries[i].packed.store(entry.pack(), Ordering::Relaxed);
    }
}

fn iter_deep(state: &TTState, board: Board, depth: u8) -> Move {
    let mut mv = Move::Pass;

    for depth in 0..=depth {
        mv = best_move(state, board, depth)
    }

    mv
}

fn best_move(state: &TTState, board: Board, depth: u8) -> Move {
    let moves = board.legal_moves();
    let mut neg = NegamaxState::new(state, board, depth + 1);

    let start = Instant::now();
    let mv = moves.into_iter().max_by_key(|mv| neg.run_child(*mv)).unwrap();
    let elapsed = start.elapsed().as_secs_f64();

    println!("d={}: {} nodes in {:.2}s = {:.0} nps", depth, neg.node_count, elapsed, neg.node_count as f64 / elapsed);
    mv
}

struct NegamaxState<'a> {
    tt: &'a TTState,
    node_count: u64,

    board: Board,
    depth: u8,
    
    alpha_orig: i16,
    alpha: i16,

    beta_orig: i16,
    beta: i16,
}

enum TTResult {
    Return(i16),
    TTMove(Move, i16),
    None,
}

impl TTResult {
    fn to_opt(self) -> Option<(Move, i16)> {
        match self {
            Self::Return(_) => panic!("TTResult should return!!"),
            Self::TTMove(mv, val) => Some((mv, val)),
            Self::None => None
        }
    }
}

impl NegamaxState<'_> {
    fn new(state: &TTState, board: Board, depth: u8) -> NegamaxState<'_> {
        NegamaxState { 
            tt: state, 
            node_count: 0, 
            
            board, 
            depth, 
            
            alpha_orig: i16::MIN + 1, 
            alpha: i16::MIN + 1, 
            
            beta_orig: i16::MAX, 
            beta: i16::MAX, 
        }
    }

    fn child(&self, mv: Move) -> NegamaxState<'_> {
        NegamaxState { 
            tt: self.tt,
            node_count: 0,

            board: self.board.make_move(mv), 
            depth: self.depth - 1, 
           
            alpha_orig: -self.beta, 
            alpha: -self.beta, 
            
            beta_orig: -self.alpha, 
            beta: -self.alpha 
        }
    }

    fn update_alpha(&mut self, val: i16) {
        self.alpha = self.alpha.max(val);
    }
    
    fn update_beta(&mut self, val: i16) {
        self.beta = self.beta.min(val);
    }

    fn update_tt(&mut self, mv: Move, val: i16, color: i16) {
        let flag = if val <= self.alpha_orig {
            TTFlag::UpperBound
        } else if val >= self.beta_orig {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };
        self.tt.insert(self.board.hash(), TTEntry { hash_check: (self.board.hash() >> 48) as u16, depth: self.depth, value: val * color, flag, mv });
    }

    fn probe_tt(&mut self) -> TTResult {
        let color = 2 * (self.board.is_black_turn() as i16) - 1;

        if let Some(entry) = self.tt.get(self.board.hash()) {
            if entry.depth >= self.depth {
                let val = color * entry.value;
                match entry.flag {
                    TTFlag::Exact => return TTResult::Return(val),
                    TTFlag::UpperBound => self.update_alpha(val),
                    TTFlag::LowerBound => self.update_beta(val),
                };

                if self.alpha >= self.beta {
                    return TTResult::Return(val);
                }
            }

            let mv = entry.mv;
            let val = self.run_child(mv);
            self.update_alpha(val);

            if self.alpha >= self.beta {
                self.update_tt(mv, val, color);
                return TTResult::Return(val);
            }

            return TTResult::TTMove(mv, val);
        }

        TTResult::None
    }

    fn run_child(&mut self, mv: Move) -> i16 {
        let (val, node_count) = self.child(mv).run();
        self.node_count += node_count;
        -val
    } 

    fn run(&mut self) -> (i16, u64) {
        self.node_count += 1;
        let color = 2 * (self.board.is_black_turn() as i16) - 1;

        if self.board.win().is_some() || self.depth == 0 {
            return (eval(self.board), self.node_count);
        }


        let ttres = self.probe_tt();

        if let TTResult::Return(val) = ttres {
            return (val, self.node_count);
        }

        let tt_move = ttres.to_opt(); 

        let (mut best, mut best_value) = tt_move.unwrap_or((Move::Pass, i16::MIN + 1));

        let mut legal = self.board.legal_moves().excluding(best);

        while !legal.is_empty() {
            let mv = legal.pick(self.board);

            let v = self.run_child(mv);

            if v >= best_value {
                best = mv;
                best_value = v;
            }

            self.update_alpha(v);
            if self.alpha >= self.beta {
                break
            }
        }

        self.update_tt(best, best_value, color);
        (best_value, self.node_count)
    }
}
