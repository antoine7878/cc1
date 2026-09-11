use std::fmt::Display;

use libft::{BLUE, RESET};

use crate::ast::{
    BinaryOp, DeclarationSpecifier, Declarator, Enum, Expression, FunctionParameters, Initializer, IterationStatement,
    JumpStatement, Labeled, MemberOp, Qualifier, SelectionStatement, Storage, Type, TypeSpecifier, UnaryOp, Variant,
};

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UnaryOp::PostInc => "post ++",
            UnaryOp::PostDec => "post --",
            UnaryOp::PreInc => "pre ++",
            UnaryOp::PreDec => "pre --",
            UnaryOp::Addr => "Addr",
            UnaryOp::Deref => "Deref",
            UnaryOp::Plus => "Plus",
            UnaryOp::Minus => "Minus",
            UnaryOp::BitNot => "BitNot",
            UnaryOp::LogicalNot => "LogicalNot",
        };
        write!(f, "{}", s)
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BinaryOp::Add => "Add",
            BinaryOp::Sub => "Sub",
            BinaryOp::Mul => "Mul",
            BinaryOp::Div => "Div",
            BinaryOp::Mod => "Mod",
            BinaryOp::Left => "Left",
            BinaryOp::Right => "Right",
            BinaryOp::Greater => "Greater",
            BinaryOp::Lower => "Lower",
            BinaryOp::GreaterEq => "GreaterEq",
            BinaryOp::LowerEq => "LowerEq",
            BinaryOp::Eq => "Eq",
            BinaryOp::Neq => "Neq",
            BinaryOp::BitAnd => "BitAnd",
            BinaryOp::BitOr => "BitOr",
            BinaryOp::BitXor => "BitXor",
            BinaryOp::LogicalAnd => "LogicalAnd",
            BinaryOp::LogicalOr => "LogicalOr",
        };
        write!(f, "{}", s)
    }
}

impl MemberOp {
    pub fn symbol(&self) -> &str {
        match self {
            MemberOp::Dot => ".",
            MemberOp::Arrow => "->",
        }
    }
}

impl Display for MemberOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MemberOp::Dot => "Dot access",
            MemberOp::Arrow => "Ptr access",
        };
        write!(f, "{}", s)
    }
}

impl Display for TypeSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TypeSpecifier::Void => "void",
            TypeSpecifier::Char => "char",
            TypeSpecifier::Short => "short",
            TypeSpecifier::Int => "int",
            TypeSpecifier::Long => "long",
            TypeSpecifier::Float => "float",
            TypeSpecifier::Double => "double",
            TypeSpecifier::Signed => "signed",
            TypeSpecifier::Unsigned => "unsigned",
            TypeSpecifier::Struct(_) => "struct",
            TypeSpecifier::Union(_) => "union",
            TypeSpecifier::Enum(_) => "enum",
            TypeSpecifier::TypedefName(_) => "typedef",
        };
        write!(f, "{}", s)
    }
}

impl Display for Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Qualifier::Const => "const",
            Qualifier::Volatile => "volatile",
        };
        write!(f, "{}", s)
    }
}

impl Display for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Storage::Auto => "auto",
            Storage::Static => "static",
            Storage::Extern => "extern",
            Storage::Typedef => "typedef",
            Storage::Register => "register",
        };
        write!(f, "{}", s)
    }
}

impl Display for DeclarationSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeclarationSpecifier::Type(t) => write!(f, "{}", t),
            DeclarationSpecifier::Qualifier(t) => write!(f, "{}", t),
            DeclarationSpecifier::Storage(t) => write!(f, "{}", t),
        }
    }
}

impl Display for Declarator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Declarator::Ident(_) => "Ident",
            Declarator::Abstract => "Abstract",
            Declarator::Pointer { .. } => "Pointer",
            Declarator::Array { .. } => "Array",
            Declarator::Function { .. } => "Function",
        };
        write!(f, "{}", s)
    }
}

impl Display for Initializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Initializer::Single(_) => "Single",
            Initializer::List(_) => "List",
        };
        write!(f, "{}", s)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type")
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Constant(_) => write!(f, "NumberLiteral"),
            Expression::Identifier(_) => write!(f, "Identifier"),
            Expression::StringLiteral(_) => write!(f, "StringLiteral"),
            Expression::ConstantExpression(_) => write!(f, "ConstantExpression"),
            Expression::Unary(op, _) => write!(f, "{}", op),
            Expression::Binary(op, _, _) => write!(f, "{}", op),
            Expression::Assign(None, _, _) => write!(f, "Assign"),
            Expression::Assign(Some(op), _, _) => write!(f, "{}Assign", op),
            Expression::List(_) => write!(f, "List"),
            Expression::Ternary(_, _, _) => write!(f, "Ternary"),
            Expression::ArraySubscripting(_, _) => write!(f, "Array access"),
            Expression::FunctionCall(_, _) => write!(f, "Fn call"),
            Expression::Member(op, _, _) => write!(f, "{}", op),
            Expression::SizeofExpr(_) | Expression::SizeofType(_) => write!(f, "Sizeof"),
            Expression::Cast(_, _) => write!(f, "Cast"),
        }
    }
}

impl Display for FunctionParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FunctionParameters::Empty => "Empty",
            FunctionParameters::OldStyle(_) => "OldStyle",
            FunctionParameters::ParameterTypeList(_) => "Parameters",
            FunctionParameters::Variadic(_) => "Variadic",
        };
        write!(f, "{}", s)
    }
}

impl Display for Labeled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Labeled::Identifier(..) => "Identifier",
            Labeled::Case(..) => "Case",
            Labeled::Default(_) => "Default",
        };
        write!(f, "{}", s)
    }
}

impl Display for SelectionStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SelectionStatement::If(..) => "If",
            SelectionStatement::Switch(..) => "Switch",
        };
        write!(f, "{}", s)
    }
}

impl Display for IterationStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            IterationStatement::While(..) => "While",
            IterationStatement::Do(..) => "Do",
            IterationStatement::For(..) => "For",
        };
        write!(f, "{}", s)
    }
}

impl Display for JumpStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            JumpStatement::Goto(_) => "Goto",
            JumpStatement::Continue => "Continue",
            JumpStatement::Break => "Break",
            JumpStatement::Return(_) => "Return",
        };
        write!(f, "{}", s)
    }
}

impl Display for Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{BLUE}Enum{RESET} {}", self.span)
    }
}

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{BLUE}Variant{RESET} {}", self.span)
    }
}
