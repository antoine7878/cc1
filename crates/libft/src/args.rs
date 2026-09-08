use std::env;
use std::fmt;
use std::str::Chars;
use std::vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgError {
    MissingValue(char),
    WrongValue(char, String),
    UnknownOption(char),
    UnknownLongOption(String),
    BadArgumentCount(usize, usize),
    NotAfile(String),
    Process(String),
}

impl std::error::Error for ArgError {}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgError::MissingValue(opt) => write!(f, "Option -{} is missing a value", opt),
            ArgError::WrongValue(opt, val) => write!(f, "{} is not a value of option -{} ", val, opt),
            ArgError::UnknownOption(opt) => write!(f, "Unknown option -{}", opt),
            ArgError::UnknownLongOption(opt) => write!(f, "Unknown option {}", opt),
            ArgError::BadArgumentCount(a, b) => write!(f, "Bad argument count, got {}, expected {}", a, b),
            ArgError::NotAfile(name) => write!(f, "{} is not a file", name),
            ArgError::Process(name) => write!(f, "{}", name),
        }
    }
}

pub fn argv() -> vec::IntoIter<String> {
    env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .into_iter()
}

pub trait ArgParser: Sized {
    type Argv: Iterator<Item = String>;

    fn positional(&mut self, arg: String);
    fn flag(&mut self, c: char, it: &mut Chars) -> Result<(), ArgError>;
    fn argv(&mut self) -> &mut Self::Argv;

    fn walk(&mut self) -> Result<(), ArgError> {
        let mut only_unnamed = false;
        while let Some(arg) = self.argv().next() {
            if only_unnamed || !arg.starts_with('-') || arg == "-" {
                self.positional(arg);
            } else if arg == "--" {
                only_unnamed = true;
            } else if arg.starts_with("--") {
                return Err(ArgError::UnknownLongOption(arg));
            } else {
                let mut it = arg.chars();
                it.next();
                while let Some(c) = it.next() {
                    self.flag(c, &mut it)?;
                }
            }
        }
        Ok(())
    }

    fn value<T: TryFrom<String>>(&mut self, it: &mut Chars, opt: char) -> Result<T, ArgError> {
        let mut str_value = it.collect::<String>();
        if str_value.is_empty() {
            str_value = match self.argv().next() {
                Some(next) if !next.is_empty() && next != "--" => next,
                _ => return Err(ArgError::MissingValue(opt)),
            }
        }
        T::try_from(str_value.clone()).map_err(|_| ArgError::WrongValue(opt, str_value))
    }
}
