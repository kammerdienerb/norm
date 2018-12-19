/*
 * parse.rs
 * parsing code for norm commands and motions
 */

use crate::op::*;
use crate::cmd::*;

pub fn parse_command(cmd : &mut Cmd) -> Vec<Op> {
    let mut ret = Vec::new();

    while cmd.len() > 0 {
        let mut n_s = String::new();

        let c = cmd.first();
        if c.is_numeric() && c != '0' {
            while cmd.first().is_numeric() {
                n_s.push(cmd.consume());
            }
        }

        let quant = if n_s.len() > 0 { n_s.parse::<i32>().unwrap() } else { 1 };

        let o =
            if let Some(motion) = parse_motion(cmd) {
                Op::Motion(motion)
            } else if let Some(parser) = get_cmd_parser(cmd.first()) {
                parser(cmd)
            } else {
                let msg = String::from("unknown op");
                cmd.err(&msg);
                unreachable!()
            };

        for _ in 0..quant    { ret.push(o.clone()); }
    }

    ret
}

fn get_cmd_parser(c : char) -> Option<fn(&mut Cmd) -> Op> {
    match c {
        'i' => Some(parse_Insert),
        'a' => Some(parse_Append),
        'A' => Some(parse_LineAppend),
        'd' => Some(parse_Delete),
        'D' => Some(parse_DeleteToEnd),
        'y' => Some(parse_Yank),
        'p' => Some(parse_Put),
        '.' => Some(parse_Repeat),
         _  => None
    }
}

fn parse_Insert(cmd : &mut Cmd) -> Op {
    cmd.consume();
    let mut s = String::new();
    let delim = cmd.consume();
    let mut c : char;
    while { c = cmd.consume(); c != delim }   { s.push(c); }
    Op::Insert{ s : s }
}

fn parse_Append(cmd : &mut Cmd) -> Op {
    cmd.consume();
    let mut s = String::new();
    let delim = cmd.consume();
    let mut c : char;
    while { c = cmd.consume(); c != delim }   { s.push(c); }
    Op::Append{ s : s }
}

fn parse_LineAppend(cmd : &mut Cmd) -> Op {
    cmd.consume();
    let mut s = String::new();
    let delim = cmd.consume();
    let mut c : char;
    while { c = cmd.consume(); c != delim }   { s.push(c); }
    Op::LineAppend{ s : s }
}

fn parse_Delete(cmd : &mut Cmd) -> Op {
    cmd.consume();
    if let Some(m) = parse_motion(cmd) {
        Op::Delete{ motion : m }
    } else {
        let msg = String::from("expected motion after delete");
        cmd.err(&msg);
        unreachable!()
    }
}

fn parse_DeleteToEnd(cmd : &mut Cmd) -> Op { cmd.consume(); Op::DeleteToEnd }

fn parse_Yank(cmd : &mut Cmd) -> Op {
    cmd.consume();
    if let Some(m) = parse_motion(cmd) {
        Op::Yank{ motion : m }
    } else {
        let msg = String::from("expected motion after yank");
        cmd.err(&msg);
        unreachable!()
    }
}

fn parse_Put(cmd : &mut Cmd) -> Op { cmd.consume(); Op::Put }

fn parse_Repeat(cmd : &mut Cmd) -> Op { cmd.consume(); Op::Repeat }

fn parse_motion(cmd : &mut Cmd) -> Option<Motion> {
    if cmd.len() == 0    { return None; }

    let mut n_s = String::new();
    let c       = cmd.first();
    if c.is_numeric() && c != '0' {
        while cmd.first().is_numeric()    { n_s.push(cmd.consume()); }
    }
    let n = n_s.parse::<u32>().unwrap_or(1);

    fn m(cmd : &mut Cmd, s_mot : SingleMotion, n : u32) -> Option<Motion> {
        cmd.consume();
        Some(Motion{ mot : s_mot, repeat : n})
    };
    fn m_plus(s_mot : SingleMotion, n : u32) -> Option<Motion> {
        Some(Motion{ mot : s_mot, repeat : n})
    };

    match cmd.first() {
        '0' => m(cmd, SingleMotion::Beg, n),
        '$' => m(cmd, SingleMotion::End, n),
        'h' => m(cmd, SingleMotion::Left, n),
        'l' => m(cmd, SingleMotion::Right, n),
        'w' => m(cmd, SingleMotion::Word, n),
        'b' => m(cmd, SingleMotion::Back, n),
        't' => { cmd.consume(); let c = cmd.consume();
               m_plus(SingleMotion::Till     { c : c }, n) },
        'T' => { cmd.consume(); let c = cmd.consume();
               m_plus(SingleMotion::BackTill { c : c }, n) },
        'f' => { cmd.consume(); let c = cmd.consume();
               m_plus(SingleMotion::Find     { c : c }, n) },
        'F' => { cmd.consume(); let c = cmd.consume();
               m_plus(SingleMotion::BackFind { c : c }, n) },
        ';' => m(cmd, SingleMotion::FindNext, n),
         _  => None
    }
}
