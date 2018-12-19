/*
 * op.rs
 * motions and operations that comprise a norm command
 */

#[derive(Copy, Clone, Debug)]
pub enum SingleMotion {
    Beg,
    End,
    Left,
    Right,
    Word,
    Back,
    Till     { c : char },
    BackTill { c : char },
    Find     { c : char },
    BackFind { c : char },
    FindNext,
}

#[derive(Copy, Clone, Debug)]
pub struct Motion {
    pub mot    : SingleMotion,
    pub repeat : u32,
}

#[derive(Clone, Debug)]
pub enum Op {
    Motion(Motion),
    Insert { s : String },
    Append { s : String },
    LineAppend { s : String },
    Delete { motion : Motion },
    DeleteToEnd,
    Yank { motion : Motion },
    Put,
    Repeat,
}

impl SingleMotion {
    pub fn explain(&self, repeat : u32) {
        if repeat == 1 {
            match self {
                SingleMotion::Beg            => print!("to beginning of line"),
                SingleMotion::End            => print!("to end of line"),
                SingleMotion::Left           => print!("left"),
                SingleMotion::Right          => print!("right"),
                SingleMotion::Word           => print!("forward word"),
                SingleMotion::Back           => print!("backward word"),
                SingleMotion::Till     { c } => print!("forward until character '{}'", c),
                SingleMotion::BackTill { c } => print!("backward until character '{}'", c),
                SingleMotion::Find     { c } => print!("forward to character '{}'", c),
                SingleMotion::BackFind { c } => print!("backward to character '{}'", c),
                SingleMotion::FindNext       => print!("to next occurance of target from previous 't/T' or 'f/F' command"),
            }
        } else {
            match self {
                SingleMotion::Beg            => print!("to beginning of line"),
                SingleMotion::End            => print!("to end of line"),
                SingleMotion::Left           => print!("left {}", repeat),
                SingleMotion::Right          => print!("right {}", repeat),
                SingleMotion::Word           => print!("forward {} words", repeat),
                SingleMotion::Back           => print!("backward {} words", repeat),
                SingleMotion::Till     { c } => print!("forward until {} occurances of character '{}'", repeat, c),
                SingleMotion::BackTill { c } => print!("backward until {} occurances of character '{}'", repeat, c),
                SingleMotion::Find     { c } => print!("forward to {} occurances of character '{}'", repeat, c),
                SingleMotion::BackFind { c } => print!("backward to {} occurances of character '{}'", repeat, c),
                SingleMotion::FindNext       => print!("to {} next occurances of target from previous 't' or 'f' command ", repeat),
            }
        }
    }
}

impl Motion {
    pub fn explain(&self) {
        self.mot.explain(self.repeat);
    }
}

impl Op {
    pub fn explain(&self) {
        match self {
            Op::Motion(motion)   => { print!("- go "); motion.explain(); println!(); },
            Op::Insert{ s }      =>   println!("- insert '{}' at the current cursor location", s),
            Op::Append{ s }      =>   println!("- append '{}' after the current cursor location", s),
            Op::LineAppend{ s }  =>   println!("- append '{}' at the end of the line", s),
            Op::Delete{ motion } => { print!("- delete "); motion.explain(); println!(); },
            Op::DeleteToEnd      =>   println!("- delete from current cursor position to the end of the line"),
            Op::Yank{ motion }   => { print!("- yank "); motion.explain(); println!(); },
            Op::Put              =>   println!("- put yanked text at the current cursor location"),
            Op::Repeat           =>   println!("- repeat last non-motion action"),
        }
    }

    pub fn get_motion(&self) -> &Motion {
        match self {
            Op::Motion(m) => m,
            _             => panic!("internal error: get_motion() on non-motion op")
        }
    }

    pub fn get_s(&self) -> &String {
        match self {
              Op::Insert     { s }
            | Op::Append     { s }
            | Op::LineAppend { s } => s,
            _                      => panic!("get_s on non-insert op")
        }
    }

    pub fn get_repeat(&self) -> u32 { self.get_motion().repeat }
    pub fn get_c(&self) -> char {
        let m = self.get_motion();

        match m.mot {
              SingleMotion::Till     { c }
            | SingleMotion::BackTill { c }
            | SingleMotion::Find     { c }
            | SingleMotion::BackFind { c } => c,
            _                              => panic!("get_c() on simple motion")
        }
    }
}
