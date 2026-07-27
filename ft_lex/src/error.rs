use std::fmt;

#[derive(Debug)]
pub enum LexError {
    InputFile(String),
    AstParsing(String),
    Tokenizer(String),
    Parsing(&'static str),
    Io(std::io::Error),
}

impl std::error::Error for LexError {}

impl From<std::io::Error> for LexError {
    fn from(e: std::io::Error) -> Self {
        LexError::Io(e)
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexError::Io(e) => write!(f, "IO error: {e}"),
            LexError::InputFile(s) => write!(f, "during lexfile parsing: {s}"),
            LexError::AstParsing(s) => write!(f, "During ast parsing: {s}"),
            LexError::Parsing(s) => write!(f, "During parsing: {s}"),
            LexError::Tokenizer(s) => write!(f, "During regex tokenization: {s}"),
        }
    }
}
