use std::fmt::Display;
use std::io::Read;

use crate::color::{RED, RESET};
use crate::parser::Yacc;

// pub fn yyerror<D: Display>(msg: D) {
//     eprintln!("{RED}{msg}{RESET}");
// }

pub fn yyerror<D: Display, R: Read>(msg: D, yacc: &Yacc<R>) {
    eprintln!(
        "filename.c:{}:{}: {RED}{msg}{RESET}",
        yacc.lexer.line_no, yacc.lexer.col_no
    );
}
