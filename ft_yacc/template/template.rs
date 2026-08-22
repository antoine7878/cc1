#![allow(unused_braces, mixed_script_confusables, unused)]
use std::fmt;
use std::io::Read;

/* CODE_BEFORE */

/* TOKENS */

/* FEEDBACK */
pub trait YYFeedback {
    fn yy_feedback(&mut self, accepts: &dyn Fn(usize) -> bool);
}
/* FEEDBACK */

pub struct Yacc<R: Read> {
    state_stack: Vec<usize>,
    value_stack: Vec<YYToken>,
    span_stack: Vec<Span>,
    lookahead: Option<YYToken>,
    lookahead_id: usize,
    lookahead_span: Span,
    act: isize,
    continue_parse: bool,
    is_recovering: bool,
    token_since_error: usize,
    yy_status: i32,
    pub span: Span,
    pub yydebug: bool,
    pub lexer: YYLex<R>,
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

impl<R: Read> Yacc<R> {
    /* REMOVE */
    const YY_EOF_TOKEN_ID: usize = 2;
    const YY_ERROR_TOKEN_ID: usize = 3;
    const YY_GOTO_TABLE: [[isize; 0]; 0] = [];
    const YY_RLEN_TABLE: [usize; 0] = [];
    const YY_DEFAULT_ACT: [usize; 0] = [];
    /* REMOVE */

    /* TABLES */
    /* DEBUGGING_TABLES */

    pub fn new(lexer: YYLex<R>) -> Self {
        Self {
            state_stack: Vec::new(),
            value_stack: Vec::new(),
            span_stack: Vec::new(),
            span: Span::default(),
            lookahead: None,
            lookahead_id: 0,
            lookahead_span: Span::default(),
            yydebug: false,
            act: 0,
            continue_parse: true,
            is_recovering: false,
            lexer,
            token_since_error: 0,
            yy_status: 0,
            /* DEFINES */
        }
    }

    /* FEEDBACK */
    fn yy_accepts_at(state_stack: &[usize], token_id: usize) -> bool {
        let mut stack = state_stack.to_vec();
        for _ in 0..Self::YY_PROBE_LIMIT {
            let &state = stack.last().unwrap();
            let mut act = Self::YY_DEFAULT_ACT[state];
            if act == 0 {
                act = Self::YY_GOTO_TABLE[state][token_id];
            }
            if act == 0 {
                act = Self::YY_DEFAULT_REDUCE_ACT[state];
            }
            match act {
                1 => return true,
                a if a < 0 => return true,
                0 => return false,
                _ => (),
            }
            let rule = (act - 1) as usize;
            let len = Self::YY_RLEN_TABLE[rule];
            if len >= stack.len() {
                return false;
            }
            stack.truncate(stack.len() - len);
            let &top = stack.last().unwrap();
            let goto = Self::YY_GOTO_TABLE[top][Self::YY_PRODUCT_TABLE[rule]];
            if goto >= 0 {
                return false;
            }
            stack.push(-(goto + 1) as usize);
        }
        false
    }

    const YY_PROBE_LIMIT: usize = 1024;
    /* FEEDBACK */

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
        /* DEBUGGING */
        yylog!(self, "Starting parse");
        /* DEBUGGING */

        self.push_statcks(0, YYToken::yyeof, Span::default());
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
            yylog!(self, "Cleanup: popping {:?}", token);
        }
        /* DEBUGGING */
        self.yy_status
    }

    fn error(&mut self) {
        if !self.is_recovering {
            self.unwind()
        } else if matches!(self.lookahead, Some(YYToken::yyeof)) {
            self.yyabort();
        } else {
            self.yyclearin();
        }
    }

    fn unwind(&mut self) {
        yyerror(self.error_message(), self);
        let start = self.span_stack.last().cloned().unwrap_or_default().start;
        while let Some(i) = self.state_stack.last()
            && Self::YY_GOTO_TABLE[*i][Self::YY_ERROR_TOKEN_ID] == 0
        {
            self.state_stack.pop();
            self.span_stack.pop();
            let val = self.value_stack.pop().unwrap();
            /* DEBUGGING */
            yylog!(self, "Error: popping {:?}", val);
            yylog!(self, "Stack now {:?}", self.state_stack);
            /* DEBUGGING */
        }
        let end = self.span_stack.last().cloned().unwrap_or_default().end;

        let Some(i) = self.state_stack.last() else {
            self.yyabort();
            return;
        };
        self.is_recovering = true;
        let act = Self::YY_GOTO_TABLE[*i][Self::YY_ERROR_TOKEN_ID] + 1;

        /* DEBUGGING */
        yylog!(self, "Shifting token {:?}", YYToken::error);
        /* DEBUGGING */

        self.push_statcks((-act) as usize, YYToken::error, Span::new(start, end));
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

    fn error_message(&self) -> String {
        "syntax error".to_string()
    }

    pub fn yyerrok(&mut self) {
        self.is_recovering = false;
    }

    pub fn yyaccept(&mut self) {
        self.continue_parse = false;
    }

    pub fn yyabort(&mut self) {
        self.yy_status = 3;
        self.continue_parse = false;
    }

    pub fn yyrecovering(&mut self) -> bool {
        self.is_recovering
    }

    pub fn yyclearin(&mut self) {
        self.lookahead = None;
    }

    fn push_statcks(&mut self, state: usize, value: YYToken, span: Span) {
        self.state_stack.push(state);
        self.value_stack.push(value);
        self.span_stack.push(span);
        /* DEBUGGING */
        yylog!(self, "Entering state {}", state);
        yylog!(self, "Stack now {:?}", self.state_stack)
        /* DEBUGGING */
    }

    fn read_token(&mut self) {
        /* FEEDBACK */
        let state_stack = &self.state_stack;
        self.lexer
            .yy_feedback(&|token_id| Self::yy_accepts_at(state_stack, token_id));
        /* FEEDBACK */
        let c = self.lexer.yylex();
        self.lookahead_id = c.index();
        self.lookahead_span = self.lexer.span;
        self.lookahead = Some(c);
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

        if self.is_recovering && self.token_since_error >= 2 {
            self.is_recovering = false;
        } else if self.is_recovering {
            self.token_since_error += 1;
        }

        if self.lookahead.is_none() {
            self.read_token();
        }

        /* DEBUGGING */
        yylog!(
            self,
            "Shifting token {:?}",
            self.lookahead.as_ref().unwrap_or(&YYToken::yyeof)
        );
        /* DEBUGGING */
        let token = self.lookahead.take().unwrap_or(YYToken::yyeof);
        self.push_statcks(-self.act as usize, token, self.lexer.span);
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
        if yyval == YYToken::error && Self::YY_ACTION_TABLE[self.act as usize] != -1 {
            self.error();
            return;
        }

        let &top = self.state_stack.last().unwrap();
        self.push_statcks(-(Self::YY_GOTO_TABLE[top][product] + 1) as usize, yyval, self.span);
    }

    #[allow(unused_braces, clippy::let_and_return)]
    fn action(&mut self) -> YYToken {
        let len = self.value_stack.len();
        let idx = len - Self::YY_RLEN_TABLE[self.act as usize];

        if idx == len {
            let pos = self.span_stack[len - 1].end;
            self.span.start = pos;
            self.span.end = pos;
        } else {
            self.span.start = self.span_stack[idx].start;
            self.span.end = self.span_stack[len - 1].end;
        }

        /* DEBUGGING */
        for (i, tok) in self.value_stack[idx..].iter().enumerate().rev() {
            yylog!(self, "   ${} = {:?}", i + 1, tok);
        }
        /* DEBUGGING */

        let ret = self.do_action(idx);
        self.value_stack.truncate(idx);
        self.state_stack.truncate(idx);
        self.span_stack.truncate(idx);

        /* DEBUGGING */
        yylog!(self, "-> $$ = {:?}", ret);
        /* DEBUGGING */
        ret
    }

    fn do_action(&mut self, idx: usize) -> YYToken {
        let rlen = Self::YY_RLEN_TABLE[self.act as usize];
        match Self::YY_ACTION_TABLE[self.act as usize] {
            /* ACTIONS */
            -1 => {
                if rlen == 0 {
                    YYToken::Empty
                } else {
                    std::mem::replace(&mut self.value_stack[idx], YYToken::Empty)
                }
            }
            _ => unreachable!(),
        }
    }
}

/* MAIN */
pub fn yyerror<D: fmt::Display, R: Read>(msg: D, yacc: &Yacc<R>) {
    eprintln!("{}", msg);
}

fn main() {
    use std::process::exit;

    let mut lexer = YYLex::default();
    let mut yacc = Yacc::new();
    exit(yacc.yyparse());
}
