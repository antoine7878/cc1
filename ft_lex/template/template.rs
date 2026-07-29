#![allow(unused_braces, mixed_script_confusables, unused)]
use std::io::{Read, stdin};
use std::str::from_utf8;

/* CODE_BEFORE */

/* DEFINES */
const READ_LEN: usize = 4096;

#[derive(Debug, PartialEq)]
enum LexerState {
    Error,
    Running,
    Eof,
}

/* TOKENS */

#[derive(Debug)]
struct AcceptData {
    state: usize,
    buf_pos: usize,
}

pub struct YYLex<R: Read> {
    state: LexerState,
    action: isize,
    current_state: usize,
    start_condition: usize,
    buffer: Vec<u8>,
    buffer_position: usize,
    run_position: usize,
    trailing_end_pos: usize,
    trailing_action: isize,
    bol: bool,
    more: bool,
    accept_stack: Vec<AcceptData>,
    yycontinue: Box<dyn FnMut() -> Option<R>>,
    uncounted: usize,

    pub yyin: R,
    pub yytext: String,
    pub line_no: isize,
    pub col_no: isize,
    pub ctx: Context,
}

/* CONTEXT */
pub struct Context;

#[allow(unused)]
impl Default for YYLex<std::io::Stdin> {
    fn default() -> Self {
        Self::new(stdin(), || None, Context {})
    }
}

impl<R: Read> YYLex<R> {
    pub fn with_reader(yyin: R) -> YYLex<R> {
        Self::new(yyin, || None, Context {})
    }
}
/* CONTEXT */

#[allow(unused)]
impl<R: Read> YYLex<R> {
    pub fn new<F>(yyin: R, yycontinue: F, ctx: Context) -> Self
    where
        F: FnMut() -> Option<R> + 'static,
    {
        Self {
            state: LexerState::Running,
            action: 0,
            uncounted: 0,
            current_state: 1,
            start_condition: 0,
            buffer: Vec::new(),
            buffer_position: 0,
            run_position: 0,
            trailing_end_pos: 0,
            trailing_action: -1,
            bol: true,
            more: false,
            accept_stack: Vec::new(),
            yycontinue: Box::new(yycontinue),
            yyin,
            yytext: String::new(),
            line_no: 1,
            col_no: 1,
            ctx,
        }
    }

    /* TABLES */

    /* REMOVE */
    const YY_CHAR_EQ: [isize; 0] = [];
    const YY_BASE: [isize; 0] = [];
    const YY_START: [isize; 0] = [];
    const YY_TRAILLING: [isize; 0] = [];
    const YY_ACCEPT: [isize; 0] = [];
    const YY_NEXT_ACCEPT: [isize; 0] = [];

    const YY_CLASS_COUNT: usize = 42;
    const YY_RULE_COUNT: usize = 69;

    /* REMOVE */

    fn begin(&mut self, condition: usize) {
        self.start_condition = condition;
    }

    fn reject(&mut self) {
        let next_action = Self::YY_NEXT_ACCEPT
            [self.stack_top().state * Self::YY_RULE_COUNT + self.action as usize];
        self.action = if next_action >= 0 {
            next_action
        } else {
            self.accept_stack.pop();
            Self::YY_ACCEPT[self.stack_top().state]
        };
    }

    fn keep_bytes_to_process(&mut self) -> usize {
        let bytes_to_keep = self.buffer.len() - self.buffer_position;
        let len = self.buffer.len();
        if bytes_to_keep > 0 {
            self.buffer.copy_within((len - bytes_to_keep)..len, 0);
        }
        self.shift_all_positions(self.buffer_position);
        bytes_to_keep
    }

    fn shift_all_positions(&mut self, offset: usize) {
        self.accept_stack
            .iter_mut()
            .for_each(|s| s.buf_pos -= offset);
        self.run_position -= offset;
        self.trailing_end_pos = self.trailing_end_pos.saturating_sub(offset);
        self.buffer_position -= offset;
    }

    fn load_buffer(&mut self) {
        let mut bytes_to_keep = self.keep_bytes_to_process();

        let mut read_try_len = self.buffer.len() - bytes_to_keep;
        if read_try_len == 0 {
            self.buffer.extend([0; READ_LEN]);
            read_try_len = self.buffer.len() - bytes_to_keep;
        }
        loop {
            let read_len = self.yyin.read(&mut self.buffer[bytes_to_keep..]).unwrap();
            if read_len == read_try_len {
                break;
            }
            match (self.yycontinue)() {
                Some(yyin) => {
                    self.yyin = yyin;
                    read_try_len -= read_len;
                    bytes_to_keep += read_len;
                    continue;
                }
                None => {
                    self.buffer[bytes_to_keep + read_len] = 0;
                    self.state = LexerState::Eof;
                    break;
                }
            }
        }

        if let Some(first_zero) = self.buffer.iter().position(|&b| b == 0) {
            self.buffer.truncate(first_zero);
        }
    }

    fn try_accept(&mut self) {
        if Self::YY_TRAILLING[self.current_state] >= 0 {
            self.trailing_action = Self::YY_TRAILLING[self.current_state];
            self.trailing_end_pos = self.run_position;
        }
        if Self::YY_ACCEPT[self.current_state] < 0 {
            return;
        }
        self.accept_stack.push(AcceptData {
            state: self.current_state,
            buf_pos: self.run_position,
        });
    }

    fn uncount(&mut self, c: u8) {
        self.uncounted += 1;
    }

    fn count_c(&mut self, c: u8) {
        if self.uncounted != 0 {
            self.uncounted -= 1;
            return;
        }
        if c == b'\n' {
            self.line_no += 1;
            self.col_no = 1;
        } else if c == b'\t' {
            self.col_no += 4 - (self.col_no % 4);
        } else {
            self.col_no += 1;
        }
    }

    fn count_yytext(&mut self) {
        self.yytext.clone().bytes().for_each(|c| self.count_c(c))
    }

    fn build_yytext(&mut self) {
        self.yytext = from_utf8(&self.buffer[self.buffer_position..self.run_position])
            .unwrap()
            .to_string();
    }

    pub fn yymore(&mut self) {
        self.more = true;
    }

    pub fn yyless(&mut self, n: usize) {
        self.run_position -= self.yytext.len() - n;
        self.uncounted += self.yytext.len() - n;
        self.build_yytext();
    }

    pub fn input(&mut self) -> u8 {
        if self.run_position >= self.buffer.len() {
            self.load_buffer();
        }
        if self.run_position >= self.buffer.len() {
            return 0;
        }
        let ret = *self
            .buffer
            .get(self.run_position)
            .expect("run_position out of bounds");
        self.run_position += 1;
        self.count_c(ret);
        ret
    }

    pub fn unput(&mut self, c: u8) {
        self.run_position -= 1;
        self.buffer[self.run_position] = c;
        self.yytext.pop();
    }

    fn stack_top(&self) -> &AcceptData {
        match self.accept_stack.last() {
            Some(t) => t,
            None => unreachable!(),
        }
    }

    fn stack_top_mut(&mut self) -> &mut AcceptData {
        match self.accept_stack.last_mut() {
            Some(t) => t,
            None => unreachable!(),
        }
    }

    fn prepare_run(&mut self) -> Option<()> {
        if self.state == LexerState::Error {
            return None;
        }

        if self.more {
            self.more = false;
        } else {
            self.buffer_position = self.run_position;
            self.bol = self.buffer_position == 0 || self.buffer[self.buffer_position - 1] == b'\n';
        }

        if self.run_position >= self.buffer.len() && self.state == LexerState::Eof {
            return None;
        }

        self.trailing_end_pos = 0;
        self.trailing_action = -1;

        self.current_state = Self::YY_START[self.start_condition + self.bol as usize] as usize;
        self.accept_stack = vec![AcceptData {
            state: 0,
            buf_pos: self.buffer_position,
        }];
        Some(())
    }

    fn run_dfa(&mut self) {
        loop {
            if self.run_position == self.buffer.len() {
                self.load_buffer();
            }

            if self.run_position == self.buffer.len() {
                break;
            }

            let char_class: usize =
                Self::YY_CHAR_EQ[self.buffer[self.run_position] as usize] as usize;
            self.current_state =
                Self::YY_BASE[self.current_state * Self::YY_CLASS_COUNT + char_class] as usize;
            if self.current_state == 0 {
                break;
            }
            self.try_accept();
            self.run_position += 1;
        }
    }

    fn prepare_action(&mut self) {
        let top = self.stack_top();
        self.action = Self::YY_ACCEPT[top.state];
        if self.trailing_action >= 0 && self.trailing_action == self.action {
            self.stack_top_mut().buf_pos = self.trailing_end_pos;
        }
        self.run_position = std::cmp::min(self.stack_top().buf_pos + 1, self.buffer.len());
        self.build_yytext();
    }

    pub fn yylex(&mut self) -> YYToken {
        loop {
            if self.prepare_run().is_none() {
                return YYToken::yyeof;
            }
            self.run_dfa();
            self.prepare_action();
            let ret = match self.action {
                /* ACTIONS */
                -1 => print!("{}", self.yytext),
                _ => panic!("wrong action\n"),
            };
            self.count_yytext();
            ret
        }
    }

    pub fn ctx(&mut self) -> &mut Context {
        &mut self.ctx
    }
}
/* MAIN */
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lexer = YYLex::default();
    while lexer.yylex() != YYToken::yyeof {}
    Ok(())
}
