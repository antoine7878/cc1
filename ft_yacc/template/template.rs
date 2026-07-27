#![allow(unused_braces, mixed_script_confusables, unused)]

/* CODE_BEFORE */
use std::fmt;

/* TOKENS */

pub struct Yacc<T: Iterator<Item = YYToken>> {
    state_stack: Vec<usize>,
    value_stack: Vec<YYToken>,
    default_prod_table: Vec<YYToken>,
    lookahead: Option<YYToken>,
    lookahead_id: usize,
    act: isize,
    pub yydebug: bool,
    continue_parse: bool,
    is_recovering: bool,
    lexer: T,
    ret: i32,
    token_since_error: usize,
}

/* DEBUGGING */
macro_rules! yylog {
    ($self:expr, $($arg:tt)*) => {
        if $self.yydebug {
            eprintln!($($arg)*);
        }
    };
}
/* DEBUGGING */

impl<T: Iterator<Item = YYToken>> Yacc<T> {
    /* REMOVE */
    const YY_EOF_TOKEN_ID: usize = 2;
    const YY_ERROR_TOKEN_ID: usize = 3;
    const YY_GOTO_TABLE: [[isize; 0]; 0] = [];
    const YY_RLEN_TABLE: [usize; 0] = [];
    const YY_DEFAULT_ACT: [usize; 0] = [];
    /* REMOVE */

    /* TABLES */
    /* DEBUGGING_TABLES */

    fn new(lexer: T) -> Yacc<T> {
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
            lexer: lexer.into_iter(),
            token_since_error: 0,
            /* DEFINES */
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
        return Self::YY_DEFAULT_REDUCE_ACT[state_id];
    }

    pub fn yyparse(&mut self) -> i32 {
        /* DEBUGGING */
        yylog!(self, "Starting parse");
        /* DEBUGGING */

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
        /* DEBUGGING */
        for token in self.value_stack[1..].iter().rev() {
            yylog!(
                self,
                "Cleanup: popping {} {:?}",
                Self::token_class(token.index()),
                token
            );
        }
        /* DEBUGGING */
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
            /* DEBUGGING */
            yylog!(self, "Error: popping {} {:?}", Self::token_class(val.index()), val);
            yylog!(self, "Stack now {:?}", self.state_stack);
            /* DEBUGGING */
        }

        let Some(i) = self.state_stack.last() else {
            self.yyabort();
            return;
        };
        self.is_recovering = true;
        let act = Self::YY_GOTO_TABLE[*i][Self::YY_ERROR_TOKEN_ID] + 1;
        /* DEBUGGING */
        yylog!(self, "Shifting token {:?}", YYToken::error);
        /* DEBUGGING */
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

    /* DEBUGGING */
    fn token_class(token_id: usize) -> &'static str {
        if Self::YY_TERMINAL_TABLE[token_id] {
            "nterm"
        } else {
            "token"
        }
    }
    /* DEBUGGING */

    fn push_statcks(&mut self, state: usize, value: YYToken) {
        self.state_stack.push(state);
        self.value_stack.push(value);
        /* DEBUGGING */
        yylog!(self, "Entering state {}", state);
        yylog!(self, "Stack now {:?}", self.state_stack)
        /* DEBUGGING */
    }

    fn read_token(&mut self) {
        self.lookahead = self.lexer.next();
        self.lookahead_id = self.lookahead.as_ref().unwrap().index();
        /* DEBUGGING */
        yylog!(self, "Reading a token");
        match &self.lookahead {
            None => yylog!(self, "Now at end of input."),
            Some(l) => yylog!(self, "Next token is token {:?}", l),
        }
        /* DEBUGGING */
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

        /* DEBUGGING */
        yylog!(
            self,
            "Shifting token {:?}",
            self.lookahead.clone().unwrap_or(YYToken::yyeof)
        );
        /* DEBUGGING */
        self.push_statcks(-self.act as usize, self.lookahead.clone().unwrap_or(YYToken::yyeof));

        self.lookahead = None;
    }

    fn reduce(&mut self) {
        self.act -= 1;

        /* DEBUGGING */
        yylog!(
            self,
            "Reducing stack by rule {} (line {}):",
            self.act,
            Self::YY_PRODUCTION_LINE[self.act as usize]
        );
        /* DEBUGGING */
        let len = Self::YY_RLEN_TABLE[self.act as usize];
        let product = Self::YY_PRODUCT_TABLE[self.act as usize];

        let yyval = self.action();
        if yyval == YYToken::error {
            self.error();
            return;
        }

        let &top = self.state_stack.last().unwrap();

        match Self::YY_GOTO_TABLE[top][product] + 1 {
            goto => self.push_statcks(-goto as usize, yyval),
        }
    }

    #[allow(unused_braces)]
    fn action(&mut self) -> YYToken {
        let idx = self.value_stack.len() - Self::YY_RLEN_TABLE[self.act as usize];
        let values = self.value_stack.drain(idx..).collect::<Vec<YYToken>>().clone();
        let _a = self.state_stack.drain(idx..).collect::<Vec<_>>().clone();

        /* DEBUGGING */
        for (i, tok) in values.iter().enumerate().rev() {
            yylog!(
                self,
                "   ${} = {} {:?}",
                i + 1,
                Self::token_class(self.lookahead_id),
                tok
            );
        }
        /* DEBUGGING */

        let ret = match Self::YY_ACTION_TABLE[self.act as usize] {
            /* ACTIONS */
            -1 => self.default_prod_table[self.act as usize].clone(),
            _ => unreachable!(),
        };

        /* DEBUGGING */
        yylog!(self, "-> $$ = {} {:?}", Self::token_class(self.lookahead_id), ret);
        /* DEBUGGING */
        ret
    }
}

pub fn yyerror<D: fmt::Display>(msg: D) {
    eprintln!("{}", msg);
}

/* MAIN */
fn main() {
    use std::process::exit;

    let mut lexer = YYLex::default();
    let mut yacc = Yacc::new();
    exit(yacc.yyparse());
}
