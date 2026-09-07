use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub struct Position {
    pub line: usize,
    pub col: usize,
    pub file: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Span {
        Span { start, end }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.start.line == self.end.line && self.start.col == self.end.col {
            false => write!(
                f,
                "<{}:{}, {}:{}>",
                self.start.line, self.start.col, self.end.line, self.end.col
            ),
            true => write!(f, "<{}:{}>", self.start.line, self.start.col),
        }
    }
}
