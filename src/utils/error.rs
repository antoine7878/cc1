use std::error;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

use crate::parser::Yacc;
use crate::utils::{RED, RESET};

pub fn yyerror<D: Display, R: Read>(msg: D, yacc: &Yacc<R>) {
    const CONTEXT: usize = 5;
    const SEGMENT: &str = "--------------------------------------------------------------------------------";

    let path = &yacc.lexer.ctx.file_name;
    let span = yacc.lexer.span;
    let line_no = if span.start.line != 0 {
        span.start.line
    } else {
        yacc.lexer.pos.line
    };
    let col_no = if span.start.line != 0 {
        span.start.col
    } else {
        yacc.lexer.pos.col + 1
    };

    let start = line_no.saturating_sub(CONTEXT);
    let end = line_no.saturating_add(CONTEXT);
    let padding = end.to_string().len();

    let Ok(file) = File::open(path) else { return };
    let lines = BufReader::new(file).lines();

    eprintln!("{path}:{line_no}:{col_no}: {RED}{msg}{RESET}");

    eprintln!("{SEGMENT}");
    for (i, line) in lines.enumerate().skip(start).take(end - start + 1) {
        let Ok(line) = line else { break };

        eprintln!("{:>padding$} {line}", i + 1);
        if i == line_no - 1 {
            eprintln!("{:>padding$} {RED}{:>col_no$} {msg}{RESET} ", "", "^");
        }
    }
    eprintln!("{SEGMENT}");
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
