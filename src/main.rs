mod c_tab;
mod lex_yy;

pub use c_tab::{YYToken, Yacc};
pub use lex_yy::YYLex;

fn main() {
    let lexer = YYLex::default();
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
}
