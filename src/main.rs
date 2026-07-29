pub mod color;
pub mod context;
pub mod error;
pub mod lexer;
pub mod parser;

use context::Context;
use lexer::YYLex;
use parser::Yacc;

use std::io::stdin;

fn main() {
    // let args: Vec<String> = env::args().collect();
    // let reader = Cursor::new(args[1].clone());
    let lexer = YYLex::new(stdin(), || None, Context::new());
    let mut yacc = Yacc::new(lexer);
    yacc.yydebug = true;
    yacc.yyparse();
}
