use std::fmt::{self, Display, Formatter};
use std::io::Write;

use crate::ast::Value;
use crate::codegen::TyName;

#[derive(Debug)]
pub enum LLVMValue {
    SSA(usize),
    Literal(Value),
    // Global(NameId),
}

impl Display for LLVMValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LLVMValue::SSA(id) => write!(f, "%{id}"),
            LLVMValue::Literal(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    SDiv,
    SRem,
    FAdd,
    FSub,
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Op::Add => "add nsw",
            Op::Sub => "sub nsw",
            Op::Mul => "mul nsw",
            Op::SDiv => "sdiv",
            Op::SRem => "srem",
            Op::FAdd => "fadd",
            Op::FSub => "fsub",
        })
    }
}

#[derive(Debug)]
pub struct Builder<W: Write> {
    w: W,
    counter: usize,
}

impl<W: Write> Builder<W> {
    pub fn new(w: W) -> Self {
        Self { w, counter: 0 }
    }

    pub fn reset(&mut self, counter: usize) {
        self.counter = counter;
    }

    fn fresh(&mut self) -> LLVMValue {
        self.counter += 1;
        LLVMValue::SSA(self.counter)
    }

    fn line(&mut self, args: fmt::Arguments<'_>) {
        let _ = self.w.write_fmt(args);
        let _ = self.w.write_all(b"\n");
    }

    pub fn target(&mut self, datalayout: &str, triple: &str) {
        self.line(format_args!(r#"target datalayout = "{datalayout}""#));
        self.line(format_args!(r#"target triple = "{triple}""#));
    }

    pub fn blank(&mut self) {
        self.line(format_args!(""));
    }

    pub fn define(&mut self, ret: TyName<'_>, name: impl Display) {
        self.line(format_args!("define {ret} @{name}() {{"));
    }

    pub fn end_function(&mut self) {
        self.line(format_args!("}}"));
    }

    pub fn alloca(&mut self, id: usize, ty: TyName<'_>, align: u32) {
        self.line(format_args!("  %{id} = alloca {ty}, align {align}"));
    }

    pub fn load(&mut self, ty: TyName<'_>, slot: usize, align: u32) -> LLVMValue {
        let r = self.fresh();
        self.line(format_args!("  {r} = load {ty}, ptr %{slot}, align {align}"));
        r
    }

    pub fn binop(&mut self, op: Op, ty: TyName<'_>, a: LLVMValue, b: LLVMValue) -> LLVMValue {
        let r = self.fresh();
        self.line(format_args!("  {r} = {op} {ty} {a}, {b}"));
        r
    }

    pub fn ret(&mut self, ty: TyName<'_>, v: LLVMValue) {
        self.line(format_args!("  ret {ty} {v}"));
    }

    pub fn ret_void(&mut self) {
        self.line(format_args!("  ret void"));
    }
}
