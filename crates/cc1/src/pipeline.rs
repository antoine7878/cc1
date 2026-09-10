use std::process::exit;

use crate::context::Context;
use crate::semantic::{DiagnosisNode, Sema};

pub trait Errors {
    fn error_count(&self) -> usize;
}

fn count(diagnosis: &[DiagnosisNode]) -> usize {
    diagnosis.iter().filter(|d| d.is_error()).count()
}

impl Errors for Context {
    fn error_count(&self) -> usize {
        count(&self.diagnosis)
    }
}

impl Errors for Sema {
    fn error_count(&self) -> usize {
        count(&self.diagnosis)
    }
}

impl Errors for () {
    fn error_count(&self) -> usize {
        0
    }
}

pub struct Pipeline<T> {
    state: T,
    exit_code: i32,
    stopped: bool,
}

impl Default for Pipeline<Context> {
    fn default() -> Self {
        Self::new(Context::default())
    }
}

impl<T: Errors> Pipeline<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            exit_code: 0,
            stopped: false,
        }
    }

    pub fn pass(mut self, pass: fn(T) -> T) -> Self {
        if self.stopped {
            return self;
        }
        let watermark = self.state.error_count();
        self.state = pass(self.state);
        if self.state.error_count() > watermark {
            self.exit_code = 1;
        }
        self
    }

    pub fn pass_group<const N: usize>(mut self, fns: [fn(T) -> T; N]) -> Self {
        for f in fns {
            self = self.pass(f);
        }
        self.checkpoint()
    }

    pub fn then<U: Errors>(self, f: fn(T) -> U) -> Pipeline<U> {
        Pipeline {
            state: f(self.state),
            exit_code: self.exit_code,
            stopped: self.stopped,
        }
    }

    pub fn checkpoint(mut self) -> Self {
        self.stopped |= self.failed();
        self
    }

    pub fn report(self, obse: fn(&T)) -> Self {
        if self.stopped {
            return self;
        }
        obse(&self.state);
        self
    }

    pub fn finish(self) -> (T, bool) {
        let stopped = self.stopped;
        (self.state, stopped)
    }

    pub fn stopped(&self) -> bool {
        self.stopped
    }

    pub fn failed(&self) -> bool {
        self.exit_code > 0
    }
}

impl Pipeline<()> {
    pub fn run(self, f: fn()) -> Self {
        if !self.stopped {
            f();
        }
        self
    }

    pub fn finally(self, f: fn()) -> ! {
        f();
        exit(self.exit_code)
    }
}
