use cc1::ast::print::AstPrinter;
use cc1::parser::{Context, YYLex, Yacc};
use cc1::pipeline::Pipeline;
use cc1::semantic::Analyzer;

use std::env::args;
use std::fs::File;
use std::io::BufReader;
use std::process::exit;

fn parse_args(mut ctx: Context) -> Context {
    println!();
    let mut args = args();
    let Some(prog_name) = args.next() else {
        eprintln!("Error: wrong argument count");
        exit(2);
    };
    let Some(file_name) = args.next() else {
        eprintln!("Error: wrong argument count");
        eprintln!("Usage: {} file.c", prog_name);
        exit(2);
    };
    ctx.set_file_name(file_name);
    ctx
}

fn parse(ctx: Context) -> Context {
    let file: File = File::open(&ctx.file_name).unwrap();
    let lexer = YYLex::new(BufReader::new(file), || None, ctx);
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
    yacc.lexer.ctx
}

fn main() {
    let (_, stopped) = Pipeline::default()
        .then(parse_args)
        .then(parse)
        .tap(AstPrinter::print)
        .then(Analyzer::analyze)
        .tap(Context::dump_symbols)
        .tap(Context::dump_diagnostics)
        .finish();
    exit(if stopped { 1 } else { 0 });
}
