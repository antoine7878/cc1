use cc1::ast::print::AstPrinter;
use cc1::parser::{Context, YYLex, Yacc};
use cc1::semantic::Analyzer;

use std::env::args;
use std::fs::File;
use std::io::BufReader;
use std::process::exit;

fn parse_args() -> String {
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
    file_name
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = parse_args();
    let file: File = File::open(&file_name)?;
    let ctx = Context::new(file_name);
    let lexer = YYLex::new(BufReader::new(file), || None, ctx);
    let mut yacc = Yacc::new(lexer);
    let status = yacc.yyparse();
    let ctx = yacc.lexer.ctx;
    AstPrinter::print(&ctx)?;
    println!("------------------------------------------------");
    let ctx = match status {
        0 => Analyzer::analyze(ctx),
        _ => ctx,
    };
    ctx.report();
    if status != 0 || !ctx.diagnosis.is_empty() {
        exit(1);
    }
    Ok(())
}
