use crate::ast::{BinaryOp, UnaryOp};
use crate::codegen::Invariant;
use crate::semantic::{Class, Diagnosis, QualifiedType, sema};

pub struct LlvmOperator;

impl LlvmOperator {
    pub fn unary(op: &UnaryOp, qty: QualifiedType) -> Result<&'static str, Diagnosis> {
        let class = qty.class(sema()).invariant("unary operator on non-scalar type")?;
        match (op, class) {
            (UnaryOp::LogicalNot | UnaryOp::BitNot, _) => Ok("xor"),
            (UnaryOp::Plus | UnaryOp::PreInc | UnaryOp::PostInc, _) => Self::binary(&BinaryOp::Add, qty),
            (UnaryOp::Minus | UnaryOp::PreDec | UnaryOp::PostDec, _) => Self::binary(&BinaryOp::Sub, qty),
            (UnaryOp::Deref | UnaryOp::Addr, _) => Err(Diagnosis::Invariant("deref/addr is not an llvm operator")),
        }
    }

    pub fn binary(op: &BinaryOp, qty: QualifiedType) -> Result<&'static str, Diagnosis> {
        let class = qty.class(sema()).invariant("binary operator on non-scalar type")?;
        let op = match (op, class) {
            (BinaryOp::Add, Class::Signed) => "add nsw",
            (BinaryOp::Add, Class::Unsigned) => "add",
            (BinaryOp::Add, Class::Float) => "fadd",
            (BinaryOp::Sub, Class::Signed) => "sub nsw",
            (BinaryOp::Sub, Class::Unsigned) => "sub",
            (BinaryOp::Sub, Class::Float) => "fsub",
            (BinaryOp::Mul, Class::Signed) => "mul nsw",
            (BinaryOp::Mul, Class::Unsigned) => "mul",
            (BinaryOp::Mul, Class::Float) => "fmul",
            (BinaryOp::Div, Class::Signed) => "sdiv",
            (BinaryOp::Div, Class::Unsigned) => "udiv",
            (BinaryOp::Div, Class::Float) => "fdiv",
            (BinaryOp::Mod, Class::Signed) => "srem",
            (BinaryOp::Mod, Class::Unsigned) => "urem",
            (BinaryOp::Left, Class::Signed) => "shl nsw",
            (BinaryOp::Left, Class::Unsigned) => "shl",
            (BinaryOp::Right, Class::Signed) => "ashr",
            (BinaryOp::Right, Class::Unsigned) => "lshr",
            (BinaryOp::BitAnd, _) => "and",
            (BinaryOp::BitOr, _) => "or",
            (BinaryOp::BitXor, _) => "xor",
            (BinaryOp::Eq, Class::Float) => "fcmp oeq",
            (BinaryOp::Neq, Class::Float) => "fcmp une",
            (BinaryOp::Eq, _) => "icmp eq",
            (BinaryOp::Neq, _) => "icmp ne",
            (BinaryOp::Lower, Class::Signed) => "icmp slt",
            (BinaryOp::Lower, Class::Unsigned) => "icmp ult",
            (BinaryOp::Lower, Class::Float) => "fcmp olt",
            (BinaryOp::Greater, Class::Signed) => "icmp sgt",
            (BinaryOp::Greater, Class::Unsigned) => "icmp ugt",
            (BinaryOp::Greater, Class::Float) => "fcmp ogt",
            (BinaryOp::LowerEq, Class::Signed) => "icmp sle",
            (BinaryOp::LowerEq, Class::Unsigned) => "icmp ule",
            (BinaryOp::LowerEq, Class::Float) => "fcmp ole",
            (BinaryOp::GreaterEq, Class::Signed) => "icmp sge",
            (BinaryOp::GreaterEq, Class::Unsigned) => "icmp uge",
            (BinaryOp::GreaterEq, Class::Float) => "fcmp oge",
            (BinaryOp::LogicalAnd | BinaryOp::LogicalOr, _) => "icmp ne",
            (BinaryOp::Mod | BinaryOp::Left | BinaryOp::Right, Class::Float) => {
                return Err(Diagnosis::Invariant("integer-only operator on floating type"));
            }
        };
        Ok(op)
    }
}
