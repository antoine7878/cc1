use std::fmt::Display;

use crate::ast::{DeclarationSpecifier, DeclaratorNode, Name, StringLiteralNode, ValueNode};
use crate::parser::{Span, YYToken};
use crate::{ast_node, define_arena};

define_arena!(
    Expression,
    ExpressionArena,
    ExpressionId,
    crate::ast::AstArenas,
    expressions
);

ast_node! {
    pub struct ExpressionNode {
        pub id: ExpressionId,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    // Values
    Identifier(Name),
    Constant(ValueNode),
    StringLiteral(StringLiteralNode),

    // Cst
    ConstantExpression(ExpressionNode),

    // Unary
    PostInc(ExpressionNode),
    PostDec(ExpressionNode),
    PreInc(ExpressionNode),
    PreDec(ExpressionNode),
    Addr(ExpressionNode),
    Deref(ExpressionNode),
    Plus(ExpressionNode),
    Minus(ExpressionNode),
    BitNot(ExpressionNode),
    LogicalNot(ExpressionNode),

    // Binary
    Add(ExpressionNode, ExpressionNode),
    Sub(ExpressionNode, ExpressionNode),
    Mul(ExpressionNode, ExpressionNode),
    Div(ExpressionNode, ExpressionNode),
    Mod(ExpressionNode, ExpressionNode),
    Right(ExpressionNode, ExpressionNode),
    Left(ExpressionNode, ExpressionNode),
    Greater(ExpressionNode, ExpressionNode),
    Lower(ExpressionNode, ExpressionNode),
    GreaterEq(ExpressionNode, ExpressionNode),
    LowerEq(ExpressionNode, ExpressionNode),
    Eq(ExpressionNode, ExpressionNode),
    Neq(ExpressionNode, ExpressionNode),
    BitAnd(ExpressionNode, ExpressionNode),
    BitOr(ExpressionNode, ExpressionNode),
    BitXor(ExpressionNode, ExpressionNode),
    LogicalAnd(ExpressionNode, ExpressionNode),
    LogicalOr(ExpressionNode, ExpressionNode),
    Assign(ExpressionNode, ExpressionNode),
    MulAssign(ExpressionNode, ExpressionNode),
    DivAssign(ExpressionNode, ExpressionNode),
    ModAssign(ExpressionNode, ExpressionNode),
    AddAssign(ExpressionNode, ExpressionNode),
    SubAssign(ExpressionNode, ExpressionNode),
    LeftAssign(ExpressionNode, ExpressionNode),
    RightAssign(ExpressionNode, ExpressionNode),
    AndAssign(ExpressionNode, ExpressionNode),
    XorAssign(ExpressionNode, ExpressionNode),
    OrAssign(ExpressionNode, ExpressionNode),
    List(Vec<ExpressionNode>),

    // Ternary
    Ternary(ExpressionNode, ExpressionNode, ExpressionNode),

    // Array
    ArrayAcces(ExpressionNode, ExpressionNode),

    // Function
    FunctionCall(ExpressionNode, Option<ExpressionNode>),

    // Tag acces
    DotAcces(ExpressionNode, Name),
    PtrAcces(ExpressionNode, Name),

    // sizeof
    SizeofExpr(ExpressionNode),
    SizeofType(Type),

    // Cast
    Cast(Type, ExpressionNode),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Type {
    pub specifiers: Vec<DeclarationSpecifier>,
    pub declarator: DeclaratorNode,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type")
    }
}

impl ExpressionArena {
    pub fn add(id: ExpressionId, span: Span) -> ExpressionNode {
        ExpressionNode::new(id, span)
    }

    pub fn identifier(&mut self, name: Name, span: Span) -> ExpressionNode {
        Self::add(self.alloc(Expression::Identifier(name)), span)
    }

    pub fn constant(&mut self, value: ValueNode, span: Span) -> ExpressionNode {
        Self::add(self.alloc(Expression::Constant(value)), span)
    }

    pub fn string_literal(&mut self, literal: StringLiteralNode, span: Span) -> ExpressionNode {
        Self::add(self.alloc(Expression::StringLiteral(literal)), span)
    }

    pub fn constant_expression(&mut self, node_id: ExpressionNode, span: Span) -> ExpressionNode {
        Self::add(self.alloc(Expression::ConstantExpression(node_id)), span)
    }

    pub fn function_call(
        &mut self,
        function: ExpressionNode,
        args: Option<ExpressionNode>,
        span: Span,
    ) -> ExpressionNode {
        Self::add(self.alloc(Expression::FunctionCall(function, args)), span)
    }

    pub fn sizeof_expr(&mut self, node_node: ExpressionNode, span: Span) -> ExpressionNode {
        Self::add(self.alloc(Expression::SizeofExpr(node_node)), span)
    }

    pub fn sizeof_type(&mut self, type_node: Type, span: Span) -> ExpressionNode {
        Self::add(self.alloc(Expression::SizeofType(type_node)), span)
    }

    pub fn cast(&mut self, type_node: Type, node_id: ExpressionNode, span: Span) -> ExpressionNode {
        Self::add(self.alloc(Expression::Cast(type_node, node_id)), span)
    }

    pub fn access(&mut self, tag: ExpressionNode, tok: YYToken, identifier: Name, span: Span) -> ExpressionNode {
        let expr = match tok {
            YYToken::Char('.') => Expression::DotAcces(tag, identifier),
            YYToken::PTR_OP => Expression::PtrAcces(tag, identifier),
            _ => unreachable!(),
        };
        Self::add(self.alloc(expr), span)
    }

    pub fn unary(&mut self, operator: YYToken, operand: ExpressionNode, span: Span) -> ExpressionNode {
        let expr = match operator {
            YYToken::POST_INC_OP => Expression::PostInc(operand),
            YYToken::POST_DEC_OP => Expression::PostDec(operand),
            YYToken::INC_OP => Expression::PreInc(operand),
            YYToken::DEC_OP => Expression::PreDec(operand),
            YYToken::Char('&') => Expression::Addr(operand),
            YYToken::Char('*') => Expression::Deref(operand),
            YYToken::Char('+') => Expression::Plus(operand),
            YYToken::Char('-') => Expression::Minus(operand),
            YYToken::Char('~') => Expression::BitNot(operand),
            YYToken::Char('!') => Expression::LogicalNot(operand),
            _ => unreachable!(),
        };
        Self::add(self.alloc(expr), span)
    }

    pub fn add_list(&mut self, mut lhs: ExpressionNode, rhs: ExpressionNode) -> ExpressionNode {
        let Expression::List(v) = self.get_mut(lhs.id) else {
            let span = Span::new(lhs.span.start, rhs.span.end);
            return Self::add(self.alloc(Expression::List(vec![lhs, rhs])), span);
        };
        lhs.span.end = rhs.span.end;
        v.push(rhs);
        lhs
    }

    pub fn binary(&mut self, lhs: ExpressionNode, tok: YYToken, rhs: ExpressionNode, span: Span) -> ExpressionNode {
        let expr = match tok {
            YYToken::Char('+') => Expression::Add(lhs, rhs),
            YYToken::Char('-') => Expression::Sub(lhs, rhs),
            YYToken::Char('*') => Expression::Mul(lhs, rhs),
            YYToken::Char('/') => Expression::Div(lhs, rhs),
            YYToken::Char('%') => Expression::Mod(lhs, rhs),
            YYToken::RIGHT_OP => Expression::Right(lhs, rhs),
            YYToken::LEFT_OP => Expression::Left(lhs, rhs),
            YYToken::Char('>') => Expression::Greater(lhs, rhs),
            YYToken::Char('<') => Expression::Lower(lhs, rhs),
            YYToken::GE_OP => Expression::GreaterEq(lhs, rhs),
            YYToken::LE_OP => Expression::LowerEq(lhs, rhs),
            YYToken::EQ_OP => Expression::Eq(lhs, rhs),
            YYToken::NE_OP => Expression::Neq(lhs, rhs),
            YYToken::Char('&') => Expression::BitAnd(lhs, rhs),
            YYToken::Char('|') => Expression::BitOr(lhs, rhs),
            YYToken::Char('^') => Expression::BitXor(lhs, rhs),
            YYToken::AND_OP => Expression::LogicalAnd(lhs, rhs),
            YYToken::OR_OP => Expression::LogicalOr(lhs, rhs),
            YYToken::Char('=') => Expression::Assign(lhs, rhs),
            YYToken::MUL_ASSIGN => Expression::MulAssign(lhs, rhs),
            YYToken::DIV_ASSIGN => Expression::DivAssign(lhs, rhs),
            YYToken::MOD_ASSIGN => Expression::ModAssign(lhs, rhs),
            YYToken::ADD_ASSIGN => Expression::AddAssign(lhs, rhs),
            YYToken::SUB_ASSIGN => Expression::SubAssign(lhs, rhs),
            YYToken::LEFT_ASSIGN => Expression::LeftAssign(lhs, rhs),
            YYToken::RIGHT_ASSIGN => Expression::RightAssign(lhs, rhs),
            YYToken::AND_ASSIGN => Expression::AndAssign(lhs, rhs),
            YYToken::XOR_ASSIGN => Expression::XorAssign(lhs, rhs),
            YYToken::OR_ASSIGN => Expression::OrAssign(lhs, rhs),
            // YYToken::Char(',') => Expression::List(lhs, rhs),
            YYToken::Char('[') => Expression::ArrayAcces(lhs, rhs),
            _ => unreachable!(),
        };
        Self::add(self.alloc(expr), span)
    }

    pub fn ternary(
        &mut self,
        cond: ExpressionNode,
        then: ExpressionNode,
        or: ExpressionNode,
        span: Span,
    ) -> ExpressionNode {
        Self::add(self.alloc(Expression::Ternary(cond, then, or)), span)
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Expression::Constant(_) => "NumberLiteral",
            Expression::Identifier(_) => "Identifier",
            Expression::StringLiteral(_) => "StringLiteral",
            Expression::PostInc(_) => "post ++",
            Expression::PostDec(_) => "post --",
            Expression::PreInc(_) => "pre ++",
            Expression::PreDec(_) => "pre --",
            Expression::Addr(_) => "Addr",
            Expression::Deref(_) => "Deref",
            Expression::Plus(_) => "Plus",
            Expression::Minus(_) => "Minus",
            Expression::BitNot(_) => "BitNot",
            Expression::LogicalNot(_) => "LogicalNot",
            Expression::Add(_, _) => "Add",
            Expression::Sub(_, _) => "Sub",
            Expression::Mul(_, _) => "Mul",
            Expression::Div(_, _) => "Div",
            Expression::Mod(_, _) => "Mod",
            Expression::Right(_, _) => "Right",
            Expression::Left(_, _) => "Left",
            Expression::Greater(_, _) => "Greater",
            Expression::Lower(_, _) => "Lower",
            Expression::GreaterEq(_, _) => "GreaterEq",
            Expression::LowerEq(_, _) => "LowerEq",
            Expression::Eq(_, _) => "Eq",
            Expression::Neq(_, _) => "Neq",
            Expression::BitAnd(_, _) => "BitAnd",
            Expression::BitOr(_, _) => "BitOr",
            Expression::BitXor(_, _) => "BitXor",
            Expression::LogicalAnd(_, _) => "LogicalAnd",
            Expression::LogicalOr(_, _) => "LogicalOr",
            Expression::Assign(_, _) => "Assign",
            Expression::MulAssign(_, _) => "MulAssign",
            Expression::DivAssign(_, _) => "DivAssign",
            Expression::ModAssign(_, _) => "ModAssign",
            Expression::AddAssign(_, _) => "AddAssign",
            Expression::SubAssign(_, _) => "SubAssign",
            Expression::LeftAssign(_, _) => "LeftAssign",
            Expression::RightAssign(_, _) => "RightAssign",
            Expression::AndAssign(_, _) => "AndAssign",
            Expression::OrAssign(_, _) => "OrAssign",
            Expression::XorAssign(_, _) => "XorAssign",
            Expression::List(_) => "List",
            Expression::ArrayAcces(_, _) => "Array access",
            Expression::FunctionCall(_, _) => "Fn call",
            Expression::DotAcces(_, _) => "Dot access",
            Expression::PtrAcces(_, _) => "Ptr access",
            Expression::SizeofExpr(_) => "SizeofExpr",
            Expression::SizeofType(_) => "SizeofType",
            Expression::ConstantExpression(_) => "ConstantExpression",
            Expression::Ternary(_, _, _) => "Ternary",
            Expression::Cast(_, _) => "Cast",
        };
        write!(f, "{}", s)
    }
}
