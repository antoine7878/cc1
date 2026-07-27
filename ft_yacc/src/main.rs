mod generator;
mod models;
mod parser;
mod tester;
mod utils;

use std::fs::File;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::generator::dumper::Dumper;
use crate::generator::lang::Lang;
use crate::parser::{LALRParser, YaccParser};
use crate::utils::{Args, Graph};

fn yacc() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse()?;
    let parser = YaccParser::new(args.mandatory.first().cloned())?;

    let yacc = parser.run()?;
    args.v.then(|| yacc.print());

    let start = SystemTime::now().duration_since(UNIX_EPOCH)?;

    let parser = LALRParser::new(yacc)?;

    args.v.then(|| parser.print());
    if args.g {
        Graph::graph(&parser, args.b.as_ref().unwrap())?;
    }

    if args.v {
        let mut f = File::create(format!("{}.output", args.b.as_ref().unwrap()))?;
        parser.verbose(&mut f)?;
    }

    Dumper::dump_src(&parser, &args)?;
    if args.x == Lang::C && args.d {
        Dumper::dump_hdr(&parser, &args)?;
    }

    if !args.v {
        return Ok(());
    }
    let time = SystemTime::now().duration_since(UNIX_EPOCH)? - start;
    if time.as_micros() > 1000 {
        eprintln!("Finished in {}ms", time.as_millis());
    } else {
        eprintln!("Finished in {}us", time.as_micros());
    }
    Ok(())
}

fn main() -> ExitCode {
    match yacc() {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            eprintln!("\nError: {e}");
            ExitCode::from(1)
        }
    }
}
