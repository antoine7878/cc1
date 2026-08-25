use crate::parser::Context;
use crate::semantic::DiagnosisNode;

pub struct Pipeline {
    ctx: Context,
    stopped: bool,
}

impl Pipeline {
    pub fn new(ctx: Context) -> Self {
        let stopped = ctx.diagnosis.iter().any(DiagnosisNode::is_error);
        Self { ctx, stopped }
    }

    pub fn then(mut self, pass: fn(Context) -> Context) -> Self {
        if self.stopped {
            return self;
        }
        let watermark = self.ctx.diagnosis.len();
        self.ctx = pass(self.ctx);
        debug_assert!(self.ctx.diagnosis.len() >= watermark);
        self.stopped = self.ctx.diagnosis[watermark..].iter().any(DiagnosisNode::is_error);
        self
    }

    pub fn tap(self, observe: fn(&Context)) -> Self {
        observe(&self.ctx);
        self
    }

    pub fn stopped(&self) -> bool {
        self.stopped
    }

    pub fn finish(self) -> (Context, bool) {
        (self.ctx, self.stopped)
    }
}
