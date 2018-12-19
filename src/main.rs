#![allow(non_snake_case)]

mod op;
mod cmd;
mod parse;
mod exec;

extern crate clap;
use clap::{Arg, App};

fn main() {
    let matches =
        App::new("norm")
          .version("0.1")
          .author("Brandon Kammerdiener <kammerdienerb@gmail.com>")
          .about("Vim-inspired stream editor")
          .arg(Arg::with_name("Explain")
               .short("e")
               .long("explain")
               .help("Explains the input command rather than executing it"))
          .arg(Arg::with_name("COMMAND")
               .help("The command pattern")
               .required(true))
          .get_matches();

    let s = matches.value_of("COMMAND").unwrap().to_owned();
    let mut cmd = cmd::Cmd::new(&s);

    let ops = parse::parse_command(&mut cmd);

    if matches.is_present("Explain") {
        println!("I can explain...");
        for op in ops    { op.explain(); }
    } else {
        exec::execute(ops);
    }
}
