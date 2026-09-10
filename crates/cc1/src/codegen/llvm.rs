use std::fmt::{self, Display, Formatter};
use std::io::Write;

use crate::ast::{self, BinaryOp, Value};
use crate::codegen::TyName;
use crate::codegen::local::Local;
use crate::semantic::{QualifiedType, ResolvedType};

#[derive(Debug)]
pub enum LLVMValue {
    SSA(usize),
    Literal(Value),
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
    UDiv,
    SDiv,
    URem,
    SRem,
    FAdd,
    FSub,
    FMul,
    FDiv,
    FRem,
    Shl,
    LShr,
    AShr,
    And,
    Or,
    Xor,
}

impl Op {
    fn new(value: QualifiedType, op: ast::BinaryOp) -> Self {
        Op::Xor
        // match op {
        //     BinaryOp::Add => Op::Add, }
        // match value.id.resolve() {
        //     ResolvedType::Char
        //     | ResolvedType::SignedChar
        //     | ResolvedType::Short
        //     | ResolvedType::Int
        //     | ResolvedType::Long => Op::Add,
        //     ResolvedType::UnsignedInt
        //     | ResolvedType::UnsignedChar
        //     | ResolvedType::UnsignedShort
        //     | ResolvedType::UnsignedLong => Op::Add,
        //     _ => Op::Xor,
        // }
    }
}
impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Op::Add => "add",
            Op::Sub => "sub",
            Op::Mul => "mul",
            Op::UDiv => "udiv",
            Op::SDiv => "sdiv",
            Op::URem => "urem",
            Op::SRem => "srem",
            Op::FAdd => "fadd",
            Op::FSub => "fsub",
            Op::FMul => "fmul",
            Op::FDiv => "fdiv",
            Op::FRem => "frem",
            Op::Shl => "shl",
            Op::LShr => "lshr",
            Op::AShr => "ashr",
            Op::And => "and",
            Op::Or => "or",
            Op::Xor => "xor",
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

    pub fn define(&mut self, ret: TyName, name: impl Display) {
        self.line(format_args!("define {ret} @{name}() {{"));
    }

    pub fn end_function(&mut self) {
        self.line(format_args!("}}"));
    }

    pub fn alloca(&mut self, slot: Local, ty: TyName, align: u32) {
        self.line(format_args!("  {slot} = alloca {ty}, align {align}"));
    }

    pub fn load(&mut self, ty: TyName, slot: Local, align: u32) -> LLVMValue {
        let r = self.fresh();
        self.line(format_args!("  {r} = load {ty}, ptr {slot}, align {align}"));
        r
    }

    pub fn binop(&mut self, op: Op, ty: TyName, a: LLVMValue, b: LLVMValue) -> LLVMValue {
        let r = self.fresh();
        self.line(format_args!("  {r} = {op} {ty} {a}, {b}"));
        r
    }

    pub fn ret(&mut self, ty: TyName, v: LLVMValue) {
        self.line(format_args!("  ret {ty} {v}"));
    }

    pub fn ret_void(&mut self) {
        self.line(format_args!("  ret void"));
    }
}
