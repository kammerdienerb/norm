/*
 * cmd.rs
 * wrapper for command string
 */

extern crate colored;
use colored::*;

pub struct Cmd {
    pub orig     : String,
    pub working  : String,
    pub consumed : usize
}

impl Cmd {
    pub fn new(cmd : &String) -> Cmd {
        Cmd {
            orig     : cmd.clone(),
            working  : cmd.clone(),
            consumed : 0
        }
    }

    pub fn len(&self) -> usize { self.working.len() }

    pub fn first(&self) -> char {
        if let Some(c) = self.working.chars().nth(0) {
            c
        } else {
            let msg = String::from("unexpected end of command input");
            self.err(&msg);
            unreachable!()
        }
    }

    pub fn consume(&mut self) -> char {
        let c = self.first();
        self.working.remove(0);
        self.consumed += 1;
        c
    }

    pub fn err(&self, msg : &String) {
        eprintln!("norm: {}", msg.red());
        let used : String = self.orig.chars().take(self.consumed).collect();
        eprintln!("      Here: '{}{}'", used, self.working);
        eprintln!("             {}{}{}",
                  std::iter::repeat("~").take(self.consumed).collect::<String>().green(),
                  "^".green(),
                  std::iter::repeat("~").take(
                      if self.consumed < self.orig.len() {
                          self.orig.len() - self.consumed - 1
                      } else { 0 }).collect::<String>().green());
        std::process::exit(1);
    }
}
