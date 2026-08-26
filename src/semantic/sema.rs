use std::collections::HashMap;

use crate::ast::visit::walk_expression;
use crate::ast::{Expression, ExpressionId, ExpressionNode, Name, Tag, Value, Visitor};
use crate::parser::{Context, Span};
use crate::semantic::{
    Diag, DiagCollector, Diagnosis, DiagnosisNode, FunctionDefArena, QualifiedType, ResolvedTypeArena, ResolvedTypeId,
    ScopeKind, Scopes, Symbol, SymbolArena, SymbolId, SymbolKind, TagDefArena, TagDefId,
};
use crate::target::{Layout, Target};

#[derive(Default, Debug)]
pub struct Sema {
    pub scopes: Scopes,
    pub diagnosis: Vec<DiagnosisNode>,
    pub symbols: SymbolArena,
    pub types: ResolvedTypeArena,
    pub tags: TagDefArena,
    pub functions: FunctionDefArena,
    pub bindings: HashMap<ExpressionId, Option<SymbolId>>,
    pub const_values: HashMap<ExpressionId, Option<Value>>,
    pub layouts: HashMap<ResolvedTypeId, Layout>,
    pub target: Target,
}

impl DiagCollector for Sema {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        &mut self.diagnosis
    }
}

impl Sema {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            ..Self::default()
        }
    }

    pub fn into_context(self, mut ctx: Context) -> Context {
        let Sema {
            scopes: _,
            diagnosis,
            symbols,
            types,
            tags,
            functions,
            bindings,
            const_values,
            layouts: _,
            target: _,
        } = self;
        ctx.arenas.symbols = symbols;
        ctx.arenas.resolved_type = types;
        ctx.arenas.tags = tags;
        ctx.arenas.functions = functions;
        ctx.bindings = bindings;
        ctx.const_values = const_values;
        ctx.diagnosis.extend(diagnosis);
        ctx
    }

    // ----- Resolution --------------------

    pub fn resolve_typedef(
        &mut self,
        name: Name,
        is_const: bool,
        is_volatile: bool,
        span: &Span,
    ) -> Option<QualifiedType> {
        let sym_id = self.scopes.lookup_ordinary(name.id)?;
        let sym = self.symbols.get(sym_id);
        if sym.kind != SymbolKind::Typedef {
            return self.add_diag(Diag::none_diag(Diagnosis::UndeclaredIdentifier(name)), span);
        }
        let base = sym.ty?;
        if (is_const && base.is_const) || (is_volatile && base.is_volatile) {
            self.add_diag(Diag::only_diag(Diagnosis::DuplicateTypeQualifers), span)
        }
        Some(QualifiedType::new(
            base.ty,
            base.is_const || is_const,
            base.is_volatile || is_volatile,
        ))
    }

    pub fn declare_tag(&mut self, kind: Tag, name: Option<Name>, is_definition: bool, span: &Span) -> TagDefId {
        let Some(name) = name else { return self.tags.declare(kind, None) };

        if let Some(id) = self.scopes.lookup_tag(name.id, is_definition) {
            let def = self.tags.get(id);
            if def.kind != kind || (is_definition && def.is_complete) {
                self.add_diag(Diag::only_diag(Diagnosis::DuplicateDeclaration(def.kind(), name)), span)
            }
            return id;
        }

        let id = self.tags.declare(kind, Some(name));
        self.scopes.insert_tag(name.id, id);
        id
    }

    pub fn declare(&mut self, sym: Symbol, span: &Span) -> SymbolId {
        let (name, kind) = (sym.name, sym.kind);
        let sym_id = match self.dedup(&sym, span) {
            Some(id) => id,
            None => self.symbols.alloc(sym),
        };
        self.scopes.insert(kind, name.id, sym_id);
        sym_id
    }

    fn dedup(&mut self, sym: &Symbol, span: &Span) -> Option<SymbolId> {
        let old_id = self.scopes.current(sym.kind, sym.name.id)?;
        let old_symbol = self.symbols.get(old_id);
        if self.scopes.kind() == ScopeKind::File
            && old_symbol.is_compatible(sym)
            && !(sym.is_init && old_symbol.is_init)
        {
            return Some(old_id);
        }
        self.add_diag(
            Diag::some_diag(old_id, Diagnosis::DuplicateDeclaration(sym.kind, sym.name)),
            span,
        )
    }

    pub fn add_label_symbol(&mut self, name: Name, span: &Span, is_init: bool) {
        if let Some(old) = self.scopes.lookup_label(name.id) {
            let old_init = self.symbols.get(old).is_init;
            if !old_init && is_init {
                self.symbols.get_mut(old).is_init = true;
                return;
            }
            if !(is_init && old_init) {
                return;
            }
            return self.add_diag(
                Diag::only_diag(Diagnosis::DuplicateDeclaration(SymbolKind::Label, name)),
                span,
            );
        };
        let sym_id = self.symbols.add(name, None, None, SymbolKind::Label, is_init);
        self.scopes.insert(SymbolKind::Label, name.id, sym_id);
    }
}

impl Visitor for Sema {
    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        if self.bindings.contains_key(&node.id) {
            return;
        }
        walk_expression(self, ctx, node);
        if let Expression::Identifier(name) = node.id.resolve(ctx) {
            let sym = self.scopes.lookup_ordinary(name.id);
            if sym.is_none() {
                self.add_diag(Diag::only_diag(Diagnosis::UndeclaredIdentifier(*name)), &node.span);
            }
            self.bindings.insert(node.id, sym);
        }
    }
}
