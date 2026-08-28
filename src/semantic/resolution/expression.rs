use crate::ast::{Expression, ExpressionNode};
use crate::parser::Context;
use crate::semantic::{Diag, Diagnosis, DiagnosisNode, ExpressionKind, QualifiedType, ResolvedExpression, Sema};

pub fn run(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) {
    let ty = type_of(sema, ctx, node).collect(sema, &node.span);
    sema.expressions
        .entry(node.id)
        .or_insert(ResolvedExpression::new(ty, ExpressionKind::RValue))
        .ty = ty
    // match ty {
    //     Ok(ty) => {
    //         sema.expressions
    //             .entry(node.id)
    //             .or_insert(ResolvedExpression::new(ty, ExpressionKind::RValue))
    //             .ty = ty
    //     }
    //     Err(diag) => sema.diagnosis.push(DiagnosisNode {
    //         span: node.span,
    //         inner: diag,
    //     }),
    // }
}

fn type_of(sema: &mut Sema, ctx: &Context, node: &ExpressionNode) -> Diag<Option<QualifiedType>> {
    match node.id.resolve(ctx) {
        Expression::Identifier(name) => {
            let Some(id) = sema.bindings.get(&node.id).copied().flatten() else {
                return Diag::none_diag(Diagnosis::UndeclaredIdentifier(*name));
            };
            Diag::res(sema.symbols.get(id).ty)
        }
        Expression::Constant(value) => Diag::some(value.ty(sema)),
        Expression::StringLiteral(value) => Diag::some(value.ty(sema, ctx)),
        Expression::ConstantExpression(expr) => type_of(sema, ctx, expr),
        Expression::Add(e1, e2) => Diag::none_diag(Diagnosis::DivisionByZero),
        _ => todo!(),
    }
}

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
