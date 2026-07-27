use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Read, stdin};
use std::vec::IntoIter;

use crate::models::StackPosition;
use crate::utils::{YaccError, escape_of_char};

pub struct InputItertor {
    cs: IntoIter<char>,
    peeks: [Option<char>; 2],
    pub file_name: String,
    pub line_no: usize,
    pub col_no: usize,
}

impl InputItertor {
    pub fn new(file: Option<String>) -> Result<Self, YaccError> {
        let file = file.unwrap_or("-".to_string());

        let (file, file_name): (Box<dyn Read>, String) = match file.as_str() {
            "-" => (Box::new(stdin()), "-".to_string()),
            file_name => (Box::new(File::open(file_name)?), file_name.to_string()),
        };
        let mut file = BufReader::new(file);

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let cs = buf.chars().collect::<Vec<char>>().into_iter();
        Ok(InputItertor {
            cs,
            peeks: [None, None],
            file_name,
            line_no: 1,
            col_no: 0,
        })
    }

    pub fn error_line<T: Display, R>(&mut self, msg: T, line_no: usize) -> Result<R, YaccError> {
        self.line_no = line_no;
        self.error(msg)
    }

    pub fn error<T: Display, R>(&self, msg: T) -> Result<R, YaccError> {
        YaccError::error(&format!("{}:{}", self.file_name, self.line_no), self.col_no, msg)
    }

    pub fn skip_while<P>(&mut self, pred: P)
    where
        P: Fn(char) -> bool,
    {
        while self.peek().is_some_and(&pred) {
            let _ = self.next();
        }
    }

    fn skip_whitespace(&mut self) {
        self.skip_while(|c| c.is_whitespace());
        if self.skip_comment() {
            self.skip_whitespace();
        }
    }

    pub fn take_name(&mut self) -> String {
        let s = self.take_while(|c| c.is_ascii_alphanumeric() || c == '_');
        self.skip_whitespace();
        s
    }

    pub fn take_utype(&mut self) -> Result<Option<String>, YaccError> {
        let s = if let Some('<') = self.peek() {
            self.next();
            let utype = self.take_name();
            self.expect('>')?;
            Ok(Some(utype))
        } else {
            Ok(None)
        };
        self.skip_whitespace();
        s
    }

    pub fn take_token(&mut self) -> Result<String, YaccError> {
        let s = match self.peek() {
            Some('\'') => {
                self.next();
                let c = match self.next() {
                    Some('\\') => self.next().map(escape_of_char),
                    c => c,
                };
                self.expect('\'')?;
                Ok(format!("Char{}", c.unwrap() as u8))
            }
            _ => Ok(self.take_name()),
        };
        self.skip_whitespace();
        s
    }

    pub fn take_number(&mut self) -> isize {
        let mut res: isize = 0;
        while let Some(c) = self.peek()
            && c.is_numeric()
        {
            self.next();
            res *= 10;
            res += c as isize - '0' as isize;
        }
        self.skip_whitespace();
        res
    }

    pub fn take_char(&mut self) -> Option<char> {
        self.skip_whitespace();
        let s = self.next();
        self.skip_whitespace();
        s
    }

    pub fn take_code_before(&mut self) -> Result<String, YaccError> {
        let mut buffer = String::new();
        self.take_char();
        while let Some(c) = self.next() {
            buffer.push(c);
            match c {
                '"' | '\'' => self.take_c_literal(c, &mut buffer),
                '/' if self.peek_is('*') => {
                    buffer.push(self.next().unwrap());
                    self.take_c_block_comment(&mut buffer);
                }
                '/' if self.peek_is('/') => {
                    buffer.push(self.next().unwrap());
                    let line = self.take_while(|c| c != '\n');
                    buffer.push_str(&line);
                }
                '%' if self.peek_is('}') => {
                    buffer.pop();
                    self.next();
                    break;
                }
                _ => (),
            }
        }
        self.skip_whitespace();
        Ok(buffer)
    }

    fn take_c_literal(&mut self, quote: char, buffer: &mut String) {
        while let Some(c) = self.next() {
            buffer.push(c);
            match c {
                '\\' => {
                    if let Some(esc) = self.next() {
                        buffer.push(esc);
                    }
                }
                c if c == quote => break,
                _ => (),
            }
        }
    }

    fn take_c_block_comment(&mut self, buffer: &mut String) {
        while let Some(c) = self.next() {
            buffer.push(c);
            if c == '*' && self.peek_is('/') {
                buffer.push(self.next().unwrap());
                break;
            }
        }
    }

    pub fn take_code(&mut self, stack_positions: &mut Vec<StackPosition>) -> Result<String, YaccError> {
        let mut braket_depth: u32 = 0;
        let mut buffer = String::new();
        let line = self.line_no;
        while let Some(c) = self.next() {
            match c {
                '{' => braket_depth += 1,
                '}' => braket_depth -= 1,
                '"' | '\'' => {
                    buffer.push(c);
                    self.take_c_literal(c, &mut buffer);
                    continue;
                }
                '/' if self.peek_is('*') => {
                    buffer.push(c);
                    buffer.push(self.next().unwrap());
                    self.take_c_block_comment(&mut buffer);
                    continue;
                }
                '/' if self.peek_is('/') => {
                    buffer.push(c);
                    buffer.push(self.next().unwrap());
                    let line = self.take_while(|c| c != '\n');
                    buffer.push_str(&line);
                    continue;
                }
                '$' => {
                    stack_positions.push(self.get_stack_position(buffer.len())?);
                    continue;
                }
                _ => (),
            }
            buffer.push(c);
            if braket_depth == 0 {
                break;
            }
        }
        if braket_depth != 0 {
            self.error_line("invalid action", line)?;
        }
        self.skip_whitespace();
        Ok(buffer)
    }

    fn get_stack_position(&mut self, pos: usize) -> Result<StackPosition, YaccError> {
        let utype = self.take_utype()?;
        match self.peek() {
            Some(d) if d.is_numeric() => {
                let num: isize = self.take_number();
                Ok(StackPosition::new(pos, Some(num), utype, self.line_no))
            }
            Some('-') => {
                self.take_char();
                let num = self.take_number();
                Ok(StackPosition::new(pos, Some(-num), utype, self.line_no))
            }
            Some('$') => {
                self.take_char();
                Ok(StackPosition::new(pos, None, utype, self.line_no))
            }
            Some(c) => self.error(format!("invalid stack position: '{}'", c))?,
            None => self.error("invalid stack position: 'eof'")?,
        }
    }

    #[allow(unused)]
    pub fn display<D: Display>(&mut self, msg: D) {
        eprintln!("{}: {}:{}: {:?}", msg, self.line_no, self.col_no, self.peeks);
    }

    fn count_peek(&mut self, c: Option<char>) {
        self.col_no += 1;
        if c == Some('\n') {
            self.line_no += 1;
            self.col_no = 0;
        }
    }

    pub fn next(&mut self) -> Option<char> {
        let c = self.peek();
        self.count_peek(c);
        match self.peeks {
            [Some(_), Some(_)] => {
                self.peeks[0] = self.peeks[1];
                self.peeks[1] = None;
                c
            }
            [Some(_), None] => {
                self.peeks[0] = None;
                c
            }
            [None, None] => None,
            [None, Some(_)] => unreachable!(),
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        match self.peeks {
            [Some(_), _] => (),
            [None, None] => {
                self.peeks[0] = self.cs.next();
            }
            [None, Some(_)] => unreachable!(),
        }
        self.peeks[0]
    }

    pub fn peek2(&mut self) -> [Option<char>; 2] {
        match self.peeks {
            [Some(_), Some(_)] => (),
            [Some(_), None] => {
                self.peeks[1] = self.cs.next();
            }
            [None, None] => {
                self.peeks[0] = self.cs.next();
                self.peeks[1] = self.cs.next();
            }
            [None, Some(_)] => unreachable!(),
        }
        self.peeks
    }

    fn skip_comment(&mut self) -> bool {
        if !matches!(self.peek2(), [Some('/'), Some('*')]) {
            return false;
        }
        self.peeks = [None, None];
        self.col_no += 2;
        loop {
            match self.peek2() {
                [Some('*'), Some('/')] => {
                    self.peeks = [None, None];
                    self.col_no += 2;
                    return true;
                }
                [None, _] => return true,
                _ => {
                    self.count_peek(self.peeks[0]);
                    self.peeks[0] = self.peeks[1];
                    self.peeks[1] = None;
                }
            }
        }
    }

    pub fn peek_is(&mut self, ch: char) -> bool {
        self.peek().is_some_and(|c| c == ch)
    }

    pub fn next_is(&mut self, ch: char) -> bool {
        self.next().is_some_and(|c| c == ch)
    }

    pub fn take_while<P>(&mut self, mut pred: P) -> String
    where
        P: FnMut(char) -> bool,
    {
        let mut s = String::new();
        while let Some(c) = self.peek()
            && pred(c)
        {
            s.push(c);
            self.next();
        }
        s
    }

    pub fn expect(&mut self, ch: char) -> Result<(), YaccError> {
        match self.take_char() {
            Some(c) if c == ch => Ok(()),
            Some(c) => self.error(format!("expected {:?} got {:?}", ch, c)),
            None => self.error(format!("expected {:?} got EOF", ch)),
        }
    }

    pub fn take_all(&mut self) -> String {
        self.cs.by_ref().collect::<String>().to_string()
    }
}
