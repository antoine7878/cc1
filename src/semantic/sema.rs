use std::collections::HashMap;

use crate::arena::{ResolveMutWith, ResolveWith};
use crate::ast::{DeclaratorId, ExpressionId, Name, Tag, Value};
use crate::parser::Span;
use crate::semantic::{
    Builtins, Diag, DiagCollector, Diagnosis, DiagnosisNode, FunctionDefArena, QualifiedType, ResolvedExpression,
    ResolvedTypeArena, ResolvedTypeId, ScopeKind, Scopes, Symbol, SymbolArena, SymbolId, SymbolKind, TagDefArena,
    TagDefId,
};
use crate::target::{Layout, Target};

#[derive(Debug)]
pub struct Sema {
    pub scopes: Scopes,
    pub diagnosis: Vec<DiagnosisNode>,

    pub symbols: SymbolArena,
    pub types: ResolvedTypeArena,
    pub builtins: Builtins,
    pub tags: TagDefArena,
    pub functions: FunctionDefArena,

    pub expressions: HashMap<ExpressionId, Option<ResolvedExpression>>,
    pub bindings: HashMap<ExpressionId, Option<SymbolId>>,
    pub constants: HashMap<ExpressionId, Option<Value>>,
    pub declarations: HashMap<DeclaratorId, SymbolId>,

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

            expressions: HashMap::default(),
            bindings: HashMap::default(),
            declarations: HashMap::default(),
            constants: HashMap::default(),

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

    // ----- Resolution --------------------

    pub fn resolve_typedef(
        &mut self,
        name: Name,
        is_const: bool,
        is_volatile: bool,
        span: &Span,
    ) -> Option<QualifiedType> {
        let sym_id = self.scopes.lookup_ordinary(name.id)?;
        let sym = sym_id.resolve(&*self);
        if sym.kind != SymbolKind::Typedef {
            return self.add_diag(Diag::none_diag(Diagnosis::UndeclaredIdentifier(name)), span);
        }
        let base = sym.ty?;
        if (is_const && base.is_const) || (is_volatile && base.is_volatile) {
            self.add_diag(Diag::only_diag(Diagnosis::DuplicateTypeQualifers), span)
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
            let def = id.resolve(&*self);
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
        let old_symbol = old_id.resolve(&*self);
        if self.scopes.kind() == ScopeKind::File
            && old_symbol.is_compatible(self, sym)
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
            let old_init = old.resolve(&*self).is_init;
            if !old_init && is_init {
                old.resolve_mut(self).is_init = true;
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

    // pub fn push_cast(&mut self, expr: &mut Typed, kind: CastKind, to: QualifiedType) {
    //     self.expressions
    //         .entry(expr.id)
    //         .or_default()
    //         .casts
    //         .push(ImplicitCast { kind, to });
    //     expr.ty = to;
    //     expr.kind = ExpressionKind::RValue;
    // }

    // pub fn record(&mut self, id: ExpressionId, ty: QualifiedType, kind: ExpressionKind) -> Typed {
    //     let e = self.expressions.entry(id).or_default();
    //     e.ty = Some(ty);
    //     e.kind = Some(kind);
    //     Typed { id, ty, kind }
    // }
}
