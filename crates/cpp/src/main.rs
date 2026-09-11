use std::process::exit;

use cpp::args::Args;
use cpp::driver::run;

fn main() {
    let args = match Args::parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("cpp: error: {e}");
            exit(1)
        }
    };
    if let Err(e) = run(&args) {
        eprintln!("cpp: error: {e}");
        exit(1)
    }
}
