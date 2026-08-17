use cc1::ast::print::AstPrinter;
use cc1::parser::{Context, YYLex, Yacc};
use cc1::semantic::Analyzer;

use std::env::args;
use std::fs::File;
use std::process::exit;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let file: File = File::open(&file_name)?;
    let ctx = Context::new(file_name);
    let lexer = YYLex::new(file, || None, ctx);
    let yacc = Yacc::new(lexer);
    let ctx = yacc.yyparse();
    AstPrinter::print(&ctx)?;
    let _ctx = Analyzer::analyze(ctx);
    Ok(())
}
