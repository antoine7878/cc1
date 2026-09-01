use std::fmt::{self, Debug};

use crate::{error::LexError, utils::byte_label};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Pipe,
    Caret,
    Dollar,
    OpenPar,
    ClosePar,
    Star,
    Plus,
    Dot,
    Question,
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
    Byte(u8),
    Quote,
    Slash,
}

impl Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Pipe => write!(f, "Pipe"),
            Token::Caret => write!(f, "Caret"),
            Token::Dollar => write!(f, "Dollar"),
            Token::OpenPar => write!(f, "OpenPar"),
            Token::ClosePar => write!(f, "ClosePar"),
            Token::Star => write!(f, "Start"),
            Token::Plus => write!(f, "Plus"),
            Token::Dot => write!(f, "Dot"),
            Token::Question => write!(f, "Question"),
            Token::OpenCurly => write!(f, "OpenCurly"),
            Token::CloseCurly => write!(f, "CloseCurly"),
            Token::OpenBracket => write!(f, "OpenBracket"),
            Token::CloseBracket => write!(f, "CloseBracket"),
            Token::Quote => write!(f, "Quote"),
            Token::Slash => write!(f, "Slash"),
            Token::Byte(c) => write!(f, "Byte({})", byte_label(*c)),
        }
    }
}

impl TryFrom<&u8> for Token {
    type Error = LexError;
    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        Token::try_from(*value)
    }
}

impl TryFrom<u8> for Token {
    type Error = LexError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'|' => Ok(Self::Pipe),
            b'^' => Ok(Self::Caret),
            b'$' => Ok(Self::Dollar),
            b'(' => Ok(Self::OpenPar),
            b')' => Ok(Self::ClosePar),
            b'*' => Ok(Self::Star),
            b'+' => Ok(Self::Plus),
            b'.' => Ok(Self::Dot),
            b'?' => Ok(Self::Question),
            b'{' => Ok(Self::OpenCurly),
            b'}' => Ok(Self::CloseCurly),
            b'[' => Ok(Self::OpenBracket),
            b']' => Ok(Self::CloseBracket),
            b'"' => Ok(Self::Quote),
            b'/' => Ok(Self::Slash),
            c => Err(LexError::Tokenizer(format!("{c} is not a single byte Token"))),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", crate::utils::byte_label(u8::from(self)))
    }
}

impl From<&Token> for u8 {
    fn from(value: &Token) -> Self {
        match value {
            Token::Pipe => b'|',
            Token::Caret => b'^',
            Token::Dollar => b'$',
            Token::OpenPar => b'(',
            Token::ClosePar => b')',
            Token::Star => b'*',
            Token::Plus => b'+',
            Token::Dot => b'.',
            Token::Question => b'?',
            Token::OpenCurly => b'{',
            Token::CloseCurly => b'}',
            Token::OpenBracket => b'[',
            Token::CloseBracket => b']',
            Token::Quote => b'"',
            Token::Slash => b'/',
            Token::Byte(c) => *c,
        }
    }
}

impl From<Token> for u8 {
    fn from(value: Token) -> Self {
        u8::from(&value)
    }
}

impl From<Token> for String {
    fn from(value: Token) -> Self {
        match value {
            Token::Pipe
            | Token::Caret
            | Token::Dollar
            | Token::OpenPar
            | Token::ClosePar
            | Token::Star
            | Token::Plus
            | Token::Dot
            | Token::Question
            | Token::OpenCurly
            | Token::CloseCurly
            | Token::OpenBracket
            | Token::CloseBracket
            | Token::Slash
            | Token::Quote => String::from(u8::from(value) as char),
            Token::Byte(c) => String::from(c as char),
        }
    }
}

impl Token {
    fn from_escaped_byte(b: u8) -> Self {
        Self::Byte(match b {
            b'a' => 7,
            b'b' => 8,
            b'f' => 12,
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'v' => 11,
            b => b,
        })
    }

    pub fn is_duplication(&self) -> bool {
        matches!(self, Self::Star | Self::Plus | Self::OpenCurly | Self::Question)
    }
}

#[derive(Debug, Clone)]
pub struct Tokenizer {
    pub bytes: Vec<u8>,
    pub i: usize,
    stash: Option<Token>,
    in_quotes: bool,
    in_brackets: bool,
}

impl Tokenizer {
    pub fn print(&self) {
        eprintln!("{:?}", self.clone().collect::<Vec<_>>());
    }
}

impl From<String> for Tokenizer {
    fn from(line: String) -> Self {
        Self::from(line.as_str())
    }
}

impl From<&str> for Tokenizer {
    fn from(line: &str) -> Self {
        Self::from(line.bytes().collect::<Vec<u8>>())
    }
}

impl From<Vec<u8>> for Tokenizer {
    fn from(line: Vec<u8>) -> Self {
        Self {
            bytes: line,
            i: 0,
            stash: None,
            in_quotes: false,
            in_brackets: false,
        }
    }
}

impl Tokenizer {
    pub fn peek(&mut self) -> Option<Token> {
        if self.stash.is_none() {
            self.stash = self.next();
        }
        self.stash
    }

    pub fn is_start_anchor(&self) -> bool {
        matches!(self.bytes.first(), Some(b'^'))
    }

    pub fn atoi(&mut self, base: u8, max_digits: u32) -> Option<Token> {
        let mut value: u8 = 0;
        let mut j = 0;
        while j < max_digits
            && let Some(c) = self.get_byte()
            && let Some(d) = (c as char).to_digit(base as u32)
        {
            value = value.saturating_mul(base);
            value = value.saturating_add(d as u8);
            j += 1;
            self.i += 1;
        }
        Some(Token::Byte(value))
    }

    fn get_byte(&mut self) -> Option<u8> {
        self.bytes.get(self.i).cloned()
    }

    pub fn next_number(&mut self) -> Result<usize, LexError> {
        let mut value: usize = 0;
        if let Some(Token::Byte(c)) = self.peek()
            && !c.is_ascii_digit()
        {
            return Err(LexError::Parsing("not a number"));
        }
        while let Some(Token::Byte(c)) = self.peek()
            && c.is_ascii_digit()
        {
            value *= 10;
            value += (c as char).to_digit(10).ok_or(LexError::Parsing("not a digit"))? as usize;
            self.next();
        }
        Ok(value)
    }

    pub fn next_name(&mut self, first: Token, second: Token) -> Option<Vec<u8>> {
        let mut out = vec![];
        loop {
            let tok = self.next()?;
            if tok == first && self.peek() == Some(second) {
                self.next();
                break;
            }
            out.push(u8::from(tok));
        }
        Some(out)
    }
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tok) = self.stash {
            self.stash = None;
            return Some(tok);
        }

        let c = self.get_byte()?;

        self.i += 1;
        match c {
            b'\\' => {
                let c = self.get_byte()?;
                match c {
                    b'x' => {
                        self.i += 1;
                        self.atoi(16, u32::MAX)
                    }
                    b'0'..=b'9' => self.atoi(8, 3),
                    c => {
                        self.i += 1;
                        Some(Token::from_escaped_byte(c))
                    }
                }
            }
            b'"' if !self.in_brackets => {
                self.in_quotes = !self.in_quotes;
                self.next()
            }
            b'[' if !self.in_quotes => {
                self.in_brackets = true;
                Some(Token::OpenBracket)
            }
            b']' if !self.in_quotes => {
                self.in_brackets = false;
                Some(Token::CloseBracket)
            }
            b if self.in_brackets || self.in_quotes => Some(Token::Byte(b)),
            b => {
                if let Ok(parsed_token) = Token::try_from(b) {
                    Some(parsed_token)
                } else {
                    Some(Token::Byte(c))
                }
            }
        }
    }
}

impl fmt::Display for Tokenizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Bytes:")?;
        self.bytes.iter().try_for_each(|c| write!(f, "{c}"))?;
        write!(f, "\n{:1$}^", " ", self.i)
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod test {
    use super::*;
    use Token::*;
    use crate::utils::assert_panics;

    fn str_to_bytes(line: &'static str) -> Vec<Token> {
        line.bytes().map(Byte).collect()
    }

    fn test_tokenizer(line: &str,  excp_tokens: Vec<Token>) {
        println!("{}", line);
        assert_eq!(Tokenizer::from(line).collect::<Vec<_>>(), excp_tokens);
    }

    #[test]
    fn basic() {
        test_tokenizer("salut", str_to_bytes("salut"));
        test_tokenizer("[.co.]", vec![OpenBracket, Byte(b'.'), Byte(b'c'), Byte(b'o'), Byte(b'.'), CloseBracket]);
        test_tokenizer("[=co=]", vec![OpenBracket, Byte(b'='), Byte(b'c'), Byte(b'o'),  Byte(b'='), CloseBracket]);
        test_tokenizer("[:co:]", vec![OpenBracket, Byte(b':'), Byte(b'c'), Byte(b'o'), Byte(b':'), CloseBracket]);
        test_tokenizer("zzz",  vec![Byte(b'z'), Byte(b'z'), Byte(b'z')]);
        test_tokenizer(r"s\ a", vec![Byte(b's'), Byte(b' '), Byte(b'a')]);
        test_tokenizer("hello_world-123", str_to_bytes("hello_world-123"));
        test_tokenizer("^a$", vec![Caret, Byte(b'a'), Dollar]);
        test_tokenizer("(ab)", vec![OpenPar, Byte(b'a'), Byte(b'b'), ClosePar]);
        test_tokenizer(r"ab\+cd", vec![Byte(b'a'), Byte(b'b'), Byte(b'+'), Byte(b'c'), Byte(b'd')]);
        test_tokenizer("a.b", vec![Byte(b'a'), Dot, Byte(b'b')]);
        test_tokenizer("a{2}b", vec![Byte(b'a'), OpenCurly, Byte(b'2'), CloseCurly, Byte(b'b')]);
        test_tokenizer("a*b+c?", vec![Byte(b'a'), Star, Byte(b'b'), Plus, Byte(b'c'), Question]);
        test_tokenizer(r"\|\^\$\(\)\*\+", vec![Byte(b'|'), Byte(b'^'), Byte(b'$'), Byte(b'('), Byte(b')'),Byte(b'*'), Byte(b'+')]);
        test_tokenizer("[a]", vec![OpenBracket, Byte(b'a'), CloseBracket]);
        test_tokenizer("[abc]", vec![OpenBracket, Byte(b'a'), Byte(b'b'), Byte(b'c'), CloseBracket]);
    }

    #[test]
    fn quotes() {
        test_tokenizer(r#""a \"""#, vec![Byte(b'a'), Byte(b' '), Byte(b'"') ]);
        test_tokenizer(r#""abc""#, vec![Byte(b'a'), Byte(b'b'), Byte(b'c')]);
    }

    #[test]
    fn slash() {
        test_tokenizer(r"a/b", vec![Byte(b'a'), Slash, Byte(b'b')]);
        test_tokenizer(r"ab/bc", vec![Byte(b'a'), Byte(b'b'), Slash, Byte(b'b'), Byte(b'c')]);
    }

    #[test]
    fn number() {
        test_tokenizer(r"\1", vec![Byte(1)]);
        test_tokenizer(r"\2", vec![Byte(2)]);
        test_tokenizer(r"\377", vec![Byte(0xff)]);
        test_tokenizer(r"\400", vec![Byte(0xff)]);
        test_tokenizer(r"\400x", vec![Byte(0xff), Byte(b'x')]);
        test_tokenizer(r"\4000", vec![Byte(0xff), Byte(b'0')]);
        test_tokenizer(r"\4001", vec![Byte(0xff), Byte(b'1')]);
        test_tokenizer(r"\4009", vec![Byte(0xff), Byte(b'9')]);

        test_tokenizer(r"\x1", vec![Byte(0x01)]);
        test_tokenizer(r"\x12", vec![Byte(0x12)]);
        test_tokenizer(r"\x1z", vec![Byte(0x01), Byte(b'z')]);
        test_tokenizer(r"\x12z", vec![Byte(0x12), Byte(b'z')]);
        test_tokenizer(r"\xabz", vec![Byte(0xab), Byte(b'z')]);
        test_tokenizer(r"\xffz", vec![Byte(0xff), Byte(b'z')]);
        test_tokenizer(r"\x0000000z", vec![Byte(0x00), Byte(b'z')]);
        test_tokenizer(r"\xfffffffz", vec![Byte(0xff), Byte(b'z')]);
    }

    #[test]
    fn tokenizer_failing() {
        assert_panics(|| test_tokenizer("salut", str_to_bytes("bonjour")));
        assert_panics(|| test_tokenizer("a.b", vec![Byte(b'a'), Plus, Byte(b'b')]));
    }
}
