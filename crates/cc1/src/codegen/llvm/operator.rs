use crate::ast::{self, BinaryOp};
use crate::semantic::{QualifiedType, sema};

pub struct LlvmOperator;

impl LlvmOperator {
    pub fn get(qty: QualifiedType, op: &ast::BinaryOp) -> &'static str {
        let is_signed = qty.is_signed(sema());
        let is_unsigned = qty.is_unsigned(sema());
        let is_float = qty.is_floating(sema());
        match op {
            BinaryOp::Add if is_signed => "add nsw",
            BinaryOp::Add if is_unsigned => "add",
            BinaryOp::Add if is_float => "fadd",
            BinaryOp::Sub if is_signed => "sub nsw",
            BinaryOp::Sub if is_unsigned => "sub",
            BinaryOp::Sub if is_float => "fsub",
            BinaryOp::Mul if is_signed => "mul nsw",
            BinaryOp::Mul if is_unsigned => "mul",
            BinaryOp::Mul if is_float => "fmul",
            BinaryOp::Div if is_signed => "sdiv",
            BinaryOp::Div if is_unsigned => "udiv",
            BinaryOp::Div if is_float => "fdiv",
            BinaryOp::Mod if is_signed => "srem",
            BinaryOp::Mod if is_unsigned => "urem",
            BinaryOp::Left if is_signed => "shl nsw",
            BinaryOp::Left if is_unsigned => "shl",
            BinaryOp::Right if is_signed => "ashr",
            BinaryOp::Right if is_unsigned => "lshr",
            BinaryOp::BitAnd => "and",
            BinaryOp::BitOr => "or",
            BinaryOp::BitXor => "xor",
            BinaryOp::Lower if is_signed => "icmp slt",
            BinaryOp::Lower if is_unsigned => "icmp ult",
            BinaryOp::Lower if is_float => "fcmp olt",
            BinaryOp::Greater if is_signed => "icmp sgt",
            BinaryOp::Greater if is_unsigned => "icmp ugt",
            BinaryOp::Greater if is_float => "fcmp ogt",
            BinaryOp::LowerEq if is_signed => "icmp sle",
            BinaryOp::LowerEq if is_unsigned => "icmp ule",
            BinaryOp::LowerEq if is_float => "fcmp ole",
            BinaryOp::GreaterEq if is_signed => "icmp sge",
            BinaryOp::GreaterEq if is_unsigned => "icmp uge",
            BinaryOp::GreaterEq if is_float => "fcmp oge",
            BinaryOp::Eq if is_signed => "icmp eq",
            BinaryOp::Eq if is_unsigned => "icmp eq",
            BinaryOp::Eq if is_float => "fcmp oeq",
            BinaryOp::Neq if is_signed => "icmp ne",
            BinaryOp::Neq if is_unsigned => "icmp ne",
            BinaryOp::Neq if is_float => "fcmp one",
            _ => todo!(),
        }
    }
}
