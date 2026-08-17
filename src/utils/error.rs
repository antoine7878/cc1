use std::error;
use std::fmt::{self, Display};
use std::io;
use std::io::Read;

use crate::parser::Yacc;
use crate::utils::{RED, RESET};

pub fn yyerror<D: Display, R: Read>(msg: D, yacc: &Yacc<R>) {
    eprintln!(
        "{}:{}:{}: {RED}{msg}{RESET}",
        yacc.lexer.ctx.file_name, yacc.lexer.pos.line, yacc.lexer.pos.col
    );
}

#[derive(Debug)]
pub enum CCError {
    Io(io::Error),
    SyntaxError(String, usize, usize, String),
}

impl error::Error for CCError {}

impl From<io::Error> for CCError {
    fn from(e: io::Error) -> Self {
        CCError::Io(e)
    }
}

impl Display for CCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CCError::Io(e) => write!(f, "IO error: {e}"),
            CCError::SyntaxError(file, line, col, msg) => write!(f, "Error {file}:{line}:{col}: {msg}"),
        }
    }
}
impl CCError {
    pub fn error<T: Display, R>(file_name: String, line_no: usize, col_no: usize, msg: T) -> Result<R, Self> {
        Err(Self::SyntaxError(file_name, line_no, col_no, msg.to_string()))
    }
}
