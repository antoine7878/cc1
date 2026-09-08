use std::fmt::{self, Display};
use std::io::{self, Write};

use crate::color::{RED, RESET, YELLOW};
use crate::span::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

impl Severity {
    pub fn color(&self) -> &'static str {
        match self {
            Severity::Warning => YELLOW,
            Severity::Error => RED,
        }
    }
}

impl Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

pub trait SourceMap {
    fn path_of(&self, file: usize) -> Option<&str>;
    fn source_line(&self, path: &str, line_no: usize) -> Option<String>;
}

pub fn render<W: Write, S: SourceMap, D: Display>(
    w: &mut W,
    src: &S,
    prog: &str,
    span: Span,
    severity: Severity,
    msg: D,
) -> io::Result<()> {
    let line_no = span.start.line;
    let padding = line_no.to_string().len();
    let mid_pad = 9usize.saturating_sub(padding);

    let color = severity.color();

    let Some(path) = src.path_of(span.start.file) else {
        return writeln!(w, "{prog}: {color}{severity}:{RESET} {msg}");
    };
    let path = path.to_string();

    writeln!(
        w,
        "{path}:{line_no}:{}: {color}{severity}:{RESET} {msg}",
        span.start.col,
    )?;

    let Some(line) = src.source_line(&path, line_no) else { return Ok(()) };
    let line = line.replace('\t', " ");
    let col_no = caret_end(span, &line);

    writeln!(w, "     {:>padding$}|{:>mid_pad$}{line}", line_no, "")?;
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
