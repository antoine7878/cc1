use std::collections::HashMap;

use libft::Span;

use crate::arena::{Global, Has, HasMut, HasTable, SideTable};
use crate::ast::statement::StatementId;
use crate::ast::{AstArenas, ConstValue, DeclaratorId, ExpressionId, NameId};
use crate::semantic::declaration::FunctionHeader;
use crate::semantic::{
    Builtins, DefinitionState, Diag, Diagnostic, DiagnosticNode, DiagnosticSink, FunctionDef, FunctionDefArena,
    FunctionDefId, Initializer, InitializerArena, InitializerId, Layout, Linkage, MemberRef, ResolvedExpression,
    ResolvedStatement, ResolvedType, ResolvedTypeId, ResolvedTypeInterner, Symbol, SymbolArena, SymbolId, TagDef,
    TagDefArena, TagDefId,
};

#[derive(Debug, Clone)]
pub struct External {
    pub symbol: SymbolId,
    pub defined: Option<Span>,
    pub tentative: Option<Span>,
}

thread_local! {
    static SEMA: Global<Sema> = const { Global::new("Sema") };
}

pub fn install_sema(sema: Sema) -> &'static Sema {
    SEMA.with(|g| g.install(sema))
}

pub fn sema() -> &'static Sema {
    SEMA.with(Global::get)
}

#[derive(Debug)]
pub struct Sema {
    pub diagnostics: Vec<DiagnosticNode>,

    pub symbols: SymbolArena,
    pub types: ResolvedTypeInterner,
    pub builtins: Builtins,
    pub tags: TagDefArena,
    pub functions: FunctionDefArena,
    pub initializers: InitializerArena,

    pub expressions: SideTable<ExpressionId, ResolvedExpression>,
    pub expr_bindings: SideTable<ExpressionId, SymbolId>,
    pub expr_consts: SideTable<ExpressionId, ConstValue>,
    pub member_refs: SideTable<ExpressionId, MemberRef>,
    pub declarations: HashMap<DeclaratorId, SymbolId>,
    pub externals: HashMap<NameId, External>,
    pub statements: SideTable<StatementId, ResolvedStatement>,

    pub headers: HashMap<DeclaratorId, FunctionHeader>,
    pub layouts: HashMap<ResolvedTypeId, Layout>,
}

impl Default for Sema {
    fn default() -> Self {
        let mut types = ResolvedTypeInterner::default();
        let builtins = Builtins::new(&mut types);

        Self {
            diagnostics: Vec::new(),
            symbols: SymbolArena::default(),
            types,
            builtins,
            tags: TagDefArena::default(),
            functions: FunctionDefArena::default(),
            initializers: InitializerArena::default(),

            expressions: SideTable::default(),
            expr_bindings: SideTable::default(),
            expr_consts: SideTable::default(),
            member_refs: SideTable::default(),
            statements: SideTable::default(),

            headers: HashMap::default(),
            declarations: HashMap::default(),
            externals: HashMap::default(),

            layouts: HashMap::default(),
        }
    }
}

impl Has<Symbol> for Sema {
    fn get(&self, id: SymbolId) -> &Symbol {
        self.symbols.get(id)
    }
}

impl HasMut<Symbol> for Sema {
    fn get_mut(&mut self, id: SymbolId) -> &mut Symbol {
        self.symbols.get_mut(id)
    }
}

impl Has<ResolvedType> for Sema {
    fn get(&self, id: ResolvedTypeId) -> &ResolvedType {
        self.types.get(id)
    }
}

impl Has<TagDef> for Sema {
    fn get(&self, id: TagDefId) -> &TagDef {
        self.tags.get(id)
    }
}

impl HasMut<TagDef> for Sema {
    fn get_mut(&mut self, id: TagDefId) -> &mut TagDef {
        self.tags.get_mut(id)
    }
}

impl Has<FunctionDef> for Sema {
    fn get(&self, id: FunctionDefId) -> &FunctionDef {
        self.functions.get(id)
    }
}

impl HasMut<FunctionDef> for Sema {
    fn get_mut(&mut self, id: FunctionDefId) -> &mut FunctionDef {
        self.functions.get_mut(id)
    }
}

impl Has<Initializer> for Sema {
    fn get(&self, id: InitializerId) -> &Initializer {
        self.initializers.get(id)
    }
}

impl HasMut<Initializer> for Sema {
    fn get_mut(&mut self, id: InitializerId) -> &mut Initializer {
        self.initializers.get_mut(id)
    }
}

impl HasTable<StatementId, ResolvedStatement> for Sema {
    fn table(&mut self) -> &mut SideTable<StatementId, ResolvedStatement> {
        &mut self.statements
    }
}

impl HasTable<ExpressionId, ResolvedExpression> for Sema {
    fn table(&mut self) -> &mut SideTable<ExpressionId, ResolvedExpression> {
        &mut self.expressions
    }
}

impl DiagnosticSink for Sema {
    fn diagnostics(&mut self) -> &mut Vec<DiagnosticNode> {
        &mut self.diagnostics
    }
}

impl Sema {
    // ----- Expression table ---------

    pub fn size_tables(&mut self, arenas: &AstArenas) {
        let len = arenas.expressions.len();
        self.expressions.resize(len);
        self.expr_bindings.resize(len);
        self.expr_consts.resize(len);
        self.member_refs.resize(len);
        self.statements.resize(arenas.statements.len());
    }
    // ----- Externals ---------------------

    pub fn linkage_of_name(&self, name: NameId) -> Option<Linkage> {
        let id = self.externals.get(&name)?.symbol;
        Some(id.resolve_with(self).linkage)
    }

    pub fn register_external(&mut self, sym: Symbol, span: &Span, lexical: Option<SymbolId>) -> SymbolId {
        let name = sym.name;
        let linkage = sym.linkage;
        let definition = sym.definition;
        let new_ty = sym.ty;

        let Some(entry) = self.externals.get(&name.id) else {
            let id = lexical.unwrap_or_else(|| self.symbols.alloc(sym));
            let (defined, tentative) = match definition {
                DefinitionState::Defined => (Some(*span), None),
                DefinitionState::Tentative => (None, Some(*span)),
                DefinitionState::Declared => (None, None),
            };
            self.externals.insert(name.id, External { symbol: id, defined, tentative });
            return id;
        };
        let entry_symbol = entry.symbol;
        let entry_linkage = entry_symbol.resolve_with(self).linkage;

        if entry_linkage != linkage {
            self.add_diag(Diag::err((), Diagnostic::ConflictingLinkage(name)), span);
        }

        let old_ty = entry_symbol.resolve_with(self).ty;
        if let Some(merged) = old_ty.composite(self, &new_ty) {
            entry_symbol.resolve_mut(self).ty = merged;
        }

        let old_definition = entry_symbol.resolve_with(self).definition;
        entry_symbol.resolve_mut(self).definition = promote_definition(old_definition, definition);

        let entry = self.externals.get_mut(&name.id).unwrap();
        match definition {
            DefinitionState::Defined if entry.defined.is_none() => entry.defined = Some(*span),
            DefinitionState::Tentative if entry.tentative.is_none() => entry.tentative = Some(*span),
            _ => (),
        }

        entry_symbol
    }

    pub fn layout(&self, id: &ResolvedTypeId) -> Layout {
        let ty = self.types.get(*id);
        ty.layout().unwrap_or_else(|| self.layouts[id])
    }
}

fn definition_rank(definition: DefinitionState) -> u8 {
    match definition {
        DefinitionState::Declared => 0,
        DefinitionState::Tentative => 1,
        DefinitionState::Defined => 2,
    }
}

fn promote_definition(a: DefinitionState, b: DefinitionState) -> DefinitionState {
    if definition_rank(b) > definition_rank(a) { b } else { a }
}
