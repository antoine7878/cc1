use crate::ast::{Name, StringArena};
use crate::ast_node;
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::{QualifiedType, Sema};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringLiteral {
    String(Name),
    WString(Name),
}

ast_node! {
    pub struct StringLiteralNode {
        pub string: StringLiteral
    }

}

impl StringLiteral {
    pub fn new_wide(name: Name) -> Self {
        StringLiteral::WString(name)
    }

    pub fn new(name: Name) -> Self {
        StringLiteral::String(name)
    }

    pub fn name(&self) -> Name {
        match self {
            StringLiteral::String(name) | StringLiteral::WString(name) => *name,
        }
    }

    pub fn is_wide(&self) -> bool {
        matches!(self, StringLiteral::WString(_))
    }
}

impl StringLiteralNode {
    pub fn name(&self) -> Name {
        self.string.name()
    }

    pub fn is_wide(&self) -> bool {
        self.string.is_wide()
    }

    pub fn ty(&self, sema: &mut Sema, ctx: &Context) -> QualifiedType {
        let (base_id, s) = match self.string {
            StringLiteral::String(s) => (sema.builtins.char, s),
            StringLiteral::WString(s) => (sema.builtins.int, s),
        };
        let len = s.id.resolve(ctx).len();
        let base = QualifiedType::plain(base_id);
        QualifiedType::plain(sema.types.array(base, Some(len + 1)))
    }
}

impl StringArena {
    pub fn literal(&mut self, text: &str, span: Span) -> StringLiteralNode {
        let is_wide = text.starts_with('L');
        let constructor = if is_wide { StringLiteral::new_wide } else { StringLiteral::new };
        let start = if is_wide { 2 } else { 1 };
        let name = self.add(text[start..text.len() - 1].to_string(), span);
        StringLiteralNode::new(constructor(name), span)
    }

    pub fn concat(&mut self, lhs: StringLiteralNode, rhs: StringLiteralNode, span: Span) -> StringLiteralNode {
        let is_wide = lhs.is_wide() || rhs.is_wide();
        let constructor = if is_wide { StringLiteral::new_wide } else { StringLiteral::new };
        let mut text = self.get(lhs.name().id).clone();
        text.push_str(self.get(rhs.name().id));
        let name = self.add(text, span);
        StringLiteralNode::new(constructor(name), span)
    }
}
