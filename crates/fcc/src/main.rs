use std::process::exit;

mod args;
mod driver;
mod job;
mod stage;
mod tool;

use crate::args::Args;

fn main() {
    let args = match Args::parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("fcc: error: {e}");
            exit(1)
        }
    };
    if let Err(e) = driver::run(&args) {
        eprintln!("{e}");
        exit(1)
    }
}
