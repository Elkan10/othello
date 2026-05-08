use std::{fmt::Display, str::FromStr};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Board {
    white: u64,
    black: u64,

    is_black_turn: bool,
}

#[derive(Clone, Copy)]
pub enum Move {
    Pass,
    Play(u8), // u8 representing the index
}

impl Move {
    fn mask(self) -> u64 {
        match self {
            Move::Pass => 0,
            Move::Play(i) => 1 << i,
        }
    }
    pub fn new(x: u8, y: u8) -> Move {
        Move::Play((x as i8 * HORIZ + y as i8 * VERT) as u8)
    }

    fn pos(&self) -> u8 {
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
pub struct MovesIter {
    base: u64, 
    index: u8
}

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

impl IntoIterator for Moves { 
    type Item = Move;
    type IntoIter = MovesIter;

    fn into_iter(self) -> Self::IntoIter {
        MovesIter {
            base: self.base,
            index: 0,
        }
    }
}

impl Iterator for MovesIter {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 && self.base == 0 {
            self.index += 1;
            return Some(Move::Pass)
        }

        if self.index == 0 && self.base % 2 == 1 {
            self.index += 1;
            return Some(Move::Play(0));
        }

        if self.index >= 63 {
            return None
        }

        self.index += (self.base >> (self.index + 1)).trailing_zeros() as u8 + 1;
 
        if self.index > 63 {
            return None
        }

        Some(Move::Play(self.index))
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

trait Flip: Sized {
    fn flip_vertical(self) -> Self;
    fn flip_horizontal(self) -> Self;
    fn transpose(self) -> Self;

    fn rot_ccw(self) -> Self {
        self.transpose().flip_vertical()
    }

    fn rot_cw(self) -> Self {
        self.transpose().flip_horizontal()
    }
}

impl Flip for u64 {
    fn flip_vertical(self) -> u64 {
       self.swap_bytes() 
    }

    fn flip_horizontal(self) -> u64 {
        let mut x = self;
        x = ((x >> 1) & 0x5555555555555555) | ((x & 0x5555555555555555) << 1);
        x = ((x >> 2) & 0x3333333333333333) | ((x & 0x3333333333333333) << 2);
        x = ((x >> 4) & 0x0F0F0F0F0F0F0F0F) | ((x & 0x0F0F0F0F0F0F0F0F) << 4);
        x
    }

    fn transpose(self) -> u64 {
        let mut t;
        let mut x = self;

        t = (x ^ (x >> 7)) & 0x00AA00AA00AA00AA;
        x ^= t ^ (t << 7);

        t = (x ^ (x >> 14)) & 0x0000CCCC0000CCCC;
        x ^= t ^ (t << 14);
        
        t = (x ^ (x >> 28)) & 0x000000000F0F0F0F;
        x ^= t ^ (t << 28);

        x
    }
}

impl Flip for Board {
    fn flip_vertical(self) -> Self {
        Board {
            black: self.black.flip_vertical(),
            white: self.white.flip_vertical(),
            is_black_turn: self.is_black_turn
        }
    }

    fn flip_horizontal(self) -> Self {
        Board {
            black: self.black.flip_horizontal(),
            white: self.white.flip_horizontal(),
            is_black_turn: self.is_black_turn
        }
    }

    fn transpose(self) -> Self {
        Board {
            black: self.black.transpose(),
            white: self.white.transpose(),
            is_black_turn: self.is_black_turn
        }
    }
}

impl Board {
    pub fn canonical(&self) -> Board {
        let vert = self.flip_vertical();
        let horiz = self.flip_horizontal();
        let ccw = self.rot_ccw();
        let cw = self.rot_cw();

        *[vert, horiz, ccw, cw].iter().max_by_key(|b| ((b.black as u128) << 64) | b.white as u128).unwrap()    
    }

    pub fn black_count(&self) -> u8 {
        self.black.count_ones() as u8
    }

    pub fn white_count(&self) -> u8 {
        self.white.count_ones() as u8
    }

    pub fn start() -> Board {
        Board {
            white: mask(3,3) + mask(4,4),
            black: mask(3,4) + mask(4,3),
            is_black_turn: true,
        }
    }

    pub fn flipped(&self) -> Board {
        Board {
            white: self.white,
            black: self.black,
            is_black_turn: !self.is_black_turn
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
        let out: Moves = Moves::new(u64::MAX).into_iter().filter(|x| self.is_legal(*x)).collect();
        
        out
    }

    pub fn is_legal(&self, mv: Move) -> bool {
        if mv.mask() & self.black != 0 || mv.mask() & self.white != 0 {
            return false;
        } 

        let (me, opp) = if self.is_black_turn {
            (self.black, self.white)
        } else {
            (self.white, self.black)
        };

        let pos = mv.pos();
        if Board::flip_mask(pos, me, opp, HORIZ) != 0 {
            return true;
        }
        if Board::flip_mask(pos, me, opp, -HORIZ) != 0 {
            return true;
        }

        if Board::flip_mask(pos, me, opp, VERT) != 0 {
            return true;
        }
        if Board::flip_mask(pos, me, opp, -VERT) != 0 {
            return true;
        }

        if Board::flip_mask(pos, me, opp, HORIZ + VERT) != 0 {
            return true;
        }
        if Board::flip_mask(pos, me, opp, -HORIZ - VERT) != 0 {
            return true;
        }

        if Board::flip_mask(pos, me, opp, VERT - HORIZ) != 0 {
            return true;
        }
        if Board::flip_mask(pos, me, opp, -VERT + HORIZ) != 0 {
            return true;
        }

        false
    }

    fn flip_mask(start: u8, me: u64, opp: u64, axis: i8) -> u64 {
        let mut flip = 0;
        let mut pos = start as i8;

        loop { 
            if axis.rem_euclid(8) == 1 && (pos % 8) == 7 {
                return 0
            }

            if axis.rem_euclid(8) == 7 && (pos % 8) == 0 {
                return 0
            }

            if axis > 1 && pos >= 56 {
                return 0
            }

            if axis < -1 && pos < 8 {
                return 0
            }

            pos += axis;

            if (opp & (1 << pos)) == 0 {
                break
            } 
            flip += 1 << pos;
        }

        if (me & (1 << pos)) == 0 {
            return 0
        }

        flip
    }

    pub fn make_move(&self, mv: Move) -> Board { 
        if let Move::Pass = mv {
            return Board {
                black: self.black,
                white: self.white,
                is_black_turn: !self.is_black_turn,
            };
        }

        let (me, opp) = if self.is_black_turn {
            (self.black, self.white)
        } else {
            (self.white, self.black)
        };

        let pos = mv.pos();

        let flip_mask = 
            Board::flip_mask(pos, me, opp, HORIZ) +
            Board::flip_mask(pos, me, opp, -HORIZ) +
            Board::flip_mask(pos, me, opp, VERT) +
            Board::flip_mask(pos, me, opp, -VERT) +
            Board::flip_mask(pos, me, opp, VERT + HORIZ) +
            Board::flip_mask(pos, me, opp, -VERT - HORIZ) +
            Board::flip_mask(pos, me, opp, VERT - HORIZ) +
            Board::flip_mask(pos, me, opp, -VERT + HORIZ);

        let (mut white, mut black) = (self.white ^ flip_mask, self.black ^ flip_mask);

        if self.is_black_turn {
            black += mv.mask(); 
        } else {
            white += mv.mask();
        }

        Board {
            white,
            black,
            is_black_turn: !self.is_black_turn
        }
    }

    pub fn is_black_turn(&self) -> bool {
        self.is_black_turn
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
            let b = (self.black & (1 << i)) != 0;
            let w = (self.white & (1 << i)) != 0;
            s += match (b, w) {
                (true, true) => unreachable!(),
                (false, true) => " ",
                (true, false) => " ",
                (false, false) => {
                    if legal.contains(Move::Play(i)) {
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
