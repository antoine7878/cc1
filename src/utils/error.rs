use std::error;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

use crate::parser::{Context, Span, Yacc};
use crate::semantic::{Diagnosis, DiagnosisNode, ExpectedTokens, Severity};
use crate::utils::RESET;

pub fn yyerror<D: Display, R: Read>(_msg: D, yacc: &mut Yacc<R>) {
    let span = yacc.lexer.span;
    let found = yacc.yy_lookahead_name();
    let inner = Diagnosis::SyntaxError {
        found,
        expected: ExpectedTokens::new(found, &yacc.yy_expected()),
    };
    yacc.lexer.ctx.diagnosis.push(DiagnosisNode::new(inner, span));
}

pub fn report<W: Write, D: Display>(
    w: &mut W,
    ctx: &Context,
    span: Span,
    severity: Severity,
    msg: D,
) -> io::Result<()> {
    let err_line_no = span.start.line - 1;
    let padding = err_line_no.to_string().len();
    let mid_pad = 9 - padding;

    let path = ctx.file_of(span);
    let Ok(file) = File::open(path) else { return Ok(()) };
    let Some(Ok(line)) = BufReader::new(file).lines().nth(err_line_no) else { return Ok(()) };
    let line = line.replace('\t', " ");
    let color = severity.color();

    let col_no = caret_end(span, &line);

    writeln!(
        w,
        "{path}:{err_line_no}:{}: {color}{severity}:{RESET} {msg}",
        span.start.col
    )?;
    writeln!(w, "     {:>padding$}|{:>mid_pad$}{line}", err_line_no, "")?;
    writeln!(
        w,
        "     {:>padding$}|{:>mid_pad$}{color}{:>col_no$}{RESET} ",
        "",
        "",
        "^".repeat(col_no + 1 - span.start.col)
    )
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
