use std::fmt;

use crate::utils::{YaccError, str_of_escape};

#[derive(Debug, Clone, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub enum TokenKind {
    Left,
    Right,
    Nonassoc,
    Token,
    NonTerminal,
}

impl TryFrom<&str> for TokenKind {
    type Error = YaccError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "nonassoc" => Ok(Self::Nonassoc),
            "token" => Ok(Self::Token),
            "type" => Ok(Self::NonTerminal),
            _ => Err(YaccError::Error),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenData {
    pub kind: TokenKind,
    pub name: String,
    pub utype: Option<String>,
    pub precedence: Option<usize>,
    pub value: usize,
    pub line_no: usize,
}

impl TokenData {
    pub fn new(
        name: String,
        kind: TokenKind,
        utype: Option<String>,
        precedence: Option<usize>,
        value: usize,
        line_no: usize,
    ) -> Self {
        Self {
            name,
            kind,
            utype,
            precedence,
            value,
            line_no,
        }
    }

    pub fn display_name(name: &str) -> String {
        if name.is_empty() {
            "''".to_string()
        } else if name.starts_with("Char") {
            format!("'{}'", str_of_escape(Self::get_char_value(name) as char))
        } else {
            name.to_string()
        }
    }

    pub fn self_display_name(&self) -> String {
        Self::display_name(&self.name)
    }

    pub fn get_char_value(name: &str) -> u8 {
        name[4..].parse::<u8>().unwrap()
    }

    pub fn is_nonterminal(&self) -> bool {
        self.kind == TokenKind::NonTerminal
    }
    pub fn is_char(&self) -> bool {
        self.name.starts_with("Char")
    }
}

impl fmt::Display for TokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{} {}", self.kind, self.name)?;
        if let Some(utype) = &self.utype {
            write!(f, "({})", utype)?;
        }
        Ok(())
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Left => write!(f, "left"),
            TokenKind::Right => write!(f, "right"),
            TokenKind::Nonassoc => write!(f, "nonassoc"),
            TokenKind::Token => write!(f, "token"),
            TokenKind::NonTerminal => write!(f, "type"),
        }
    }
}
