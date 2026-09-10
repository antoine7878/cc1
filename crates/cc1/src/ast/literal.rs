use crate::ast::escape;
use crate::ast_node;
use crate::define_interner;
use crate::semantic::{Diag, QualifiedType, Sema};
use libft::Span;

define_interner!(StringConstant, StringPool, StringConstId);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StringConstant {
    pub units: Vec<u32>,
    pub is_wide: bool,
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

    pub fn ty(&self, sema: &mut Sema) -> QualifiedType {
        let constant = self.constant();
        let (is_wide, len) = (constant.is_wide, constant.units.len());
        let base_id = if is_wide { sema.builtins.int } else { sema.builtins.char };
        let base = QualifiedType::plain(base_id);
        QualifiedType::plain(sema.types.array(base, Some(len + 1)))
    }
}

impl StringPool {
    pub fn literal(&mut self, text: &str, span: Span) -> Diag<StringLiteralNode> {
        let is_wide = text.starts_with('L');
        let start = if is_wide { 2 } else { 1 };
        let Diag { res: units, diagnosis } = escape::decode(&text[start..text.len() - 1], is_wide);
        let id = self.alloc(StringConstant { units, is_wide });
        Diag::new(StringLiteralNode::new(id, span), diagnosis)
    }

    pub fn concat(&mut self, lhs: StringLiteralNode, rhs: StringLiteralNode, span: Span) -> StringLiteralNode {
        let head = self.get(lhs.id).clone();
        let tail = self.get(rhs.id).clone();
        let mut units = head.units;
        units.extend(tail.units);
        let id = self.alloc(StringConstant {
            units,
            is_wide: head.is_wide || tail.is_wide,
        });
        StringLiteralNode::new(id, span)
    }
}
