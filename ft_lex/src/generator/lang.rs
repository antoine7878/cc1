use std::fmt;

use crate::error::LexError;
use crate::generator::{CGenerator, Generator, RSGenerator};

#[derive(Debug, Default, Clone, Copy)]
pub enum Lang {
    #[default]
    C,
    Rust,
}

impl Lang {
    pub fn defalut_out_file(&self) -> &'static str {
        match self {
            Self::C => "lex.yy.c",
            Self::Rust => "lex_yy.rs",
        }
    }

    pub fn template_file(&self) -> &'static str {
        match self {
            Self::C => include_str!("../../template/template.c"),
            Self::Rust => include_str!("../../template/template.rs"),
        }
    }

    pub fn generator(&self) -> Box<dyn Generator> {
        match self {
            Self::C => Box::new(CGenerator::new()),
            Self::Rust => Box::new(RSGenerator::new()),
        }
    }
}

#[allow(unused)]
impl Lang {
    pub fn compiler(&self) -> &'static str {
        match self {
            Self::C => "clang",
            Self::Rust => "rustc",
        }
    }

    pub fn lex_flag(&self) -> &'static str {
        match self {
            Self::C => "c",
            Self::Rust => "rust",
        }
    }

    pub fn src_extension(&self) -> &'static str {
        match self {
            Self::C => ".c",
            Self::Rust => ".rs",
        }
    }
}

impl TryFrom<&str> for Lang {
    type Error = LexError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "c" => Ok(Self::C),
            "rust" => Ok(Self::Rust),
            _ => Err(LexError::Parsing("wrong lang")),
        }
    }
}

impl TryFrom<String> for Lang {
    type Error = LexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::C => write!(f, "c"),
            Self::Rust => write!(f, "rust"),
        }
    }
}
