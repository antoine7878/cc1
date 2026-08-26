use crate::ast::{Expression, ExpressionNode, Value, Visitor};
use crate::parser::Context;
use crate::semantic::{Diagnosis, DiagnosisNode, SymbolKind, constrain};

#[derive(Default, Debug)]
pub struct ConstChecker {
    pub diagnosis: Vec<DiagnosisNode>,
}

impl ConstChecker {
    fn is_const(&mut self, ctx: &Context, node: &ExpressionNode) -> bool {
        match node.id.resolve(ctx) {
            Expression::ConstantExpression(expr) => self.is_const(ctx, expr),
            Expression::SizeofType(_) | Expression::Constant(_) => true,
            Expression::Identifier(_) => {
                true
                // let id = ctx.bindings.get(&expr.id).copied().flatten()?;
                // let s = id.resolve(ctx);
                // if s.kind != SymbolKind::Variant {
                //     return None;
                // }
                // s.value.map(Value::Int)
            }
            Expression::Cast(_, expr)
            | Expression::SizeofExpr(expr)
            | Expression::Plus(expr)
            | Expression::Minus(expr)
            | Expression::BitNot(expr)
            | Expression::LogicalNot(expr) => self.is_const(ctx, expr),
            Expression::StringLiteral(_)
            | Expression::PostInc(_)
            | Expression::PostDec(_)
            | Expression::PreInc(_)
            | Expression::Deref(_)
            | Expression::Addr(_)
            | Expression::Assign(_, _)
            | Expression::MulAssign(_, _)
            | Expression::DivAssign(_, _)
            | Expression::ModAssign(_, _)
            | Expression::AddAssign(_, _)
            | Expression::SubAssign(_, _)
            | Expression::LeftAssign(_, _)
            | Expression::RightAssign(_, _)
            | Expression::AndAssign(_, _)
            | Expression::XorAssign(_, _)
            | Expression::OrAssign(_, _)
            | Expression::List(_, _)
            | Expression::ArrayAcces(_, _)
            | Expression::FunctionCall(_, _)
            | Expression::DotAcces(_, _)
            | Expression::PtrAcces(_, _)
            | Expression::PreDec(_) => false,
            Expression::Add(e1, e2)
            | Expression::Sub(e1, e2)
            | Expression::Mul(e1, e2)
            | Expression::Div(e1, e2)
            | Expression::Mod(e1, e2)
            | Expression::Left(e1, e2)
            | Expression::Right(e1, e2)
            | Expression::BitAnd(e1, e2)
            | Expression::BitOr(e1, e2)
            | Expression::BitXor(e1, e2)
            | Expression::Greater(e1, e2)
            | Expression::Lower(e1, e2)
            | Expression::GreaterEq(e1, e2)
            | Expression::LowerEq(e1, e2)
            | Expression::Eq(e1, e2)
            | Expression::Neq(e1, e2)
            | Expression::LogicalAnd(e1, e2)
            | Expression::LogicalOr(e1, e2) => self.is_const(ctx, e1) && self.is_const(ctx, e2),
            Expression::Ternary(e1, e2, e3) => {
                self.is_const(ctx, e1) && self.is_const(ctx, e2) && self.is_const(ctx, e3)
            }
        }
    }
}

impl Visitor for ConstChecker {
    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {}
}
// pub fn const_eval(ctx: &Context, expr: &ExpressionNode) -> Result<Value, Diagnosis> {
//     match expr.id.resolve(ctx) {
//         Expression::ConstantExpression(expr) => const_eval(ctx, expr),
//         Expression::Identifier(_) => {
//             let id = ctx.bindings.get(&expr.id).copied().flatten()?;
//             let s = id.resolve(ctx);
//             if s.kind != SymbolKind::Variant {
//                 return None;
//             }
//             s.value.map(Value::Int)
//         }
//         Expression::Constant(value_node) => Some(value_node.value),
//         Expression::Plus(expr) => const_eval(ctx, expr),
//         Expression::Minus(expr) => Some(-const_eval(ctx, expr)?),
//         Expression::BitNot(expr) => Some(!const_eval(ctx, expr)?),
//         Expression::LogicalNot(expr) => Some(const_eval(ctx, expr)?.logical_not()),
//         Expression::Add(e1, e2) => Some(const_eval(ctx, e1)? + const_eval(ctx, e2)?),
//         Expression::Sub(e1, e2) => Some(const_eval(ctx, e1)? - const_eval(ctx, e2)?),
//         Expression::Mul(e1, e2) => Some(const_eval(ctx, e1)? * const_eval(ctx, e2)?),
//         Expression::Div(e1, e2) => Some(const_eval(ctx, e1)? / const_eval(ctx, e2)?),
//         Expression::Mod(e1, e2) => Some(const_eval(ctx, e1)? % const_eval(ctx, e2)?),
//         Expression::Left(e1, e2) => Some(const_eval(ctx, e1)? << const_eval(ctx, e2)?),
//         Expression::Right(e1, e2) => Some(const_eval(ctx, e1)? >> const_eval(ctx, e2)?),
//         Expression::BitAnd(e1, e2) => Some(const_eval(ctx, e1)? & const_eval(ctx, e2)?),
//         Expression::BitOr(e1, e2) => Some(const_eval(ctx, e1)? | const_eval(ctx, e2)?),
//         Expression::BitXor(e1, e2) => Some(const_eval(ctx, e1)? ^ const_eval(ctx, e2)?),
//         Expression::Greater(e1, e2) => Some(Value::from(const_eval(ctx, e1)? > const_eval(ctx, e2)?)),
//         Expression::Lower(e1, e2) => Some(Value::from(const_eval(ctx, e1)? < const_eval(ctx, e2)?)),
//         Expression::GreaterEq(e1, e2) => Some(Value::from(const_eval(ctx, e1)? >= const_eval(ctx, e2)?)),
//         Expression::LowerEq(e1, e2) => Some(Value::from(const_eval(ctx, e1)? <= const_eval(ctx, e2)?)),
//         Expression::Eq(e1, e2) => Some(Value::from(const_eval(ctx, e1)? == const_eval(ctx, e2)?)),
//         Expression::Neq(e1, e2) => Some(Value::from(const_eval(ctx, e1)? != const_eval(ctx, e2)?)),
//         Expression::LogicalAnd(e1, e2) => Some(Value::from(
//             const_eval(ctx, e1)?.is_true() && const_eval(ctx, e2)?.is_true(),
//         )),
//         Expression::LogicalOr(e1, e2) => Some(Value::from(
//             const_eval(ctx, e1)?.is_true() || const_eval(ctx, e2)?.is_true(),
//         )),
//         Expression::Ternary(condition, e1, e2) => {
//             if const_eval(ctx, condition)?.is_true() {
//                 const_eval(ctx, e1)
//             } else {
//                 const_eval(ctx, e2)
//             }
//         }
//         Expression::SizeofExpr(_expr) => unimplemented!(),
//         Expression::SizeofType(ty) => {
//             let qualif = constrain::declaration::resolve_type(ctx, &ty.specifiers, &expr.span)?;
//             Some(Value::UnsignedLong(type_size(ctx, qualif)?))
//         }
//         Expression::Cast(ty, operand) => {
//             let base = constrain::declaration::resolve_type(ctx, &ty.specifiers, &expr.span);
//             let (qualif, _) = make_qualified_type(ctx, base, &ty.declarator)?;
//             let val = const_eval(ctx, operand)?;
//             if val.is_floating() && !matches!(operand.id.resolve(ctx), Expression::Constant(_)) {
//                 return None;
//             }
//             cast_value(ctx, qualif, val, &expr.span)
//         }
//         Expression::StringLiteral(_)
//         | Expression::PostInc(_)
//         | Expression::PostDec(_)
//         | Expression::PreInc(_)
//         | Expression::Deref(_)
//         | Expression::Addr(_)
//         | Expression::Assign(_, _)
//         | Expression::MulAssign(_, _)
//         | Expression::DivAssign(_, _)
//         | Expression::ModAssign(_, _)
//         | Expression::AddAssign(_, _)
//         | Expression::SubAssign(_, _)
//         | Expression::LeftAssign(_, _)
//         | Expression::RightAssign(_, _)
//         | Expression::AndAssign(_, _)
//         | Expression::XorAssign(_, _)
//         | Expression::OrAssign(_, _)
//         | Expression::List(_, _)
//         | Expression::ArrayAcces(_, _)
//         | Expression::FunctionCall(_, _)
//         | Expression::DotAcces(_, _)
//         | Expression::PtrAcces(_, _)
//         | Expression::PreDec(_) => None,
//     }
// }
