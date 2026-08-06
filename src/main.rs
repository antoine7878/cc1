pub mod arena;
pub mod ast;
pub mod color;
pub mod context;
pub mod error;
pub mod parser;

use context::Context;
use parser::{YYLex, Yacc};

use std::io::{stdin, stdout};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lexer = YYLex::new(stdin(), || None, Context::default());
    let yacc = Yacc::new(lexer);
    let ctx = yacc.yyparse();
    ctx.print_ast(&mut stdout())?;
    Ok(())
}
