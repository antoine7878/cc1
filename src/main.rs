use cc1::ast::print::AstPrinter;
use cc1::parser::{Context, YYLex, Yacc};

use std::io::{stdin, stdout};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lexer = YYLex::new(stdin(), || None, Context::default());
    let yacc = Yacc::new(lexer);
    let ctx = yacc.yyparse();
    AstPrinter::print_ast(stdout(), &ctx)?;
    Ok(())
}
