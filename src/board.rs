use std::{fmt::Display, ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr}, str::FromStr};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Board {
    white: BBoard,
    black: BBoard,

    is_black_turn: bool,

    hash: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pos(u8);

#[derive(Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum Move {
    #[default]
    Pass,
    Play(Pos), // u8 representing the index
}

const MASKS: [u64; 64] = masks();

const fn masks() -> [u64; 64] {
    let mut masks = [0; 64];
    let mut i = 0;
    while i < 64 {
        masks[i] = 1 << i;
        i += 1;
    }
    masks
}


impl Pos {
    fn mask(self) -> u64 {
        MASKS[self.0 as usize]
    }
    
    pub fn new(x: u8, y: u8) -> Pos {
        Pos((x as i8 * HORIZ + y as i8 * VERT) as u8)
    }

    pub fn index(self) -> u8 {
        self.0
    }
}

impl Move {
    fn mask(self) -> u64 {
        match self {
            Move::Pass => 0,
            Move::Play(i) => i.mask(),
        }
    }
    pub fn new(x: u8, y: u8) -> Move {
        Move::Play(Pos::new(x, y))
    }

    pub fn pos(&self) -> Pos {
        match self {
            Move::Pass => panic!("Pass move does not have a pos"),
            Move::Play(i) => *i,
        }
    } 
}

#[derive(Debug)]
pub struct MoveParseErr();

impl FromStr for Move {
    type Err = MoveParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        
        let col = chars.next().ok_or(MoveParseErr())?;
        if col == 'p' {
            return Ok(Move::Pass)
        }

        let row = chars.next().ok_or(MoveParseErr())?;

        let col = col.is_ascii_alphabetic().then(|| col.to_ascii_lowercase() as u8 - b'a').ok_or(MoveParseErr())?;
        let row = row.is_ascii_digit().then(|| row as u8 - b'1').ok_or(MoveParseErr())?;
        Ok(Move::new(col, row))
    }
}

pub enum MovesIter {
    Base(u64),
    Empty,
}

pub struct BBoardIter(u64);

pub struct Moves {
    base: u64, 
}

impl FromIterator<Move> for Moves {
    fn from_iter<T: IntoIterator<Item = Move>>(iter: T) -> Self {
        let mut base = 0;
        let iter = iter.into_iter();
        for v in iter {
            base += v.mask();
        }
        Moves {
            base,
        }
    }
}

impl Moves {
    fn new(base: u64) -> Self {
        Self {
            base,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.base == 0
    }

    pub fn len(&self) -> u8 {
        self.base.count_ones() as u8
    }

    pub fn contains(&self, mv: Move) -> bool {
        match mv {
            Move::Pass => self.is_empty(),
            Move::Play(_) => (mv.mask() & self.base) != 0,
        }
    } 
}

impl From<BBoard> for Moves {
    fn from(value: BBoard) -> Self {
        Moves {
            base: value.0
        }
    }
}

impl IntoIterator for Moves { 
    type Item = Move;
    type IntoIter = MovesIter;

    fn into_iter(self) -> Self::IntoIter {
        if self.is_empty() {
            return MovesIter::Empty;
        }

        MovesIter::Base(self.base)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BBoard(u64);

impl BBoard {
    pub fn count_ones(self) -> u8 {
        self.0.count_ones() as u8
    }
}

impl Shl<usize> for BBoard {
    type Output = BBoard;

    fn shl(self, rhs: usize) -> Self::Output {
        BBoard(self.0 << rhs)
    }
}

impl Shr<usize> for BBoard {
    type Output = BBoard;

    fn shr(self, rhs: usize) -> Self::Output {
        BBoard(self.0 >> rhs)
    }
}


impl BitXor for BBoard {
    type Output = BBoard;

    fn bitxor(self, rhs: Self) -> Self::Output {
        BBoard(self.0 ^ rhs.0)
    }
}

impl BitAnd for BBoard {
    type Output = BBoard;

    fn bitand(self, rhs: Self) -> Self::Output {
        BBoard(self.0 & rhs.0)
    }
}

impl BitOr for BBoard {
    type Output = BBoard;

    fn bitor(self, rhs: Self) -> Self::Output {
        BBoard(self.0 | rhs.0)
    }
}

impl Not for BBoard {
    type Output = BBoard;

    fn not(self) -> Self::Output {
        BBoard(!self.0)
    }
}

impl IntoIterator for BBoard {
    type Item = Pos;

    type IntoIter = BBoardIter;

    fn into_iter(self) -> Self::IntoIter {
        BBoardIter(self.0)
    }
}

impl Iterator for BBoardIter {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let i = self.0.trailing_zeros() as u8;
        self.0 &= self.0 - 1; // Clear lowest bit
        Some(Pos(i))
    } 
}

impl Iterator for MovesIter {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MovesIter::Base(0) => None,
            MovesIter::Base(base) => {
                let i = base.trailing_zeros() as u8;
                *self = MovesIter::Base(*base & (*base - 1)); // Clear lowest bit
                Some(Move::Play(Pos(i)))
            }
            MovesIter::Empty => {
                *self = MovesIter::Base(0);
                Some(Move::Pass)
            },
        }
    } 
}

const HORIZ: i8 = 1;
const VERT: i8 = 8;

fn mask(x: u8, y: u8) -> u64 {
    1 << (HORIZ * x as i8 + VERT * y as i8)
}

pub enum Win {
    Black, 
    White,
    Tie,
}


const fn ray_masks() -> [[u64; 8]; 64] {
    let dirs = [HORIZ, -HORIZ, VERT, -VERT, VERT + HORIZ, -VERT - HORIZ, VERT - HORIZ, HORIZ - VERT];
    let mut masks =  [[0u64; 8]; 64];
    let mut sq = 0;

    while sq < 64 {
        let mut dir_idx = 0;
        while dir_idx < 8 {
            let dir = dirs[dir_idx];
            let mut mask = 0;
            let mut pos = sq;
            loop {
                let next = pos + dir;
                
                if next < 0 || next >= 64 { break; }
                if pos % 8 == 7 && next % 8 == 0 { break; }
                if pos % 8 == 0 && next % 8 == 7 { break; }
                
                mask |= 1 << next;
                pos = next;
            }
            masks[sq as usize][dir_idx] = mask;
            dir_idx += 1;
        }
        sq += 1;
    }

    masks
}

static ZOBRIST_TABLE: ([[u64; 64]; 2], u64) = precompute_zobrist();
static ZOBRIST: [[u64; 64]; 2] = ZOBRIST_TABLE.0;
static ZOBRIST_TURN: u64 = ZOBRIST_TABLE.1;

const fn precompute_zobrist() -> ([[u64; 64]; 2], u64) {
    let mut table = [[0u64; 64]; 2];
    let mut state = 0x123456789abcdef;
    let mut i = 0;
    while i < 2 {
        let mut sq = 0;
        while sq < 64 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            table[i][sq] = state;
            sq += 1;
        }
        i += 1;
    }

    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    let turn = state;

    (table, turn)
}


#[derive(Clone, Copy)]
enum Axis {
    Right = 0,
    Left = 1,
    Down = 2,
    Up = 3,
    RightDown = 4,
    LeftUp = 5,
    LeftDown = 6,
    RightUp = 7,
}

const AXIS: [Axis; 8] = [Axis::Right, Axis::Left, Axis::Down, Axis::Up, Axis::RightDown, Axis::LeftUp, Axis::LeftDown, Axis::RightUp];
const NOT_A_FILE: BBoard = BBoard(0xfefefefefefefefe);
const NOT_H_FILE: BBoard = BBoard(0x7f7f7f7f7f7f7f7f);

fn shift(p: BBoard, axis: Axis) -> BBoard {
    match axis {
        Axis::Right => (p << 1) & NOT_A_FILE,
        Axis::Left =>  (p >> 1) & NOT_H_FILE,
        Axis::Down => p << 8,
        Axis::Up => p >> 8,
        Axis::RightDown => (p << 9) & NOT_A_FILE,
        Axis::LeftUp => (p >> 9) & NOT_H_FILE,
        Axis::LeftDown => (p << 7) & NOT_H_FILE,
        Axis::RightUp => (p >> 7) & NOT_A_FILE,
    }
}

impl Board { 
    pub fn black_count(&self) -> u8 {
        self.black.0.count_ones() as u8
    }

    pub fn white_count(&self) -> u8 {
        self.white.0.count_ones() as u8
    }

    pub fn start() -> Board {
        Board {
            white: BBoard(mask(3,3) | mask(4,4)),
            black: BBoard(mask(3,4) | mask(4,3)),
            is_black_turn: true,
            hash: ZOBRIST[0][28] ^ ZOBRIST[0][35] ^ ZOBRIST[1][27] ^ ZOBRIST[1][36],
        }
    }

    pub fn flipped(&self) -> Board {
        Board {
            white: self.white,
            black: self.black,
            is_black_turn: !self.is_black_turn,
            hash: self.hash,
        }
    }

    pub fn win(&self) -> Option<Win> {
        if self.legal_moves().is_empty() && self.flipped().legal_moves().is_empty() {
            let w = self.white.count_ones();
            let b = self.black.count_ones();
            return match w.cmp(&b) {
                std::cmp::Ordering::Less => Some(Win::Black),
                std::cmp::Ordering::Equal => Some(Win::Tie),
                std::cmp::Ordering::Greater => Some(Win::White),
            }
        }

        None
    }

    pub fn legal_moves(&self) -> Moves {
        let (me, opp) = self.me_opp();
        let empty = !(me | opp);
        let mut legal = BBoard(0);

        for axis in AXIS {
            let mut flood = shift(me, axis) & opp;
            
            flood = flood | shift(flood, axis) & opp;
            flood = flood | shift(flood, axis) & opp;
            flood = flood | shift(flood, axis) & opp;
            flood = flood | shift(flood, axis) & opp;
            flood = flood | shift(flood, axis) & opp;
            flood = flood | shift(flood, axis) & opp;

            legal = legal | shift(flood, axis) & empty;
        }

        legal.into()
    }

    pub fn is_legal(&self, mv: Move) -> bool {
        if mv.mask() & self.black.0 != 0 || mv.mask() & self.white.0 != 0 {
            return false;
        } 

        let pos = mv.pos();
        for axis in AXIS {
            if self.flip_mask(pos, axis) != BBoard(0) {
                return true;
            }
        }

        false
    }

    fn flip_mask(&self, start: Pos, axis: Axis) -> BBoard {
        let (me, opp) = self.me_opp();
        
        let mut flip = shift(BBoard(start.mask()), axis) & opp;
        flip = flip | (shift(flip, axis) & opp);
        flip = flip | (shift(flip, axis) & opp);
        flip = flip | (shift(flip, axis) & opp);
        flip = flip | (shift(flip, axis) & opp);
        flip = flip | (shift(flip, axis) & opp);
        flip = flip | (shift(flip, axis) & opp);

        if shift(flip, axis) & me == BBoard(0) {
            return BBoard(0)
        }

        flip
    }

    pub fn make_move(&self, mv: Move) -> Board { 
        if let Move::Pass = mv {
            return Board {
                black: self.black,
                white: self.white,
                is_black_turn: !self.is_black_turn,
                hash: self.hash,
            };
        }

        let pos = mv.pos();

        let flip_mask = 
            self.flip_mask(pos, Axis::Right) |
            self.flip_mask(pos, Axis::Left) |
            self.flip_mask(pos, Axis::Down) |
            self.flip_mask(pos, Axis::Up) |
            self.flip_mask(pos, Axis::RightDown) |
            self.flip_mask(pos, Axis::LeftUp) |
            self.flip_mask(pos, Axis::LeftDown) |
            self.flip_mask(pos, Axis::RightUp);
 
        let (mut white, mut black) = (self.white ^ flip_mask, self.black ^ flip_mask);
        let mut hash = self.hash;

        for pos in flip_mask {
            hash ^= ZOBRIST[0][pos.index() as usize];
            hash ^= ZOBRIST[1][pos.index() as usize];
        }

        if self.is_black_turn {
            hash ^= ZOBRIST[0][mv.pos().index() as usize];
            black = black | BBoard(mv.mask()); 
        } else {
            hash ^= ZOBRIST[1][mv.pos().index() as usize];
            white = white | BBoard(mv.mask());
        }

        hash ^= ZOBRIST_TURN;

        Board {
            white,
            black,
            is_black_turn: !self.is_black_turn,
            hash,
        }
    }

    pub fn is_black_turn(&self) -> bool {
        self.is_black_turn
    }

    pub fn me_opp(&self) -> (BBoard, BBoard) {
        if self.is_black_turn {
            return (self.black, self.white);
        }

        (self.white, self.black)
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }
}

impl Display for Moves {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s: String = "".into();
        for i in 0..64 {
            let b = (self.base & (1 << i)) != 0;
            s += if b {
                " "
            } else {
                " "
            };
            if i % 8 == 7 {
                s += "\n";
            }
        }
        f.write_str(&s)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s: String = "".into();
        let legal = self.legal_moves();
        for i in 0..64 {
            let b = (self.black.0 & (1 << i)) != 0;
            let w = (self.white.0 & (1 << i)) != 0;
            s += match (b, w) {
                (true, true) => unreachable!(),
                (false, true) => " ",
                (true, false) => " ",
                (false, false) => {
                    if legal.contains(Move::Play(Pos(i))) {
                        "@ "
                    } else {
                        "# "
                    }
                },
            };

            if i % 8 == 7 {
                s += "\n";
            }
        }
        f.write_str(&s)
    }
}

impl Display for Win {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Win::Black => f.write_str("black"),
            Win::White => f.write_str("white"),
            Win::Tie => f.write_str("tie"),
        }
    }
}
