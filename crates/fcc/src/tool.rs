use std::fmt;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Ccp,
    Clang,
    Cc1,
    Llc,
    As,
}

impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Tool::Ccp => "ccp",
            Tool::Clang => "clang",
            Tool::Cc1 => "cc1",
            Tool::Llc => "llc",
            Tool::As => "as",
        }
    }

    pub fn path(self) -> &'static Path {
        let s = match self {
            Tool::Ccp => "./target/debug/ccp",
            Tool::Cc1 => "./target/debug/cc1",
            Tool::Llc => "llc",
            Tool::As => "as",
            Tool::Clang => "clang",
        };
        Path::new(s)
    }
}

impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Tool::Ccp => "ccp",
            Tool::Cc1 => "cc1",
            Tool::Llc => "llc",
            Tool::As => "As",
            Tool::Clang => "clang",
        };
        write!(f, "{s}")
    }
}
