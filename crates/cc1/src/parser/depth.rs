use libft::Span;

use crate::ast::{Expression, ExpressionArena, ExpressionId, ExpressionNode};

pub const MAX_EXPRESSION_DEPTH: usize = 32768;

pub fn deepest_expression(arena: &ExpressionArena, limit: usize) -> Option<Span> {
    let mut heights = vec![0usize; arena.len()];
    let mut stack: Vec<ExpressionId> = Vec::new();
    let mut offender = None;
    for index in 0..arena.len() {
        stack.push(ExpressionId::from(index));
        while let Some(&id) = stack.last() {
            let slot: usize = id.into();
            if heights[slot] != 0 {
                stack.pop();
                continue;
            }
            let children = children(arena.get(id));
            let pending = stack.len();
            stack.extend(children.iter().map(|child| child.id).filter(|child| heights[usize::from(*child)] == 0));
            if stack.len() != pending {
                continue;
            }
            stack.pop();
            let mut height = 1;
            for child in children {
                let child_height = heights[usize::from(child.id)];
                if child_height == limit && offender.is_none() {
                    offender = Some(child.span);
                }
                height = height.max(child_height + 1);
            }
            heights[slot] = height;
        }
    }
    offender
}

fn children(expression: &Expression) -> Vec<&ExpressionNode> {
    match expression {
        Expression::Identifier(_)
        | Expression::Constant(_)
        | Expression::StringLiteral(_)
        | Expression::SizeofType(_) => Vec::new(),
        Expression::ConstantExpression(e)
        | Expression::Unary(_, e)
        | Expression::Member(_, e, _)
        | Expression::SizeofExpr(e)
        | Expression::Cast(_, e)
        | Expression::Block(e) => vec![e],
        Expression::Binary(_, a, b) | Expression::Assign(_, a, b) | Expression::ArraySubscripting(a, b) => vec![a, b],
        Expression::Ternary(a, b, c) => vec![a, b, c],
        Expression::List(items) => items.iter().collect(),
        Expression::FunctionCall(function, args) => std::iter::once(function).chain(args).collect(),
    }
}
