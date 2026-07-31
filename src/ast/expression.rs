use std::fmt::Display;

use crate::arena::{Arena, ArenaId};
use crate::ast::{Name, TypeNode};
use crate::define_arena;
use crate::parser::{Span, YYToken};

define_arena!(Expression, ExpressionArena, ExpressionId);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ExpressionNode {
    pub span: Span,
    pub id: ExpressionId,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Expression {
    // Values
    Identifier(Name),
    Constant(Name),
    StringLiteral(Name),

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
    Not(ExpressionNode),

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
    And(ExpressionNode, ExpressionNode),
    Or(ExpressionNode, ExpressionNode),
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
    List(ExpressionNode, ExpressionNode),

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
    SizeofType(TypeNode),

    // Cast
    Cast(TypeNode, ExpressionNode),
}

impl ExpressionArena {
    pub fn add(id: ExpressionId, span: Span) -> ExpressionNode {
        ExpressionNode { id, span }
    }

    pub fn identifier(&mut self, name: Name, span: Span) -> ExpressionNode {
        Self::add(self.alloc(Expression::Identifier(name)), span)
    }

    pub fn constant(&mut self, name: Name, span: Span) -> ExpressionNode {
        Self::add(self.alloc(Expression::Constant(name)), span)
    }

    pub fn string_literal(&mut self, name: Name, span: Span) -> ExpressionNode {
        Self::add(self.alloc(Expression::StringLiteral(name)), span)
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

    pub fn sizeof_type(&mut self, type_node: TypeNode, span: Span) -> ExpressionNode {
        Self::add(self.alloc(Expression::SizeofType(type_node)), span)
    }

    pub fn cast(&mut self, type_node: TypeNode, node_id: ExpressionNode, span: Span) -> ExpressionNode {
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
            YYToken::Char('!') => Expression::Not(operand),
            _ => unreachable!(),
        };
        Self::add(self.alloc(expr), span)
    }

    pub fn binary(&mut self, rhs: ExpressionNode, tok: YYToken, lhs: ExpressionNode, span: Span) -> ExpressionNode {
        let expr = match tok {
            YYToken::Char('+') => Expression::Add(rhs, lhs),
            YYToken::Char('-') => Expression::Sub(rhs, lhs),
            YYToken::Char('*') => Expression::Mul(rhs, lhs),
            YYToken::Char('/') => Expression::Div(rhs, lhs),
            YYToken::Char('%') => Expression::Mod(rhs, lhs),
            YYToken::RIGHT_OP => Expression::Right(rhs, lhs),
            YYToken::LEFT_OP => Expression::Left(rhs, lhs),
            YYToken::Char('>') => Expression::Greater(rhs, lhs),
            YYToken::Char('<') => Expression::Lower(rhs, lhs),
            YYToken::GE_OP => Expression::GreaterEq(rhs, lhs),
            YYToken::LE_OP => Expression::LowerEq(rhs, lhs),
            YYToken::EQ_OP => Expression::Eq(rhs, lhs),
            YYToken::NE_OP => Expression::Neq(rhs, lhs),
            YYToken::Char('&') => Expression::BitAnd(rhs, lhs),
            YYToken::Char('|') => Expression::BitOr(rhs, lhs),
            YYToken::Char('^') => Expression::BitXor(rhs, lhs),
            YYToken::AND_OP => Expression::And(rhs, lhs),
            YYToken::OR_OP => Expression::Or(rhs, lhs),
            YYToken::Char('=') => Expression::Assign(rhs, lhs),
            YYToken::MUL_ASSIGN => Expression::MulAssign(rhs, lhs),
            YYToken::DIV_ASSIGN => Expression::DivAssign(rhs, lhs),
            YYToken::MOD_ASSIGN => Expression::ModAssign(rhs, lhs),
            YYToken::ADD_ASSIGN => Expression::AddAssign(rhs, lhs),
            YYToken::SUB_ASSIGN => Expression::SubAssign(rhs, lhs),
            YYToken::LEFT_ASSIGN => Expression::LeftAssign(rhs, lhs),
            YYToken::RIGHT_ASSIGN => Expression::RightAssign(rhs, lhs),
            YYToken::AND_ASSIGN => Expression::AndAssign(rhs, lhs),
            YYToken::XOR_ASSIGN => Expression::XorAssign(rhs, lhs),
            YYToken::OR_ASSIGN => Expression::OrAssign(rhs, lhs),
            YYToken::Char(',') => Expression::List(rhs, lhs),
            YYToken::Char('[') => Expression::ArrayAcces(rhs, lhs),
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
            Expression::Constant(_) => "IntegerLiteral",
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
            Expression::Not(_) => "Not",
            Expression::Add(_, _) => "Add",
            Expression::Sub(_, _) => "Sub",
            Expression::Mul(_, _) => "Mul",
            Expression::Div(_, _) => "/",
            Expression::Mod(_, _) => "%",
            Expression::Right(_, _) => ">>",
            Expression::Left(_, _) => "<<",
            Expression::Greater(_, _) => ">",
            Expression::Lower(_, _) => "<",
            Expression::GreaterEq(_, _) => ">=",
            Expression::LowerEq(_, _) => "<=",
            Expression::Eq(_, _) => "==",
            Expression::Neq(_, _) => "!=",
            Expression::BitAnd(_, _) => "|",
            Expression::BitOr(_, _) => "&",
            Expression::BitXor(_, _) => "^",
            Expression::And(_, _) => "&&",
            Expression::Or(_, _) => "||",
            Expression::Assign(_, _) => "=",
            Expression::MulAssign(_, _) => "*=",
            Expression::DivAssign(_, _) => "/=",
            Expression::ModAssign(_, _) => "%=",
            Expression::AddAssign(_, _) => "+=",
            Expression::SubAssign(_, _) => "-=",
            Expression::LeftAssign(_, _) => "<<=",
            Expression::RightAssign(_, _) => ">>=",
            Expression::AndAssign(_, _) => "&=",
            Expression::XorAssign(_, _) => "|=",
            Expression::OrAssign(_, _) => "^=",
            Expression::List(_, _) => ",",
            Expression::ArrayAcces(_, _) => "Array acces",
            Expression::FunctionCall(_, _) => "Fn call",
            Expression::DotAcces(_, _) => "Dot access",
            Expression::PtrAcces(_, _) => "Ptr access",
            Expression::SizeofExpr(_) => "Sizeof",
            Expression::SizeofType(_) => "Sizeof",
            Expression::ConstantExpression(_) => "ConstantExpression",
            Expression::Ternary(_, _, _) => "Ternary",
            Expression::Cast(_, _) => "Cast",
        };
        write!(f, "{}", s)
    }
}
