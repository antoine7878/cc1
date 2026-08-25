use std::process::exit;

use crate::parser::Context;
use crate::semantic::DiagnosisNode;

#[derive(Default)]
pub struct Pipeline {
    ctx: Context,
    exit_code: i32,
}

impl Pipeline {
    pub fn then(mut self, pass: fn(Context) -> Context) -> Self {
        if self.stopped() {
            return self;
        }
        let watermark = self.ctx.diagnosis.len();
        self.ctx = pass(self.ctx);
        debug_assert!(self.ctx.diagnosis.len() >= watermark);
        self.exit_code = if self.ctx.diagnosis[watermark..].iter().any(DiagnosisNode::is_error) { 1 } else { 0 };
        self
    }

    pub fn peek(self, observe: fn(&Context)) -> Self {
        if self.stopped() {
            return self;
        }
        observe(&self.ctx);
        self
    }

    pub fn finally(self, f: fn(&Context)) -> ! {
        f(&self.ctx);
        exit(self.exit_code)
    }

    pub fn stopped(&self) -> bool {
        self.exit_code > 0
    }
}
