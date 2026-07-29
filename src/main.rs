pub mod arena;
pub mod ast;
pub mod color;
pub mod context;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod symbol;
pub mod tag;
pub mod types;

use context::Context;
use lexer::YYLex;
use parser::Yacc;

use std::io::stdin;

fn main() {
    let lexer = YYLex::new(stdin(), || None, Context::default());
    let mut yacc = Yacc::new(lexer);
    yacc.yydebug = true;
    yacc.yyparse();
}
