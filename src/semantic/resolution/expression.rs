// use crate::ast::{Expression, ExpressionNode};
// use crate::parser::Context;
// use crate::semantic::{Diagnosis, QualifiedType, Sema};
//
// pub fn type_of(sema: &mut Sema, ctx: &Context, expr: &ExpressionNode) -> Result<QualifiedType, Diagnosis> {
//     match expr.id.resolve(ctx) {
//         Expression::ConstantExpression(expr) => type_of(sema, ctx, expr),
//         Expression::Identifier(name) => {
//             let id = sema
//                 .expressions
//                 .get(&expr.id)
//                 .and_then(|re| re.sym)
//                 .ok_or(Diagnosis::UndeclaredIdentifier(*name))?;
//             let symbol = sema.symbols.get(id);
//             Ok(symbol.ty.unwrap())
//         }
//         Expression::Constant(value_node) => Ok(value_node.ty(sema)),
//         Expression::Plus(expr) | Expression::Minus(expr) | Expression::BitNot(expr) | Expression::LogicalNot(expr) => {
//             type_of(sema, ctx, expr)
//         }
//         Expression::Add(e1, e2)
//         | Expression::Sub(e1, e2)
//         | Expression::Mul(e1, e2)
//         | Expression::Div(e1, e2)
//         | Expression::Mod(e1, e2)
//         | Expression::Left(e1, e2)
//         | Expression::Right(e1, e2)
//         | Expression::BitAnd(e1, e2)
//         | Expression::BitOr(e1, e2)
//         | Expression::BitXor(e1, e2)
//         | Expression::Greater(e1, e2)
//         | Expression::Lower(e1, e2)
//         | Expression::GreaterEq(e1, e2)
//         | Expression::LowerEq(e1, e2)
//         | Expression::Eq(e1, e2)
//         | Expression::Neq(e1, e2)
//         | Expression::LogicalOr(e1, e2)
//         | Expression::LogicalAnd(e1, e2) => type_of(sema, ctx, expr),
//         Expression::Ternary(condition, e1, e2) => type_of(sema, ctx, expr),
//         Expression::SizeofExpr(expr) => type_of(sema, ctx, expr),
//         Expression::SizeofType(ty_node) => todo!(),
//         Expression::Cast(ty_node, expr) => type_of(sema, ctx, expr),
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
//         | Expression::PreDec(_) => todo!(),
//     }
// }
