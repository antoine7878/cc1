use crate::ast::{BinaryOp, UnaryOp};
use crate::semantic::{Class, QualifiedType, sema};

pub struct LlvmOperator;

impl LlvmOperator {
    pub fn unary(op: &UnaryOp, qty: QualifiedType) -> &'static str {
        use UnaryOp::*;

        let Some(class) = qty.class(sema()) else { todo!() };
        match (op, class) {
            (LogicalNot | BitNot, _) => "xor",
            (Plus | PreInc | PostInc, _) => Self::binary(&BinaryOp::Add, qty),
            (Minus | PreDec | PostDec, _) => Self::binary(&BinaryOp::Sub, qty),
            _ => todo!(),
        }
    }

    pub fn binary(op: &BinaryOp, qty: QualifiedType) -> &'static str {
        use BinaryOp::*;
        use Class::*;
        let Some(class) = qty.class(sema()) else { todo!() };
        match (op, class) {
            (Add, Signed) => "add nsw",
            (Add, Unsigned) => "add",
            (Add, Float) => "fadd",
            (Sub, Signed) => "sub nsw",
            (Sub, Unsigned) => "sub",
            (Sub, Float) => "fsub",
            (Mul, Signed) => "mul nsw",
            (Mul, Unsigned) => "mul",
            (Mul, Float) => "fmul",
            (Div, Signed) => "sdiv",
            (Div, Unsigned) => "udiv",
            (Div, Float) => "fdiv",
            (Mod, Signed) => "srem",
            (Mod, Unsigned) => "urem",
            (Left, Signed) => "shl nsw",
            (Left, Unsigned) => "shl",
            (Right, Signed) => "ashr",
            (Right, Unsigned) => "lshr",
            (BitAnd, _) => "and",
            (BitOr, _) => "or",
            (BitXor, _) => "xor",
            (Eq, Float) => "fcmp oeq",
            (Neq, Float) => "fcmp one",
            (Eq, _) => "icmp eq",
            (Neq, _) => "icmp ne",
            (Lower, Signed) => "icmp slt",
            (Lower, Unsigned) => "icmp ult",
            (Lower, Float) => "fcmp olt",
            (Greater, Signed) => "icmp sgt",
            (Greater, Unsigned) => "icmp ugt",
            (Greater, Float) => "fcmp ogt",
            (LowerEq, Signed) => "icmp sle",
            (LowerEq, Unsigned) => "icmp ule",
            (LowerEq, Float) => "fcmp ole",
            (GreaterEq, Signed) => "icmp sge",
            (GreaterEq, Unsigned) => "icmp uge",
            (GreaterEq, Float) => "fcmp oge",
            (LogicalAnd | LogicalOr, _) => "icmp ne",
            (Mod | Left | Right, Float) => unreachable!(),
        }
    }
}
