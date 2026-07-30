use crate::arena::{Arena, ArenaId};
use crate::parser::YYToken;
use crate::symbol::NameId;
use crate::types::TypeId;

pub type NodeId = ArenaId<Node>;
pub type NodeArena = Arena<NodeId, Node>;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Expression {
    // Values
    Identifier(NameId),
    Constant(NameId),
    StringLiteral(NameId),

    // Cst
    ConstantExpression(NodeId),

    // Unary
    PostInc(NodeId),
    PostDec(NodeId),
    PreInc(NodeId),
    PreDec(NodeId),
    Addr(NodeId),
    Deref(NodeId),
    Plus(NodeId),
    Minus(NodeId),
    BitNot(NodeId),
    Not(NodeId),

    // Binary
    Add(NodeId, NodeId),
    Sub(NodeId, NodeId),
    Mul(NodeId, NodeId),
    Div(NodeId, NodeId),
    Mod(NodeId, NodeId),
    Right(NodeId, NodeId),
    Left(NodeId, NodeId),
    Greater(NodeId, NodeId),
    Lower(NodeId, NodeId),
    GreaterEq(NodeId, NodeId),
    LowerEq(NodeId, NodeId),
    Eq(NodeId, NodeId),
    Neq(NodeId, NodeId),
    BitAnd(NodeId, NodeId),
    BitOr(NodeId, NodeId),
    BitXor(NodeId, NodeId),
    And(NodeId, NodeId),
    Or(NodeId, NodeId),
    Assign(NodeId, NodeId),
    MulAssign(NodeId, NodeId),
    DivAssign(NodeId, NodeId),
    ModAssign(NodeId, NodeId),
    AddAssign(NodeId, NodeId),
    SubAssign(NodeId, NodeId),
    LeftAssign(NodeId, NodeId),
    RightAssign(NodeId, NodeId),
    AndAssign(NodeId, NodeId),
    XorAssign(NodeId, NodeId),
    OrAssign(NodeId, NodeId),
    List(NodeId, NodeId),

    // Ternary
    Ternary(NodeId, NodeId, NodeId),

    // Array
    ArrayAcces(NodeId, NodeId),

    // Function
    FunctionCall(NodeId, Option<NodeId>),

    // Tag acces
    DotAcces(NodeId, NameId),
    PtrAcces(NodeId, NameId),

    // sizeof
    SizeofExpr(NodeId),
    SizeofType(TypeId),

    // Cast
    Cast(TypeId, NodeId),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Declaration;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Statement;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TranslationUnit;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Node {
    Expression(Expression),
    Declaration(Declaration),
    TranslationUnit(TranslationUnit),
}

impl NodeArena {
    pub fn identifier(&mut self, name_id: NameId) -> NodeId {
        self.alloc(Node::Expression(Expression::Identifier(name_id)))
    }

    pub fn constant(&mut self, name_id: NameId) -> NodeId {
        self.alloc(Node::Expression(Expression::Constant(name_id)))
    }

    pub fn string_literal(&mut self, name_id: NameId) -> NodeId {
        self.alloc(Node::Expression(Expression::StringLiteral(name_id)))
    }

    pub fn constant_expression(&mut self, node_id: NodeId) -> NodeId {
        self.alloc(Node::Expression(Expression::ConstantExpression(node_id)))
    }

    pub fn function_call(&mut self, function: NodeId, args: Option<NodeId>) -> NodeId {
        self.alloc(Node::Expression(Expression::FunctionCall(function, args)))
    }

    pub fn sizeof_expr(&mut self, node_id: NodeId) -> NodeId {
        self.alloc(Node::Expression(Expression::SizeofExpr(node_id)))
    }

    pub fn sizeof_type(&mut self, type_id: TypeId) -> NodeId {
        self.alloc(Node::Expression(Expression::SizeofType(type_id)))
    }

    pub fn cast(&mut self, type_id: TypeId, node_id: NodeId) -> NodeId {
        self.alloc(Node::Expression(Expression::Cast(type_id, node_id)))
    }

    pub fn access(&mut self, tag: NodeId, tok: YYToken, identifier: NameId) -> NodeId {
        let expr = match tok {
            YYToken::Char('.') => Expression::DotAcces(tag, identifier),
            YYToken::PTR_OP => Expression::PtrAcces(tag, identifier),
            _ => unreachable!(),
        };
        self.alloc(Node::Expression(expr))
    }

    pub fn unary(&mut self, operator: YYToken, operand: NodeId) -> NodeId {
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
        self.alloc(Node::Expression(expr))
    }

    pub fn binary(&mut self, rhs: NodeId, tok: YYToken, lhs: NodeId) -> NodeId {
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
        self.alloc(Node::Expression(expr))
    }

    pub fn ternary(&mut self, condition: NodeId, then: NodeId, or: NodeId) -> NodeId {
        self.alloc(Node::Expression(Expression::Ternary(condition, then, or)))
    }
}
