pub mod arena;
pub mod ast;
pub mod color;
pub mod context;
pub mod error;
pub mod parser;

use context::Context;
use parser::{YYLex, Yacc};

use std::io::stdin;
fn main() {
    let lexer = YYLex::new(stdin(), || None, Context::default());
    let mut yacc = Yacc::new(lexer);
    yacc.yydebug = true;
    yacc.yyparse();
}
