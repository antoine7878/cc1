use std::fmt;
use std::str::Chars;

#[derive(Debug)]
pub enum ArgError {
    MissingValue(char),
    WrongType(char, String),
    UnknownOption(char),
    BadArgumentCount(usize, usize),
    NotAfile(String),
    Process(String),
}

impl std::error::Error for ArgError {}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgError::MissingValue(opt) => write!(f, "Option -{} is missing a value", opt),
            ArgError::WrongType(opt, val) => write!(f, "{} is not a value of option -{} ", val, opt),
            ArgError::UnknownOption(opt) => write!(f, "Unknown option -{}", opt),
            ArgError::BadArgumentCount(a, b) => write!(f, "Bad argument count, got {}, expected {}", a, b),
            ArgError::NotAfile(name) => write!(f, "{} is not a file", name),
            ArgError::Process(name) => write!(f, "{}", name),
        }
    }
}

pub trait ArgParser: Sized {
    fn positional(&mut self, arg: String);
    fn flag(&mut self, c: char, it: &mut Chars) -> Result<(), ArgError>;
    fn argv(&mut self) -> &mut std::env::Args;

    fn walk(&mut self) -> Result<(), ArgError> {
        let mut only_unamed = false;
        while let Some(arg) = self.argv().next() {
            match arg.starts_with('-') {
                _ if only_unamed => self.positional(arg),
                false => self.positional(arg),
                true if arg == "--" => only_unamed = true,
                true => {
                    let mut it = arg.chars();
                    it.next();
                    while let Some(c) = it.next() {
                        self.flag(c, &mut it)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn value<T: TryFrom<String>>(&mut self, it: &mut Chars, opt: char) -> Result<T, ArgError> {
        let mut str_value = it.collect::<String>();
        if str_value.is_empty() {
            str_value = self.argv().next().ok_or(ArgError::MissingValue(opt))?
        }
        let value = T::try_from(str_value.clone());
        value.or(Err(ArgError::WrongType(opt, str_value)))
    }
}
