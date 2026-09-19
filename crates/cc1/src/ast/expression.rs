use libft::Span;

use crate::ast::{
    BinaryOp, ConstValueNode, DeclarationSpecifier, DeclaratorNode, MemberOp, Name, StringLiteralNode, UnaryOp,
};
use crate::{ast_node, define_arena};

define_arena!(Expression, ExpressionArena, ExpressionId);

ast_node! {
    pub struct ExpressionNode {
        pub id: ExpressionId,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Identifier(Name),
    Constant(ConstValueNode),
    StringLiteral(StringLiteralNode),
    ConstantExpression(ExpressionNode),
    Unary(UnaryOp, ExpressionNode),
    Binary(BinaryOp, ExpressionNode, ExpressionNode),
    Assign(Option<BinaryOp>, ExpressionNode, ExpressionNode),
    List(Vec<ExpressionNode>),
    Ternary(ExpressionNode, ExpressionNode, ExpressionNode),
    ArraySubscripting(ExpressionNode, ExpressionNode),
    FunctionCall(ExpressionNode, Vec<ExpressionNode>),
    Member(MemberOp, ExpressionNode, Name),
    SizeofExpr(ExpressionNode),
    SizeofType(TypeName),
    Cast(TypeName, ExpressionNode),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeName {
    pub specifiers: Vec<DeclarationSpecifier>,
    pub declarator: DeclaratorNode,
}

impl ExpressionArena {
    pub fn identifier(&mut self, name: Name, span: Span) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::Identifier(name)), span)
    }

    pub fn constant(&mut self, value: ConstValueNode, span: Span) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::Constant(value)), span)
    }

    pub fn string_literal(&mut self, literal: StringLiteralNode, span: Span) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::StringLiteral(literal)), span)
    }

    pub fn constant_expression(&mut self, inner: ExpressionNode, span: Span) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::ConstantExpression(inner)), span)
    }

    fn get_args(&self, args: Option<ExpressionNode>) -> Vec<ExpressionNode> {
        let Some(args) = args else {
            return vec![];
        };
        let Expression::List(v) = self.get(args.id) else {
            return vec![args];
        };
        v.clone()
    }

    pub fn function_call(
        &mut self,
        function: ExpressionNode,
        args: Option<ExpressionNode>,
        span: Span,
    ) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::FunctionCall(function, self.get_args(args))), span)
    }

    pub fn sizeof_expr(&mut self, operand: ExpressionNode, span: Span) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::SizeofExpr(operand)), span)
    }

    pub fn sizeof_type(&mut self, type_node: TypeName, span: Span) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::SizeofType(type_node)), span)
    }

    pub fn cast(&mut self, type_node: TypeName, operand: ExpressionNode, span: Span) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::Cast(type_node, operand)), span)
    }

    pub fn member(&mut self, object: ExpressionNode, op: MemberOp, identifier: Name, span: Span) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::Member(op, object, identifier)), span)
    }

    pub fn unary(&mut self, op: UnaryOp, operand: ExpressionNode, span: Span) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::Unary(op, operand)), span)
    }

    pub fn array_access(&mut self, lhs: ExpressionNode, index: ExpressionNode, span: Span) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::ArraySubscripting(lhs, index)), span)
    }

    pub fn push_comma(&mut self, mut lhs: ExpressionNode, rhs: ExpressionNode) -> ExpressionNode {
        let Expression::List(v) = self.get_mut(lhs.id) else {
            let span = Span::new(lhs.span.start, rhs.span.end);
            return ExpressionNode::new(self.alloc(Expression::List(vec![lhs, rhs])), span);
        };
        lhs.span.end = rhs.span.end;
        v.push(rhs);
        lhs
    }

    pub fn binary(&mut self, lhs: ExpressionNode, op: BinaryOp, rhs: ExpressionNode, span: Span) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::Binary(op, lhs, rhs)), span)
    }

    pub fn assign(
        &mut self,
        lhs: ExpressionNode,
        op: Option<BinaryOp>,
        rhs: ExpressionNode,
        span: Span,
    ) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::Assign(op, lhs, rhs)), span)
    }

    pub fn ternary(
        &mut self,
        cond: ExpressionNode,
        then: ExpressionNode,
        or: ExpressionNode,
        span: Span,
    ) -> ExpressionNode {
        ExpressionNode::new(self.alloc(Expression::Ternary(cond, then, or)), span)
    }
}
