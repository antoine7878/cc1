use std::env::args;
use std::process::exit;

use crate::context::Context;
use crate::parser::Span;
use crate::semantic::{Diagnosis, DiagnosisNode};

#[derive(Default)]
pub struct Pipeline {
    ctx: Context,
    exit_code: i32,
}

impl Pipeline {
    pub fn pass(mut self, pass: fn(Context) -> Context) -> Self {
        if self.stopped() {
            return self;
        }
        let watermark = self.ctx.diagnosis.len();
        self.ctx = pass(self.ctx);
        debug_assert!(self.ctx.diagnosis.len() >= watermark);
        self.exit_code = if self.ctx.diagnosis[watermark..].iter().any(DiagnosisNode::is_error) { 1 } else { 0 };
        self
    }

    pub fn report(self, obse: fn(&Context)) -> Self {
        if self.stopped() {
            return self;
        }
        obse(&self.ctx);
        self
    }

    pub fn finally(self, f: fn(&Context)) -> ! {
        f(&self.ctx);
        exit(self.exit_code)
    }

    pub fn finish(self) -> (Context, bool) {
        let stopped = self.stopped();
        (self.ctx, stopped)
    }

    pub fn stopped(&self) -> bool {
        self.exit_code > 0
    }
}

pub fn parse_args(mut ctx: Context) -> Context {
    println!();
    let mut args = args();
    if args.next().is_none() {
        ctx.diagnosis
            .push(DiagnosisNode::new(Diagnosis::BadArgumentsCount, Span::default()));
        return ctx;
    };
    let Some(file_name) = args.next() else {
        ctx.diagnosis
            .push(DiagnosisNode::new(Diagnosis::BadArgumentsCount, Span::default()));
        return ctx;
    };
    ctx.set_file_name(file_name);
    ctx
}
