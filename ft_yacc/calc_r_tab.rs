#![allow(unused_braces, mixed_script_confusables, unused)]

mod lex_yy;
mod context;

use std::io::stdin;
use context::Context;
use std::fmt;

#[allow(non_camel_case_types, mixed_script_confusables)]
            #[derive(Debug, Clone, PartialEq)]
            pub enum YYToken {
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
YYToken::Char ('\n') => 5,
YYToken::Char ('+') => 6,
YYToken::Char ('-') => 7,
YYToken::Char ('*') => 8,
YYToken::Char ('/') => 9,
YYToken::Char ('(') => 10,
YYToken::Char (')') => 11,
YYToken::prog => 12,
YYToken::expr => 13,
_ => unreachable!()
                    }
                }
            }
impl From<YYToken> for f32 {
                fn from(token: YYToken) -> f32 {
                    match token {
YYToken::rvalue(s)|YYToken::Number(s) => s,
                    _ => panic!("wrong type"),
                    }
                }
            }

pub trait YYLexer {
    fn yylex(&mut self) -> YYToken;
    fn ctx(&mut self) -> &mut Context;
}

pub struct Yacc<T: YYLexer> {
    state_stack: Vec<usize>,
    value_stack: Vec<YYToken>,
    default_prod_table: Vec<YYToken>,
    lookahead: Option<YYToken>,
    lookahead_id: usize,
    act: isize,
    pub yydebug: bool,
    continue_parse: bool,
    is_recovering: bool,
    pub lexer: T,
    ret: i32,
    token_since_error: usize,
}


impl<T: YYLexer> Yacc<T> {

const YY_ERROR_TOKEN_ID: usize = 1;
const YY_EOF_TOKEN_ID: usize = 0;
const YY_GOTO_TABLE: [[isize; 14]; 21] =[[5,-2,0,-3,-4,0,0,0,0,0,-5,0,-6,-7,],
[0,0,0,0,0,-8,0,0,0,0,0,0,0,0,],
[6,0,0,0,0,6,-9,-10,-11,-12,0,0,0,0,],
[12,0,0,0,0,12,12,12,12,12,0,12,0,0,],
[0,0,0,-13,-4,0,0,0,0,0,-5,0,0,0,],
[1,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[2,0,0,0,0,-14,0,0,0,0,0,0,0,0,],
[5,-2,0,-3,-4,0,0,0,0,0,-5,0,-15,-7,],
[0,0,0,-16,-4,0,0,0,0,0,-5,0,0,0,],
[0,0,0,-17,-4,0,0,0,0,0,-5,0,0,0,],
[0,0,0,-18,-4,0,0,0,0,0,-5,0,0,0,],
[0,0,0,-19,-4,0,0,0,0,0,-5,0,0,0,],
[0,0,0,0,0,0,-9,-10,-11,-12,0,-20,0,0,],
[5,-2,0,-3,-4,0,0,0,0,0,-5,0,-21,-7,],
[4,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[7,0,0,0,0,7,7,7,-11,-12,0,7,0,0,],
[8,0,0,0,0,8,8,8,-11,-12,0,8,0,0,],
[9,0,0,0,0,9,9,9,9,9,0,9,0,0,],
[10,0,0,0,0,10,10,10,10,10,0,10,0,0,],
[11,0,0,0,0,11,11,11,11,11,0,11,0,0,],
[3,0,0,0,0,0,0,0,0,0,0,0,0,0,],
];
const YY_RLEN_TABLE: [usize; 12] = [1,1,3,3,0,1,3,3,3,3,3,1,];
const YY_PRODUCT_TABLE: [usize; 12] = [2,12,12,12,12,13,3,3,3,3,3,3,];
const YY_ACTION_TABLE: [isize; 12] = [-1,-1,-1,-1,-1,0,1,2,3,4,5,6,];
const YY_DEFAULT_ACT: [isize; 21] = [0,0,0,12,0,0,0,0,0,0,0,0,0,0,4,0,0,9,10,11,3,];
const YY_DEFAULT_REDUCE_ACT: [isize; 21] = [5,0,6,12,0,0,2,5,0,0,0,0,0,5,4,7,8,9,10,11,3,];
const YY_PRODUCTION_LINE: [usize; 12] = [0,21,22,23,24,28,32,33,34,42,43,44,];
const YY_TERMINAL_TABLE: [bool; 14] = [false,false,false,true,false,false,false,false,false,false,false,false,true,true,];

    pub fn new(lexer: T) -> Yacc<T> {
        Yacc {
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
default_prod_table: vec![YYToken::accept,YYToken::prog,YYToken::prog,YYToken::prog,YYToken::prog,YYToken::expr,YYToken::rvalue(f32::default()),YYToken::rvalue(f32::default()),YYToken::rvalue(f32::default()),YYToken::rvalue(f32::default()),YYToken::rvalue(f32::default()),YYToken::rvalue(f32::default()),],
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
        yyerror("syntax error");
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

        self.push_statcks(-self.act as usize, self.lookahead.clone().unwrap_or(YYToken::yyeof));

        self.lookahead = None;
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
        let values = self.value_stack.drain(idx..).collect::<Vec<YYToken>>().clone();
        let _a = self.state_stack.drain(idx..).collect::<Vec<_>>().clone();


        let ret = match Self::YY_ACTION_TABLE[self.act as usize] {
0 => {
{ println!("= {}", f32::from(values[0].clone())); };
YYToken::expr}
1 => YYToken::rvalue({ self.lexer.ctx().increment_op(); f32::from(values[0].clone())+ f32::from(values[2].clone())}),
2 => YYToken::rvalue({ self.lexer.ctx().increment_op(); f32::from(values[0].clone())- f32::from(values[2].clone())}),
3 => YYToken::rvalue({ self.lexer.ctx().increment_op(); f32::from(values[0].clone())* f32::from(values[2].clone())}),
4 => YYToken::rvalue({
        self.lexer.ctx().increment_op();
        if f32::from(values[2].clone())== 0.0 {
            return YYToken::error;
        }
        f32::from(values[0].clone())/ f32::from(values[2].clone())}),
5 => YYToken::rvalue({ f32::from(values[1].clone())}),
6 => YYToken::rvalue({ self.lexer.ctx().increment_num(); f32::from(values[0].clone())}),
            -1 => self.default_prod_table[self.act as usize].clone(),
            _ => unreachable!(),
        };

        ret
    }
}

pub fn yyerror<D: fmt::Display>(msg: D) {
    eprintln!("{}", msg);
}



fn main() {
    use std::env;
    use std::io::{Cursor, Read};
    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());
    let mut lexer = lex_yy::YYLex::new(reader, || None, Context::new());

    let mut yacc = Yacc::new(lexer);
    // yacc.yydebug = true;
    yacc.yyparse();
    println!("ctx: {:?}", yacc.lexer.ctx);
}
