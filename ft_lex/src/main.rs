use std::time::{SystemTime, UNIX_EPOCH};

mod args;
mod bitset;
mod error;
mod front;
mod generator;
mod regex;
mod tester;
mod utils;

use crate::args::Args;
use crate::front::LexParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let mut args = Args::parse()?;

    if args.t {
        args.o = None;
    } else if args.o.is_none() {
        args.o = Some("lex_yy.rs".to_string());
    }
    let parser = LexParser::new(&args.i)?;
    let mut lex = parser.run(args.c)?;
    lex.run(&args)?;

    let end = SystemTime::now().duration_since(UNIX_EPOCH)?;
    if args.v && !args.n {
        eprintln!("Finished in {}ms", (end - start).as_millis());
        lex.summary(&args);
    }
    Ok(())
}
