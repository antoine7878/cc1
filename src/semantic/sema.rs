use std::collections::HashMap;
use std::mem;

use crate::arena::{ResolveMutWith, ResolveWith};
use crate::ast::{DeclaratorId, ExpressionId, Name, StringId, Tag, Value};
use crate::parser::Span;
use crate::semantic::{
    Builtins, Definition, Diag, DiagCollector, Diagnosis, DiagnosisNode, FunctionDefArena, InitializerArena, Linkage,
    QualifiedType, ResolvedExpression, ResolvedTypeArena, ResolvedTypeId, ScopeKind, Scopes, Symbol, SymbolArena,
    SymbolId, SymbolKind, TagDefArena, TagDefId,
};
use crate::target::{Layout, Target};

#[derive(Debug, Clone)]
pub struct External {
    pub symbol: SymbolId,
    pub defined: Option<Span>,
    pub tentative: Option<Span>,
}

#[derive(Debug, Default)]
pub struct ExprFacts {
    pub resolved: Option<ResolvedExpression>,
    pub binding: Option<SymbolId>,
    pub constant: Option<Value>,
    pub resolved_seen: bool,
    pub binding_seen: bool,
    pub constant_seen: bool,
}

#[derive(Debug)]
pub struct Sema {
    pub scopes: Scopes,
    pub diagnosis: Vec<DiagnosisNode>,

    pub symbols: SymbolArena,
    pub types: ResolvedTypeArena,
    pub builtins: Builtins,
    pub tags: TagDefArena,
    pub functions: FunctionDefArena,
    pub inits: InitializerArena,

    exprs: Vec<ExprFacts>,
    pub declarations: HashMap<DeclaratorId, SymbolId>,
    pub externals: HashMap<StringId, External>,

    pub layouts: HashMap<ResolvedTypeId, Layout>,
    pub target: Target,
}

impl Default for Sema {
    fn default() -> Self {
        let mut types = ResolvedTypeArena::default();
        let target = Target::default();
        let builtins = Builtins::new(&mut types, &target);
        Self {
            scopes: Scopes::default(),
            diagnosis: Vec::new(),
            symbols: SymbolArena::default(),
            types,
            builtins,
            tags: TagDefArena::default(),
            functions: FunctionDefArena::default(),
            inits: InitializerArena::default(),

            exprs: Vec::new(),
            declarations: HashMap::default(),
            externals: HashMap::default(),

            layouts: HashMap::default(),
            target,
        }
    }
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

    // ----- Expression table ---------

    pub fn size_expr_facts(&mut self, len: usize) {
        if self.exprs.len() < len {
            self.exprs.resize_with(len, ExprFacts::default);
        }
    }

    pub fn expr_facts(&self) -> &[ExprFacts] {
        &self.exprs
    }

    fn facts(&self, id: ExpressionId) -> Option<&ExprFacts> {
        self.exprs.get(usize::from(id))
    }

    fn facts_mut(&mut self, id: ExpressionId) -> &mut ExprFacts {
        &mut self.exprs[usize::from(id)]
    }

    pub fn expr_seen(&self, id: ExpressionId) -> bool {
        self.facts(id).is_some_and(|f| f.resolved_seen)
    }

    pub fn expr_poisoned(&self, id: ExpressionId) -> bool {
        self.facts(id).is_some_and(|f| f.resolved_seen && f.resolved.is_none())
    }

    pub fn expr_resolved(&self, id: ExpressionId) -> Option<&ResolvedExpression> {
        self.facts(id)?.resolved.as_ref()
    }

    pub fn set_expr_resolved(&mut self, id: ExpressionId, resolved: Option<ResolvedExpression>) {
        let f = self.facts_mut(id);
        f.resolved = resolved;
        f.resolved_seen = true;
    }

    pub fn take_expr_resolved(&mut self, id: ExpressionId) -> Option<ResolvedExpression> {
        mem::take(&mut self.facts_mut(id).resolved)
    }

    pub fn binding(&self, id: ExpressionId) -> Option<SymbolId> {
        self.facts(id)?.binding
    }

    pub fn binding_seen(&self, id: ExpressionId) -> bool {
        self.facts(id).is_some_and(|f| f.binding_seen)
    }

    pub fn set_binding(&mut self, id: ExpressionId, binding: Option<SymbolId>) {
        let f = self.facts_mut(id);
        f.binding = binding;
        f.binding_seen = true;
    }

    pub fn constant_cached(&self, id: ExpressionId) -> Option<Option<Value>> {
        let f = self.facts(id)?;
        f.constant_seen.then_some(f.constant)
    }

    pub fn set_constant(&mut self, id: ExpressionId, constant: Option<Value>) {
        let f = self.facts_mut(id);
        f.constant = constant;
        f.constant_seen = true;
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
        let sym = sym_id.resolve(self);
        if sym.kind != SymbolKind::Typedef {
            return self.add_diag(Diag::err(None, Diagnosis::UndeclaredIdentifier(name)), span);
        }
        let base = sym.ty?;
        if (is_const && base.is_const) || (is_volatile && base.is_volatile) {
            self.add_diag(Diag::err((), Diagnosis::DuplicateTypeQualifiers), span)
        }
        Some(QualifiedType::new(
            base.id,
            base.is_const || is_const,
            base.is_volatile || is_volatile,
        ))
    }

    pub fn declare_tag(&mut self, kind: Tag, name: Option<Name>, is_definition: bool, span: &Span) -> TagDefId {
        let Some(name) = name else { return self.tags.declare(kind, None) };

        if let Some(id) = self.scopes.lookup_tag(name.id, is_definition) {
            let def = id.resolve(self);
            if def.kind != kind || (is_definition && def.is_complete) {
                self.add_diag(Diag::err((), Diagnosis::DuplicateDeclaration(def.kind(), name)), span)
            }
            return id;
        }

        let id = self.tags.declare(kind, Some(name));
        self.scopes.insert_tag(name.id, id);
        id
    }

    pub fn linkage_of_name(&self, name: StringId) -> Option<Linkage> {
        let id = self.externals.get(&name)?.symbol;
        Some(id.resolve(self).linkage)
    }

    pub fn declare(&mut self, sym: Symbol, span: &Span) -> SymbolId {
        let (name, kind) = (sym.name, sym.kind);
        let lexical = self.dedup(&sym, span);
        let sym_id = if sym.linkage != Linkage::None {
            self.register_external(sym, span, lexical)
        } else {
            lexical.unwrap_or_else(|| self.symbols.alloc(sym))
        };
        self.scopes.insert(kind, name.id, sym_id);
        sym_id
    }

    fn register_external(&mut self, sym: Symbol, span: &Span, lexical: Option<SymbolId>) -> SymbolId {
        let name = sym.name;
        let linkage = sym.linkage;
        let definition = sym.definition;
        let new_ty = sym.ty;

        let Some(entry) = self.externals.get(&name.id) else {
            let id = lexical.unwrap_or_else(|| self.symbols.alloc(sym));
            let (defined, tentative) = match definition {
                Definition::Definition => (Some(*span), None),
                Definition::Tentative => (None, Some(*span)),
                Definition::Declaration => (None, None),
            };
            self.externals.insert(
                name.id,
                External {
                    symbol: id,
                    defined,
                    tentative,
                },
            );
            return id;
        };
        let entry_symbol = entry.symbol;
        let entry_linkage = entry_symbol.resolve(self).linkage;

        if entry_linkage != linkage {
            self.add_diag(Diag::err((), Diagnosis::ConflictingLinkage(name)), span);
        }

        let old_ty = entry_symbol.resolve(self).ty;
        if let Some((old_ty, new_ty)) = Option::zip(old_ty, new_ty)
            && let Some(merged) = old_ty.composite(self, &new_ty)
        {
            entry_symbol.resolve_mut(self).ty = Some(merged);
        }

        let old_definition = entry_symbol.resolve(self).definition;
        entry_symbol.resolve_mut(self).definition = promote_definition(old_definition, definition);

        let entry = self.externals.get_mut(&name.id).unwrap();
        match definition {
            Definition::Definition if entry.defined.is_none() => entry.defined = Some(*span),
            Definition::Tentative if entry.tentative.is_none() => entry.tentative = Some(*span),
            _ => (),
        }

        entry_symbol
    }

    fn dedup(&mut self, sym: &Symbol, span: &Span) -> Option<SymbolId> {
        let old_id = self.scopes.current(sym.kind, sym.name.id)?;
        let old_symbol = old_id.resolve(self);
        if (self.scopes.kind() == ScopeKind::File
            || (sym.linkage != Linkage::None && old_symbol.linkage != Linkage::None))
            && old_symbol.is_compatible(self, sym)
            && !(sym.is_init && old_symbol.is_init)
        {
            return Some(old_id);
        }
        self.add_diag(
            Diag::err(Some(old_id), Diagnosis::DuplicateDeclaration(sym.kind, sym.name)),
            span,
        )
    }

    pub fn add_label_symbol(&mut self, name: Name, span: &Span, is_init: bool) {
        if let Some(old) = self.scopes.lookup_label(name.id) {
            let old_init = old.resolve(self).is_init;
            if !old_init && is_init {
                old.resolve_mut(self).is_init = true;
                return;
            }
            if !(is_init && old_init) {
                return;
            }
            return self.add_diag(
                Diag::err((), Diagnosis::DuplicateDeclaration(SymbolKind::Label, name)),
                span,
            );
        };
        let sym_id = self.symbols.alloc(Symbol::label(name, is_init));
        self.scopes.insert(SymbolKind::Label, name.id, sym_id);
    }
}

fn definition_rank(definition: Definition) -> u8 {
    match definition {
        Definition::Declaration => 0,
        Definition::Tentative => 1,
        Definition::Definition => 2,
    }
}

fn promote_definition(a: Definition, b: Definition) -> Definition {
    if definition_rank(b) > definition_rank(a) { b } else { a }
}
