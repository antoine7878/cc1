use std::fmt::{self, Display};

#[derive(Debug)]
pub enum YaccError {
    Error,
    GrammarError(String),
    Io(std::io::Error),
}

impl std::error::Error for YaccError {}
impl From<std::io::Error> for YaccError {
    fn from(e: std::io::Error) -> Self {
        YaccError::Io(e)
    }
}

impl fmt::Display for YaccError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YaccError::Io(e) => write!(f, "IO error: {e}"),
            YaccError::Error => write!(f, "Error"),
            YaccError::GrammarError(s) => write!(f, "{s}"),
        }
    }
}

impl YaccError {
    pub fn error<T: Display, R>(file_name: &str, line_no: usize, msg: T) -> Result<R, Self> {
        Err(Self::GrammarError(format!("{}:{} {}", file_name, line_no, msg,)))
    }
}
