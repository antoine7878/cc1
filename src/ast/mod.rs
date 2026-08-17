pub mod declaration;
pub mod expression;
pub mod function;
pub mod name;
pub mod print;
pub mod statement;
pub mod tag;
pub mod type_specifier;
pub mod unit;

pub use declaration::{DeclarationNode, Declarator, DeclaratorArena};
pub use declaration::{DeclaratorNode, InitDeclaratorNode, Initializer, InitializerNode};
pub use expression::{Expression, ExpressionArena, ExpressionId, ExpressionNode, Type};
pub use function::{FunctionParameters, FunctionParametersNode, ParameterDeclaration};
pub use name::{Name, StringArena, StringId};
pub use statement::{CompoundStatementNode, ExpressionStatementNode, IterationStatement};
pub use statement::{IterationStatementNode, JumpStatement, Labeled, LabeledStatementNode, StatementArena};
pub use statement::{JumpStatementNode, SelectionStatement, SelectionStatementNode, Statement, StatementNode};
pub use tag::{Enum, EnumArena, EnumId, Struct, StructArena, StructDeclaration, StructDeclarator, StructId};
pub use tag::{Tag, Union, UnionArena, UnionId, Variant, VariantArena, VariantId};
pub use type_specifier::{DeclarationSpecifier, Qualifier, Storage, TypeSpecifier};
pub use unit::{ExternalDeclaration, ExternalDeclarationNode, FunctionDefinitionNode, TranslationUnitNode};

use crate::parser::Span;

pub trait Node {
    fn span(&self) -> Span;
}

#[macro_export]
macro_rules! ast_node {
    (
        $vis:vis struct $name:ident {
            $($field_vis:vis $field:ident : $ty:ty),* $(,)?
        }
    ) => {
        #[derive(Clone, Debug, Eq, PartialEq, Hash)]
        $vis struct $name {
            pub span: Span,
            $($field_vis $field: $ty,)*
        }

        impl $name {
            pub fn new($($field: $ty,)* span: Span) -> Self {
                Self {
                    $($field,)*
                    span,
                }
            }
        }


        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{}{} {}", $crate::utils::BLUE, stringify!($name), $crate::utils::RESET, self.span)
            }
        }


        impl Node for $name {
            fn span(&self) -> Span {
                self.span
            }
        }
    };
}
