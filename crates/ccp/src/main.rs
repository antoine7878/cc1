use std::process::exit;

use ccp::args::Args;
use ccp::driver::run;

fn main() {
    let args = match Args::parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("ccp: error: {e}");
            exit(1)
        }
    };
    if let Err(e) = run(&args) {
        eprintln!("ccp: error: {e}");
        exit(1)
    }
}
