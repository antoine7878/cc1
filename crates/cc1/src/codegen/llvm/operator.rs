use crate::ast::{BinaryOp, UnaryOp};
use crate::codegen::Invariant;
use crate::semantic::{Diagnostic, NumericClass, QualifiedType, sema};

pub struct LlvmOperator;

impl LlvmOperator {
    pub fn unary(op: &UnaryOp, qty: QualifiedType) -> Result<&'static str, Diagnostic> {
        match op {
            UnaryOp::LogicalNot | UnaryOp::BitNot => Ok("xor"),
            UnaryOp::Plus | UnaryOp::PreInc | UnaryOp::PostInc => Self::binary(&BinaryOp::Add, qty),
            UnaryOp::Minus | UnaryOp::PreDec | UnaryOp::PostDec => Self::binary(&BinaryOp::Sub, qty),
            UnaryOp::Deref | UnaryOp::Addr => Err(Diagnostic::Invariant("deref/addr is not an llvm operator")),
        }
    }

    pub fn binary(op: &BinaryOp, qty: QualifiedType) -> Result<&'static str, Diagnostic> {
        let class = match qty.class(sema()) {
            None if qty.is_pointer(sema()) && op.is_comparison() => NumericClass::Unsigned,
            class => class.invariant("binary operator on non-scalar type")?,
        };
        let op = match (op, class) {
            (BinaryOp::Add, NumericClass::Signed) => "add nsw",
            (BinaryOp::Add, NumericClass::Unsigned) => "add",
            (BinaryOp::Add, NumericClass::Float) => "fadd",
            (BinaryOp::Sub, NumericClass::Signed) => "sub nsw",
            (BinaryOp::Sub, NumericClass::Unsigned) => "sub",
            (BinaryOp::Sub, NumericClass::Float) => "fsub",
            (BinaryOp::Mul, NumericClass::Signed) => "mul nsw",
            (BinaryOp::Mul, NumericClass::Unsigned) => "mul",
            (BinaryOp::Mul, NumericClass::Float) => "fmul",
            (BinaryOp::Div, NumericClass::Signed) => "sdiv",
            (BinaryOp::Div, NumericClass::Unsigned) => "udiv",
            (BinaryOp::Div, NumericClass::Float) => "fdiv",
            (BinaryOp::Mod, NumericClass::Signed) => "srem",
            (BinaryOp::Mod, NumericClass::Unsigned) => "urem",
            (BinaryOp::Left, NumericClass::Signed) => "shl nsw",
            (BinaryOp::Left, NumericClass::Unsigned) => "shl",
            (BinaryOp::Right, NumericClass::Signed) => "ashr",
            (BinaryOp::Right, NumericClass::Unsigned) => "lshr",
            (BinaryOp::BitAnd, _) => "and",
            (BinaryOp::BitOr, _) => "or",
            (BinaryOp::BitXor, _) => "xor",
            (BinaryOp::Eq, NumericClass::Float) => "fcmp oeq",
            (BinaryOp::Neq, NumericClass::Float) => "fcmp une",
            (BinaryOp::Eq, _) => "icmp eq",
            (BinaryOp::Neq, _) => "icmp ne",
            (BinaryOp::Lower, NumericClass::Signed) => "icmp slt",
            (BinaryOp::Lower, NumericClass::Unsigned) => "icmp ult",
            (BinaryOp::Lower, NumericClass::Float) => "fcmp olt",
            (BinaryOp::Greater, NumericClass::Signed) => "icmp sgt",
            (BinaryOp::Greater, NumericClass::Unsigned) => "icmp ugt",
            (BinaryOp::Greater, NumericClass::Float) => "fcmp ogt",
            (BinaryOp::LowerEq, NumericClass::Signed) => "icmp sle",
            (BinaryOp::LowerEq, NumericClass::Unsigned) => "icmp ule",
            (BinaryOp::LowerEq, NumericClass::Float) => "fcmp ole",
            (BinaryOp::GreaterEq, NumericClass::Signed) => "icmp sge",
            (BinaryOp::GreaterEq, NumericClass::Unsigned) => "icmp uge",
            (BinaryOp::GreaterEq, NumericClass::Float) => "fcmp oge",
            (BinaryOp::LogicalAnd | BinaryOp::LogicalOr, NumericClass::Float) => "fcmp une",
            (BinaryOp::LogicalAnd | BinaryOp::LogicalOr, _) => "icmp ne",
            (BinaryOp::Mod | BinaryOp::Left | BinaryOp::Right, NumericClass::Float) => {
                return Err(Diagnostic::Invariant("integer-only operator on floating type"));
            }
        };
        Ok(op)
    }
}
