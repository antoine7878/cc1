use std::process::exit;

use cc1::args::parse_args;
use cc1::codegen::generate;
use cc1::context::Context;
use cc1::parser::parse_source;
use cc1::report::dump_diagnostics;
use cc1::semantic::{Analyzer, has_errors};

fn main() {
    let mut ctx = parse_args(Context::default());
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

    let codegen = if analyzed { generate() } else { Vec::new() };
    dump_diagnostics(&codegen);
    exit(if analyzed && !has_errors(&codegen) { 0 } else { 1 });
}
