/*
 * exec.rs
 * execute the commands described in op.rs
 */

#![allow(non_camel_case_types)]

use crate::op::*;

extern crate page_size;
extern crate rayon;

use std::io::{self, BufRead};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

struct Outputer {
    pub allow  : usize,
    pub buf_wr : Box<io::Write + Send>
}

impl Outputer {
    fn new() -> Outputer {
        Outputer {
            allow  : 0,
            buf_wr : Box::new(io::BufWriter::new(io::stdout()))
        }
    }
}

struct Worker {
    pos    : usize,
    cursor : usize,
    pb     : String,
    last_f : Option<Motion>,
    last_o : Option<Op>,
    lines  : Vec<String>
}

impl Worker {
    fn execute_Beg(&mut self, _op : &Op, _line : usize) {
        self.cursor = 0;
    }

    fn execute_End(&mut self, _op : &Op, line : usize) {
        if self.lines[line].len() > 0 {
            self.cursor = self.lines[line].len() - 1;
        }
    }

    fn execute_Left(&mut self, _op : &Op, _line : usize) {
        if self.cursor > 0 {
            self.cursor -= 1; 
        }
    }
    
    fn execute_Right(&mut self, _op : &Op, line : usize) {
        if self.cursor < self.lines[line].len() - 1 {
            self.cursor += 1;
        }
    }
   
    fn execute_Word(&mut self, _op : &Op, line : usize) {
        if self.lines[line].len() > 0 {
            let c = self.lines[line].chars().nth(self.cursor).unwrap();

            if c.is_alphanumeric() || c == '_' {
                while let Some(ch) = self.lines[line].chars().nth(self.cursor) {
                    if ch.is_alphanumeric() || ch == '_' { self.cursor += 1; }
                    else                                 { break; }
                }
                while let Some(ch) = self.lines[line].chars().nth(self.cursor) {
                    if ch.is_whitespace() { self.cursor += 1; } else { break; }
                }
            } else if c.is_whitespace() {
                while let Some(ch) = self.lines[line].chars().nth(self.cursor) {
                    if ch.is_whitespace() { self.cursor += 1; } else { break; }
                }
            } else {
                while let Some(ch) = self.lines[line].chars().nth(self.cursor) {
                    if ch.is_alphanumeric() || ch == '_'    { break; }
                    self.cursor += 1;
                }
            }
        }
    }
    
    fn execute_Back(&mut self, _op : &Op, line : usize) {
        if self.lines[line].len() > 0 && self.cursor > 0 {
            let c = self.lines[line].chars().nth(self.cursor - 1).unwrap();

            if c.is_alphanumeric() || c == '_' {
                while let Some(ch) = self.lines[line].chars().nth(self.cursor - 1) {
                    if ch.is_alphanumeric() || ch == '_' { self.cursor -= 1; }
                    else                                 { break; }
                    if self.cursor == 0 { break; }
                }
            } else if c.is_whitespace() {
                while let Some(ch) = self.lines[line].chars().nth(self.cursor - 1) {
                    if ch.is_whitespace() { self.cursor -= 1; } else { break; }
                    if self.cursor == 0 { break; }
                }
                if self.cursor > 0 {
                    while let Some(ch) = self.lines[line].chars().nth(self.cursor - 1) {
                        if ch.is_alphanumeric() || ch == '_' { self.cursor -= 1; }
                        else                                 { break; }
                        if self.cursor == 0 { break; }
                    }
                }
            } else {
                while let Some(ch) = self.lines[line].chars().nth(self.cursor - 1) {
                    if ch.is_alphanumeric() || ch == '_'    { break; }
                    self.cursor -= 1;
                    if self.cursor == 0 { break; }
                }
            }
        }
    }

    fn execute_Till(&mut self, op : &Op, line : usize) {
        let c = op.get_c();

        if let Some(next) = self.lines[line].chars().nth(self.cursor + 1) {
            let advance = if next == c { 2 } else { 1 };
            let remaining : String = self.lines[line].chars().skip(self.cursor + advance).collect();

            if let Some(p) = remaining.find(c)    { self.cursor += p + (advance - 1); }
        }

        self.last_f = Some(op.get_motion().clone()); 
    }
    
    fn execute_BackTill(&mut self, op : &Op, line : usize) {
        let c = op.get_c();
    
        if self.cursor > 0 {
            let s : String = self.lines[line].chars().take(self.cursor).collect();
        
            if let Some(p) = s.rfind(c)    { self.cursor = p + 1; }
        }

        self.last_f = Some(op.get_motion().clone()); 
    }

    fn execute_Find(&mut self, op : &Op, line : usize) {
        let c                  = op.get_c();
        let remaining : String = self.lines[line].chars().skip(self.cursor + 1).collect();
        
        if let Some(p) = remaining.find(c)    { self.cursor += p + 1; }

        self.last_f = Some(op.get_motion().clone()); 
    }

    fn execute_BackFind(&mut self, op : &Op, line : usize) {
        let c          = op.get_c();
        let s : String = self.lines[line].chars().take(self.cursor).collect();

        if let Some(p) = s.rfind(c)    { self.cursor = p; }
        
        self.last_f = Some(op.get_motion().clone()); 
    }

    fn execute_FindNext(&mut self, _op : &Op, line : usize) {
        if let Some(mot) = self.last_f {
            let o = Op::Motion(mot);
            self.execute_Motion(&o, line);
        }
    }

    fn get_motion_fn(&self, mot : &Motion) -> Box<Fn(&mut Worker, &Op, usize)> {
        let f = match mot.mot {
            SingleMotion::Beg             => Worker::execute_Beg,
            SingleMotion::End             => Worker::execute_End,
            SingleMotion::Left            => Worker::execute_Left,
            SingleMotion::Right           => Worker::execute_Right,
            SingleMotion::Word            => Worker::execute_Word,
            SingleMotion::Back            => Worker::execute_Back,
            SingleMotion::Till     { .. } => Worker::execute_Till,
            SingleMotion::BackTill { .. } => Worker::execute_BackTill,
            SingleMotion::Find     { .. } => Worker::execute_Find,
            SingleMotion::BackFind { .. } => Worker::execute_BackFind,
            SingleMotion::FindNext        => Worker::execute_FindNext,
        };
        Box::new(move|worker, op, l| {
            let mut bad = false;
            let save = worker.cursor;
            for _ in 0..op.get_repeat() {
                let c = worker.cursor;
                f(worker, op, l);
                if c == worker.cursor { bad = true; } 
            }
            if bad { worker.cursor = save; }
        })
    }

    fn execute_Motion(&mut self, op : &Op, line : usize) {
        let mot = match op {
            Op::Motion(mot) => mot,
            _               => panic!("internal error: execute_Motion() on non-motion")
        };
        self.get_motion_fn(mot)(self, op, line);
    }

    fn op_motion_is_inclusive(&self, op : &Op) -> bool {
        match op {
            Op::Motion(m) => {
                match m.mot {
                      SingleMotion::Till { .. }
                    | SingleMotion::Find { .. }         => true,
                    SingleMotion::FindNext => match self.last_f {
                        Some(m) => match m.mot {
                              SingleMotion::Till { .. }
                            | SingleMotion::Find { .. } => true,
                            _                           => false
                        },
                        _                               => panic!("internal error: op_motion_is_inclusive() -- missing self.last_f")
                    },
                    _                                   => false
                }
            },
            _ => panic!("internal error: op_motion_is_inclusive() on non-motion")
        }
    }

    fn execute_Insert(&mut self, op : &Op, line : usize) {
        let s = op.get_s();

        if s.len() > 0 {
            let before : String = self.lines[line].chars().take(self.cursor).collect();
            let after  : String = self.lines[line].chars().skip(self.cursor).collect();

            self.lines[line] = before + &s + &after;

            self.cursor += s.len() - 1;
        }
    }

    fn execute_Append(&mut self, op : &Op, line : usize) {
        let s = op.get_s();

        if s.len() > 0 {
            let before : String = self.lines[line].chars().take(self.cursor + 1).collect();
            let after  : String = self.lines[line].chars().skip(self.cursor + 1).collect();

            self.lines[line] = before + &s + &after;

            self.cursor += s.len();
        }
    }

    fn execute_LineAppend(&mut self, op : &Op, line : usize) {
        let s = op.get_s();

        if s.len() > 0 {
            let before = self.lines[line].clone();

            self.lines[line] = before + &s;

            self.cursor = if self.lines[line].len() > 0 { self.lines[line].len() - 1 } else { 0 };
        }
    }

    fn execute_Delete(&mut self, op : &Op, line : usize) {
        let old_cursor = self.cursor;
        let mot        = Op::Motion(match op {
            Op::Delete{ motion } => motion.clone(),
            _                    => panic!("internal error: execute_Delete() -- missing motion")
        });
      
        let inclusive = self.op_motion_is_inclusive(&mot) as usize;

        self.execute_Motion(&mot, line);

        if self.cursor != old_cursor {
            let (beg, end)      = if old_cursor < self.cursor { (old_cursor, self.cursor) }
                                  else                        { (self.cursor, old_cursor) };
            let first  : String = self.lines[line].chars().take(beg).collect();
            let second : String = self.lines[line].chars().skip(end + inclusive).collect();
            self.lines[line]    = first + &second;

            self.cursor = beg;
        }
    }

    fn execute_DeleteToEnd(&mut self, _op : &Op, line : usize) {
        self.lines[line] = self.lines[line].chars().take(self.cursor).collect();
    }

    fn execute_Yank(&mut self, op : &Op, line : usize) {
        let old_cursor = self.cursor;
        let mot        = Op::Motion(match op {
            Op::Yank{ motion } => motion.clone(),
            _                  => panic!("internal error: execute_Yank() -- missing motion")
        });
      
        let inclusive = self.op_motion_is_inclusive(&mot) as usize;

        self.execute_Motion(&mot, line);

        if self.cursor != old_cursor {
            let (beg, end)      = if old_cursor < self.cursor { (old_cursor, self.cursor) }
                                  else                        { (self.cursor, old_cursor) };
            self.pb = self.lines[line].chars().skip(beg).take(end - beg + inclusive).collect();

            self.cursor = beg;
        }
    }

    fn execute_Put(&mut self, _op : &Op, line : usize) {
        if self.pb.len() > 0 {
            let before : String = self.lines[line].chars().take(self.cursor + 1).collect();
            let after  : String = self.lines[line].chars().skip(self.cursor + 1).collect();

            self.lines[line] = before + &self.pb + &after;
            self.cursor += self.pb.len();
        }
    }

    fn execute_Repeat(&mut self, _op : &Op, line : usize) {
        if let Some(o) = self.last_o.clone() {
            let execute_fn = self.get_execute_fn(&o);
            execute_fn(self, &o, line);
        }
    }

    fn get_execute_fn(&self, op : &Op) -> fn(&mut Worker, &Op, usize) {
        match op {
            Op::Motion(_)         => Worker::execute_Motion,
            Op::Insert     { .. } => Worker::execute_Insert,
            Op::Append     { .. } => Worker::execute_Append,
            Op::LineAppend { .. } => Worker::execute_LineAppend,
            Op::Delete     { .. } => Worker::execute_Delete,
            Op::DeleteToEnd       => Worker::execute_DeleteToEnd,
            Op::Yank       { .. } => Worker::execute_Yank,
            Op::Put               => Worker::execute_Put,
            Op::Repeat            => Worker::execute_Repeat,
        }
    }

    fn execute(&mut self, ops : &Vec<Op>) {
        for i in 0..self.lines.len() {
            self.cursor = 0;
            self.pb = String::new();
            for op in ops {
                let execute_fn = self.get_execute_fn(&op);
                execute_fn(self, op, i);
                self.last_o = match op {
                    Op::Motion(_) => None,
                    Op::Repeat    => self.last_o.clone(),
                    _             => Some(op.clone())
                };
            }
        }
    }
}

struct Manager {
    count     : usize,
    workers   : Vec<Worker>,
}

impl Manager {
    fn new() -> Manager {
        Manager {
            count     : 0,
            workers   : Vec::new(),
        }
    }

    fn add_worker(&mut self, lines : Vec<String>) {
        let worker = Worker {
            pos    : self.count,
            cursor : 0,
            pb     : String::new(),
            last_f : None,
            last_o : None,
            lines  : lines
        };
        self.workers.push(worker);
        self.count += 1;
    }

    fn start(&mut self, ops : Vec<Op>) {
        let outputer  = Arc::new(Mutex::new(Outputer::new()));
        let _outputer = outputer.clone(); /* one ref count */
        self.workers.par_iter_mut().for_each(|worker| {
            worker.execute(&ops);
            loop {
                let mut guarded_outputer = outputer.lock().unwrap();
                if guarded_outputer.allow == worker.pos {
                    for line in &mut worker.lines {
                        line.push('\n');
                        guarded_outputer.buf_wr.write(line.as_bytes()).unwrap();
                    }

                    guarded_outputer.allow += 1;
                    break;
                }
            }
        });
    }
}

pub fn execute(ops : Vec<Op>) {
    let mut line : String;

    let mut lines   = Vec::new();
    let page_sz     = page_size::get();
    let mut size    = 0;
    let mut manager = Manager::new();

    for _line in io::stdin().lock().lines() {
        line = _line.unwrap();
        if line.len() >= page_sz {
            lines.push(line);
            manager.add_worker(lines);
            lines = Vec::new();
            size  = 0;
        } else if line.len() + size > page_sz {
            manager.add_worker(lines);
            lines = Vec::new();
            size  = line.len();
            lines.push(line);
        } else {
            size += line.len();
            lines.push(line);
        }
    }
    if lines.len() > 0 {
        manager.add_worker(lines);
    }

    manager.start(ops);
}
