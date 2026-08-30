use crate::ast::{Name, StringArena};
use crate::ast_node;
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::{QualifiedType, Sema};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringLitral {
    String(Name),
    WString(Name),
}

ast_node! {
    pub struct StringLiteralNode {
        pub string: StringLitral
    }

}

impl StringLitral {
    pub fn new_wide(name: Name) -> Self {
        StringLitral::WString(name)
    }

    pub fn new(name: Name) -> Self {
        StringLitral::String(name)
    }

    pub fn name(&self) -> Name {
        match self {
            StringLitral::String(name) | StringLitral::WString(name) => *name,
        }
    }

    pub fn is_wide(&self) -> bool {
        matches!(self, StringLitral::WString(_))
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
            StringLitral::String(s) => (sema.builtins.char, s),
            StringLitral::WString(s) => (sema.builtins.int, s),
        };
        let len = s.id.resolve(ctx).len();
        let base = QualifiedType::new(base_id, false, false);
        QualifiedType::new(sema.types.array(base, Some(len + 1)), false, false)
    }
}

impl StringArena {
    /// 6.1.4 String literals
    /// The token still carries its optional `L` prefix and its delimiters.
    pub fn literal(&mut self, text: &str, span: Span) -> StringLiteralNode {
        let is_wide = text.starts_with('L');
        let constructor = if is_wide { StringLitral::new_wide } else { StringLitral::new };
        let start = if is_wide { 2 } else { 1 };
        let name = self.add(text[start..text.len() - 1].to_string(), span);
        StringLiteralNode::new(constructor(name), span)
    }

    /// 6.1.4 String literals
    /// Adjacent character string literal tokens are concatenated into a single literal.
    pub fn concat(&mut self, lhs: StringLiteralNode, rhs: StringLiteralNode, span: Span) -> StringLiteralNode {
        let is_wide = lhs.is_wide() || rhs.is_wide();
        let constructor = if is_wide { StringLitral::new_wide } else { StringLitral::new };
        let mut text = self.get(lhs.name().id).clone();
        text.push_str(self.get(rhs.name().id));
        let name = self.add(text, span);
        StringLiteralNode::new(constructor(name), span)
    }
}
