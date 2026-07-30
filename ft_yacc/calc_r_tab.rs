#![allow(unused_braces, mixed_script_confusables, unused)]
use std::fmt;
use std::io::Read;

#[allow(non_camel_case_types, mixed_script_confusables)]
#[derive(Debug, Clone, PartialEq)]
pub enum YYToken {
    Empty,
    yyeof,
    error,
    accept,
    rvalue(f32),
    Number(f32),
    prog,
    expr,
    Char(char),
}
impl YYToken {
    fn index(&self) -> usize {
        match self {
            YYToken::yyeof => 0,
            YYToken::error => 1,
            YYToken::accept => 2,
            YYToken::rvalue(_) => 3,
            YYToken::Number(_) => 4,
            YYToken::Char('\n') => 5,
            YYToken::Char('+') => 6,
            YYToken::Char('-') => 7,
            YYToken::Char('*') => 8,
            YYToken::Char('/') => 9,
            YYToken::Char('(') => 10,
            YYToken::prog => 11,
            YYToken::expr => 12,
            YYToken::Char(')') => 13,
            _ => unreachable!(),
        }
    }
}
impl From<YYToken> for f32 {
    fn from(token: YYToken) -> f32 {
        match token {
            YYToken::rvalue(s) | YYToken::Number(s) => s,
            _ => panic!("wrong type"),
        }
    }
}

pub struct Yacc<R: Read> {
    state_stack: Vec<usize>,
    value_stack: Vec<YYToken>,
    lookahead: Option<YYToken>,
    lookahead_id: usize,
    act: isize,
    pub yydebug: bool,
    continue_parse: bool,
    is_recovering: bool,
    pub lexer: YYLex<R>,
    ret: i32,
    token_since_error: usize,
}

impl<R: Read> Yacc<R> {
    const YY_ERROR_TOKEN_ID: usize = 1;
    const YY_EOF_TOKEN_ID: usize = 0;
    const YY_GOTO_TABLE: [[isize; 14]; 21] = [
        [0, -2, 0, -3, -4, 0, 0, 0, 0, 0, -5, -6, -7, 0],
        [0, 0, 0, 0, 0, -8, 0, 0, 0, 0, 0, 0, 0, 0],
        [5, 0, 0, 0, 0, 5, -9, -10, -11, -12, 0, 0, 0, 0],
        [11, 0, 0, 0, 0, 11, 11, 11, 11, 11, 0, 0, 0, 11],
        [0, 0, 0, -13, -4, 0, 0, 0, 0, 0, -5, 0, 0, 0],
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [2, 0, 0, 0, 0, -14, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, -2, 0, -3, -4, 0, 0, 0, 0, 0, -5, -15, -7, 0],
        [0, 0, 0, -16, -4, 0, 0, 0, 0, 0, -5, 0, 0, 0],
        [0, 0, 0, -17, -4, 0, 0, 0, 0, 0, -5, 0, 0, 0],
        [0, 0, 0, -18, -4, 0, 0, 0, 0, 0, -5, 0, 0, 0],
        [0, 0, 0, -19, -4, 0, 0, 0, 0, 0, -5, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, -9, -10, -11, -12, 0, 0, 0, -20],
        [0, -2, 0, -3, -4, 0, 0, 0, 0, 0, -5, -21, -7, 0],
        [4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [6, 0, 0, 0, 0, 6, 6, 6, -11, -12, 0, 0, 0, 6],
        [7, 0, 0, 0, 0, 7, 7, 7, -11, -12, 0, 0, 0, 7],
        [8, 0, 0, 0, 0, 8, 8, 8, 8, 8, 0, 0, 0, 8],
        [9, 0, 0, 0, 0, 9, 9, 9, 9, 9, 0, 0, 0, 9],
        [10, 0, 0, 0, 0, 10, 10, 10, 10, 10, 0, 0, 0, 10],
        [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    const YY_RLEN_TABLE: [usize; 11] = [1, 1, 3, 3, 1, 3, 3, 3, 3, 3, 1];
    const YY_PRODUCT_TABLE: [usize; 11] = [2, 11, 11, 11, 12, 3, 3, 3, 3, 3, 3];
    const YY_ACTION_TABLE: [isize; 11] = [-1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6];
    const YY_DEFAULT_ACT: [isize; 21] = [0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 8, 9, 10, 3];
    const YY_DEFAULT_REDUCE_ACT: [isize; 21] = [0, 0, 5, 11, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 4, 6, 7, 8, 9, 10, 3];
    const YY_PRODUCTION_LINE: [usize; 11] = [0, 14, 15, 16, 20, 24, 25, 26, 27, 28, 29];
    const YY_TERMINAL_TABLE: [bool; 14] = [
        false, false, false, true, false, false, false, false, false, false, false, true, true, false,
    ];

    pub fn new(lexer: YYLex<R>) -> Self {
        Self {
            state_stack: Vec::new(),
            value_stack: Vec::new(),
            lookahead: None,
            lookahead_id: 0,
            yydebug: false,
            act: 0,
            continue_parse: true,
            is_recovering: false,
            ret: 0,
            lexer: lexer,
            token_since_error: 0,
        }
    }

    fn next_act(&mut self) -> isize {
        let &state_id = self.state_stack.last().unwrap();
        let mut dflt = Self::YY_DEFAULT_ACT[state_id];
        if dflt != 0 {
            return dflt;
        }
        if self.lookahead.is_none() {
            self.read_token();
        }
        dflt = Self::YY_GOTO_TABLE[state_id][self.lookahead_id];
        if dflt != 0 {
            return dflt;
        }
        Self::YY_DEFAULT_REDUCE_ACT[state_id]
    }

    pub fn yyparse(&mut self) -> i32 {
        self.push_statcks(0, YYToken::yyeof);
        self.read_token();

        while self.continue_parse {
            let state_id = self.state_stack.last().unwrap();
            self.act = self.next_act();
            match self.act {
                1 => break,
                a if a > 0 => self.reduce(),
                a if a < 0 => self.shift(),
                _ => self.error(),
            }
        }
        self.ret
    }

    fn error(&mut self) {
        if !self.is_recovering {
            self.unwind()
        }
        if matches!(self.lookahead, Some(YYToken::yyeof)) {
            self.yyabort();
        }
    }

    fn unwind(&mut self) {
        yyerror("syntax error", self);
        while let Some(i) = self.state_stack.last()
            && Self::YY_GOTO_TABLE[*i][Self::YY_ERROR_TOKEN_ID] == 0
        {
            self.state_stack.pop();
            let val = self.value_stack.pop().unwrap();
        }

        let Some(i) = self.state_stack.last() else {
            self.yyabort();
            return;
        };
        self.is_recovering = true;
        let act = Self::YY_GOTO_TABLE[*i][Self::YY_ERROR_TOKEN_ID] + 1;
        self.push_statcks((-act) as usize, YYToken::error);
        self.token_since_error = 0;

        let top = (-act) as usize;
        self.act = Self::YY_GOTO_TABLE[top][self.lookahead_id];
        while self.act == 0 {
            self.yyclearin();
            self.read_token();
            if matches!(self.lookahead, Some(YYToken::yyeof)) {
                return;
            }
            self.act = Self::YY_GOTO_TABLE[top][self.lookahead_id];
        }
    }

    pub fn yyerrok(&mut self) {
        self.is_recovering = false;
    }

    pub fn yyaccept(&mut self) {
        self.continue_parse = false;
        self.ret = 0;
    }

    pub fn yyabort(&mut self) {
        self.continue_parse = false;
        self.ret = 3;
    }

    pub fn yyrecovering(&mut self) -> bool {
        self.is_recovering
    }

    pub fn yyclearin(&mut self) {
        self.lookahead = None;
    }

    /* /* DEBUGGING */ */
    /* fn token_class(token: &Option<YYToken>) -> &'static str { */
    /*     if token == &Some(YYToken::Empty) { */
    /*         return "nterm"; */
    /*     } */
    /*     let id = token.as_ref().unwrap().index(); */
    /*     if Self::YY_TERMINAL_TABLE[id] { "nterm" } else { "token" } */
    /* } */
    /* /* DEBUGGING */ */

    fn push_statcks(&mut self, state: usize, value: YYToken) {
        self.state_stack.push(state);
        self.value_stack.push(value);
    }

    fn read_token(&mut self) {
        let c = self.lexer.yylex();
        self.lookahead_id = c.index();
        self.lookahead = Some(c);
    }

    fn shift(&mut self) {
        self.act += 1;

        if self.token_since_error >= 2 {
            self.is_recovering = false;
        } else {
            self.token_since_error += 1;
        }

        if self.lookahead.is_none() {
            self.read_token();
        }

        let token = self.lookahead.take().unwrap_or(YYToken::yyeof);
        self.push_statcks(-self.act as usize, token);
    }

    fn reduce(&mut self) {
        self.act -= 1;

        let len = Self::YY_RLEN_TABLE[self.act as usize];
        let product = Self::YY_PRODUCT_TABLE[self.act as usize];

        let yyval = self.action();
        if yyval == YYToken::error {
            self.error();
            return;
        }

        let &top = self.state_stack.last().unwrap();
        self.push_statcks(-(Self::YY_GOTO_TABLE[top][product] + 1) as usize, yyval);
    }

    #[allow(unused_braces, clippy::let_and_return)]
    fn action(&mut self) -> YYToken {
        let idx = self.value_stack.len() - Self::YY_RLEN_TABLE[self.act as usize];

        let ret = self.do_action(idx);
        self.value_stack.truncate(idx);
        self.state_stack.truncate(idx);

        ret
    }

    fn do_action(&mut self, idx: usize) -> YYToken {
        match Self::YY_ACTION_TABLE[self.act as usize] {
            0 => {
                let __yy1 = f32::from(std::mem::replace(&mut self.value_stack[idx + 0], YYToken::error));
                {
                    println!("= {}", __yy1);
                };
                YYToken::expr
            }
            1 => YYToken::rvalue({
                let __yy1 = f32::from(std::mem::replace(&mut self.value_stack[idx + 0], YYToken::error));
                let __yy3 = f32::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::error));
                { __yy1 + __yy3 }
            }),
            2 => YYToken::rvalue({
                let __yy1 = f32::from(std::mem::replace(&mut self.value_stack[idx + 0], YYToken::error));
                let __yy3 = f32::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::error));
                { __yy1 - __yy3 }
            }),
            3 => YYToken::rvalue({
                let __yy1 = f32::from(std::mem::replace(&mut self.value_stack[idx + 0], YYToken::error));
                let __yy3 = f32::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::error));
                { __yy1 * __yy3 }
            }),
            4 => YYToken::rvalue({
                let __yy1 = f32::from(std::mem::replace(&mut self.value_stack[idx + 0], YYToken::error));
                let __yy3 = f32::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::error));
                { __yy1 / __yy3 }
            }),
            5 => YYToken::rvalue({
                let __yy2 = f32::from(std::mem::replace(&mut self.value_stack[idx + 1], YYToken::error));
                { __yy2 }
            }),
            6 => YYToken::rvalue({
                let __yy1 = f32::from(std::mem::replace(&mut self.value_stack[idx + 0], YYToken::error));
                { __yy1 }
            }),
            -1 => YYToken::Empty,
            _ => unreachable!(),
        }
    }
}

pub fn yyerror<D: fmt::Display, R: Read>(msg: D, yacc: &Yacc<R>) {
    eprintln!("{}", msg);
}

fn main() {
    use std::env;
    use std::io::Cursor;

    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());

    let lexer = YYLex::with_reader(reader);
    let mut yacc = Yacc::new(lexer);
    yacc.yyparse();
}
