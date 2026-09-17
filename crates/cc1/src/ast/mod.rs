pub mod arenas;
pub mod declaration;
pub mod display;
pub mod escape;
pub mod expression;
pub mod f80;
pub mod function;
pub mod literal;
pub mod name;
pub mod operator;
pub mod print;
pub mod statement;
pub mod tag;
pub mod type_specifier;
pub mod unit;
pub mod value;
pub mod visit;

pub use arenas::AstArenas;
pub use declaration::{
    DeclarationNode, Declarator, DeclaratorArena, DeclaratorId, DeclaratorNode, InitDeclaratorNode, Initializer,
    InitializerNode,
};
pub use expression::{Expression, ExpressionArena, ExpressionId, ExpressionNode, Type};
pub use f80::F80;
pub use function::{FunctionParameters, FunctionParametersNode, ParameterDeclaration};
pub use literal::{StringConstId, StringConstant, StringLiteralNode, StringPool};
pub use name::{Name, StringArena, StringId};
pub use operator::{BinaryOp, MemberOp, UnaryOp};
pub use statement::{
    CompoundStatementNode, ExpressionStatementNode, IterationStatement, IterationStatementNode, JumpStatement,
    JumpStatementNode, LabeledStatement, LabeledStatementNode, SelectionStatement, SelectionStatementNode, Statement,
    StatementArena, StatementNode,
};
pub use tag::{
    Enum, EnumArena, EnumId, Struct, StructArena, StructDeclaration, StructId, StructMemberDeclarator, Tag, Union,
    UnionArena, UnionId, Variant, VariantArena, VariantId,
};
pub use type_specifier::{DeclarationSpecifier, Qualifier, Storage, TypeSpecifier};
pub use unit::{ExternalDeclaration, ExternalDeclarationNode, FunctionDefinitionNode, TranslationUnitNode};
pub use value::{ConstValue, ConstValueNode, Fold};
pub use visit::Visitor;

pub trait Node {
    fn span(&self) -> libft::Span;
}
#[macro_export]
macro_rules! ast_node {
    (
        $vis:vis struct $name:ident {
            $($field_vis:vis $field:ident : $ty:ty),* $(,)?
        }
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        $vis struct $name {
            pub span: ::libft::Span,
            $($field_vis $field: $ty,)*
        }

        impl $name {
            pub fn new($($field: $ty,)* span: ::libft::Span) -> Self {
                Self {
                    $($field,)*
                    span,
                }
            }
        }


        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{}{} {}", ::libft::BLUE, stringify!($name), ::libft::RESET, self.span)
            }
        }


        impl $crate::ast::Node for $name {
            fn span(&self) -> ::libft::Span {
                self.span
            }
        }
    };
}
