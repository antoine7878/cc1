use std::error;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

use crate::parser::{Context, Span, Yacc};
use crate::utils::{BLUE, GRAY, RESET};

pub fn yyerror<D: Display, R: Read>(msg: D, yacc: &Yacc<R>) {
    let mut buf = Vec::new();
    let _ = report(&mut buf, &yacc.lexer.ctx, yacc.lexer.span, msg);
    eprint!("{}", String::from_utf8_lossy(&buf));
}

pub fn report<W: Write, D: Display>(w: &mut W, ctx: &Context, span: Span, msg: D) -> io::Result<()> {
    const CONTEXT: usize = 3;
    const ELLIPSIS: &str = "...";

    let path = ctx.file_of(span);
    let err_line_no = span.start.line;
    let start = err_line_no.saturating_sub(CONTEXT);
    let end = err_line_no.saturating_add(CONTEXT);
    let padding = end.to_string().len();

    writeln!(w, "{path}:{err_line_no}:{}: {BLUE}{msg}{RESET}", span.start.col)?;

    let Ok(file) = File::open(path) else { return Ok(()) };
    let lines = BufReader::new(file).lines();

    writeln!(w, "{ELLIPSIS}")?;
    for (line_no, line) in lines.enumerate().skip(start).take(end - start) {
        let Ok(line) = line else { break };
        let line = line.replace('\t', " ");

        writeln!(w, "{GRAY}{:>padding$}{RESET} {line}", line_no + 1)?;

        if line_no + 1 == err_line_no {
            let col_no = caret_end(span, &line);
            writeln!(
                w,
                "{:>padding$} {BLUE}{:>col_no$} {msg}{RESET} ",
                "",
                "^".repeat(col_no + 1 - span.start.col)
            )?;
        }
    }
    writeln!(w, "{ELLIPSIS}")
}

fn caret_end(span: Span, line: &str) -> usize {
    match span.start.line == span.end.line {
        true => span.end.col.max(span.start.col),
        false => line.len().max(span.start.col),
    }
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
