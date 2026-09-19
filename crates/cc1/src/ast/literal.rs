use std::fmt;

use libft::Span;

use crate::ast::escape;
use crate::semantic::{Builtins, Diag, QualifiedType, ResolvedTypeId, Sema};
use crate::{ast_node, define_interner};

define_interner!(StringConstant, StringConstInterner, StringConstId);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StringConstant {
    pub units: Vec<u32>,
    pub is_wide: bool,
}

impl StringConstant {
    pub fn ty(&self, builtins: &Builtins) -> ResolvedTypeId {
        if self.is_wide { builtins.int } else { builtins.char }
    }
}

impl fmt::Display for StringConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.units {
            write!(f, "{}", char::from_u32(*c).unwrap())?;
        }
        Ok(())
    }
}

ast_node! {
    pub struct StringLiteralNode {
        pub id: StringConstId,
    }
}

impl StringLiteralNode {
    pub fn constant(&self) -> &'static StringConstant {
        self.id.resolve()
    }

    pub fn is_wide(&self) -> bool {
        self.constant().is_wide
    }

    pub fn len(&self) -> usize {
        self.constant().units.len()
    }

    pub fn is_empty(&self) -> bool {
        self.constant().units.is_empty()
    }

    pub fn ty(&self, sema: &mut Sema) -> QualifiedType {
        let constant = self.constant();
        let base = QualifiedType::plain(constant.ty(&sema.builtins));
        QualifiedType::plain(sema.types.array(base, Some(constant.units.len() + 1)))
    }
}

impl StringConstInterner {
    pub fn literal(&mut self, text: &str, span: Span) -> Diag<StringLiteralNode> {
        let is_wide = text.starts_with('L');
        let start = if is_wide { 2 } else { 1 };
        let Diag { res: units, diagnostic } = escape::decode(&text[start..text.len() - 1], is_wide);
        let id = self.intern(StringConstant { units, is_wide });
        Diag::new(StringLiteralNode::new(id, span), diagnostic)
    }

    pub fn concat(&mut self, lhs: StringLiteralNode, rhs: StringLiteralNode, span: Span) -> StringLiteralNode {
        let head = self.get(lhs.id).clone();
        let tail = self.get(rhs.id).clone();
        let mut units = head.units;
        units.extend(tail.units);
        let id = self.intern(StringConstant { units, is_wide: head.is_wide || tail.is_wide });
        StringLiteralNode::new(id, span)
    }
}
