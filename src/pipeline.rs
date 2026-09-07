use std::process::exit;

use crate::context::Context;
use crate::semantic::DiagnosisNode;

#[derive(Default)]
pub struct Pipeline {
    ctx: Context,
    exit_code: i32,
    stopped: bool,
}

impl Pipeline {
    pub fn pass(mut self, pass: fn(Context) -> Context) -> Self {
        if self.stopped {
            return self;
        }
        let watermark = self.ctx.diagnosis.len();
        self.ctx = pass(self.ctx);
        debug_assert!(self.ctx.diagnosis.len() >= watermark);
        if self.ctx.diagnosis[watermark..].iter().any(DiagnosisNode::is_error) {
            self.exit_code = 1;
        }
        self
    }

    pub fn pass_group<const N: usize>(mut self, fns: [fn(Context) -> Context; N]) -> Self {
        for f in fns {
            self = self.pass(f);
        }
        self.checkpoint()
    }

    pub fn checkpoint(mut self) -> Self {
        self.stopped |= self.failed();
        self
    }

    pub fn report(self, obse: fn(&Context)) -> Self {
        if self.stopped {
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
        let stopped = self.stopped;
        (self.ctx, stopped)
    }

    pub fn stopped(&self) -> bool {
        self.stopped
    }

    pub fn failed(&self) -> bool {
        self.exit_code > 0
    }
}
