use std::fs::File;
use std::process::exit;
use std::thread::Builder;

use cc1::args::parse_args;
use cc1::codegen::{generate, generate_to};
use cc1::context::Context;
use cc1::parser::parse_source;
use cc1::report::dump_diagnostics;
use cc1::semantic::{Analyzer, has_errors};

const STACK_SIZE: usize = 512 << 20;

fn main() {
    let compiler = Builder::new().stack_size(STACK_SIZE).spawn(compile).expect("failed to spawn the compiler thread");
    exit(compiler.join().unwrap_or(1));
}

fn compile() -> i32 {
    let (mut ctx, outfile) = parse_args(Context::default());
    if !has_errors(&ctx.diagnostics) {
        ctx = parse_source(ctx);
    }
    let parsed = !has_errors(&ctx.diagnostics);

    let mut sema = Analyzer::begin(ctx);
    if parsed {
        sema = Analyzer::run_passes(sema);
    }
    let analyzed = parsed && !has_errors(&sema.diagnostics);
    Analyzer::end(sema);

    let (codegen, wrote_output) = if analyzed {
        match outfile {
            Some(path) => match File::create(&path) {
                Ok(file) => (generate_to(file), true),
                Err(error) => {
                    eprintln!("cc1: {path}: {error}");
                    (Vec::new(), false)
                }
            },
            None => (generate(), true),
        }
    } else {
        (Vec::new(), true)
    };
    dump_diagnostics(&codegen);
    if analyzed && wrote_output && !has_errors(&codegen) { 0 } else { 1 }
}
