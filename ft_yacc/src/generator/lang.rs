use std::fmt;

use crate::generator::{CGenerator, Generator, RSGenerator};
use crate::utils::YaccError;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    #[default]
    C,
    Rust,
}

impl Lang {
    pub fn header_file(&self) -> &'static str {
        match self {
            Self::C => include_str!("../../template/template.h"),
            Self::Rust => unimplemented!(),
        }
    }

    pub fn generator(&self) -> Box<dyn Generator> {
        match self {
            Self::C => Box::new(CGenerator::new()),
            Self::Rust => Box::new(RSGenerator::new()),
        }
    }
}

#[allow(dead_code)]
impl Lang {
    pub fn from_file(file: &str) -> Self {
        if file.ends_with("_r.l") { Lang::Rust } else { Lang::C }
    }

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

    pub fn tab_extention(&self) -> &'static str {
        match self {
            Lang::Rust => "_tab.rs",
            Lang::C => ".tab.c",
        }
    }
}

impl TryFrom<&str> for Lang {
    type Error = YaccError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "c" => Ok(Self::C),
            "rust" => Ok(Self::Rust),
            _ => Err(YaccError::Error),
        }
    }
}

impl TryFrom<String> for Lang {
    type Error = YaccError;

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
