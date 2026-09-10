use std::cell::Cell;
use std::collections::HashMap;

use crate::arena::{Global, Has, HasMut, HasTable, Owned, SideTable};
use crate::ast::statement::StatementId;
use crate::ast::{AstArenas, DeclaratorId, ExpressionId, StringId, Value};
use crate::semantic::{
    Builtins, Definition, Diag, DiagCollector, Diagnosis, DiagnosisNode, FunctionDef, FunctionDefArena, FunctionDefId,
    Initializer, InitializerArena, InitializerId, Linkage, MemberRef, ResolvedExpression, ResolvedStatement,
    ResolvedType, ResolvedTypeArena, ResolvedTypeId, Symbol, SymbolArena, SymbolId, TagDef, TagDefArena, TagDefId,
};
use crate::target::{Layout, Target};
use libft::Span;

#[derive(Debug, Clone)]
pub struct External {
    pub symbol: SymbolId,
    pub defined: Option<Span>,
    pub tentative: Option<Span>,
}

thread_local! {
    static SEMA: Cell<Option<&'static Sema>> = const { Cell::new(None) };
}

pub fn install(sema: Sema) -> &'static Sema {
    let sema = Box::leak(Box::new(sema));
    SEMA.set(Some(sema));
    sema
}

pub fn sema() -> &'static Sema {
    SEMA.get().expect("Sema is not installed")
}

#[derive(Debug)]
pub struct Sema {
    pub diagnosis: Vec<DiagnosisNode>,

    pub symbols: SymbolArena,
    pub types: ResolvedTypeArena,
    pub builtins: Builtins,
    pub tags: TagDefArena,
    pub functions: FunctionDefArena,
    pub inits: InitializerArena,

    pub expr_types: SideTable<ExpressionId, ResolvedExpression>,
    pub expr_bindings: SideTable<ExpressionId, SymbolId>,
    pub expr_consts: SideTable<ExpressionId, Value>,
    pub member_refs: SideTable<ExpressionId, MemberRef>,
    pub declarations: HashMap<DeclaratorId, SymbolId>,
    pub externals: HashMap<StringId, External>,
    pub stmts: SideTable<StatementId, ResolvedStatement>,

    pub layouts: HashMap<ResolvedTypeId, Layout>,
    pub target: Target,
}

impl Default for Sema {
    fn default() -> Self {
        Self::new(Target::default())
    }
}

impl Sema {
    pub fn new(target: Target) -> Self {
        let mut types = ResolvedTypeArena::default();
        let builtins = Builtins::new(&mut types, &target);
        Self {
            diagnosis: Vec::new(),
            symbols: SymbolArena::default(),
            types,
            builtins,
            tags: TagDefArena::default(),
            functions: FunctionDefArena::default(),
            inits: InitializerArena::default(),

            expr_types: SideTable::default(),
            expr_bindings: SideTable::default(),
            expr_consts: SideTable::default(),
            member_refs: SideTable::default(),
            stmts: SideTable::default(),

            declarations: HashMap::default(),
            externals: HashMap::default(),

            layouts: HashMap::default(),
            target,
        }
    }
}

impl Global for Sema {
    fn global() -> &'static Self {
        sema()
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

impl Owned for Symbol {
    type Holder = Sema;
}

impl Has<ResolvedType> for Sema {
    fn get(&self, id: ResolvedTypeId) -> &ResolvedType {
        self.types.get(id)
    }
}

impl Owned for ResolvedType {
    type Holder = Sema;
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

impl Owned for TagDef {
    type Holder = Sema;
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

impl Owned for FunctionDef {
    type Holder = Sema;
}

impl Has<Initializer> for Sema {
    fn get(&self, id: InitializerId) -> &Initializer {
        self.inits.get(id)
    }
}

impl HasMut<Initializer> for Sema {
    fn get_mut(&mut self, id: InitializerId) -> &mut Initializer {
        self.inits.get_mut(id)
    }
}

impl Owned for Initializer {
    type Holder = Sema;
}

impl HasTable<StatementId, ResolvedStatement> for Sema {
    fn table(&mut self) -> &mut SideTable<StatementId, ResolvedStatement> {
        &mut self.stmts
    }
}

impl HasTable<ExpressionId, ResolvedExpression> for Sema {
    fn table(&mut self) -> &mut SideTable<ExpressionId, ResolvedExpression> {
        &mut self.expr_types
    }
}

impl DiagCollector for Sema {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        &mut self.diagnosis
    }
}

impl Sema {
    // ----- Expression table ---------

    pub fn size_tables(&mut self, arenas: &AstArenas) {
        let len = arenas.expressions.len();
        self.expr_types.resize(len);
        self.expr_bindings.resize(len);
        self.expr_consts.resize(len);
        self.member_refs.resize(len);
        self.stmts.resize(arenas.statements.len());
    }
    // ----- Externals ---------------------

    pub fn linkage_of_name(&self, name: StringId) -> Option<Linkage> {
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
        let entry_linkage = entry_symbol.resolve_with(self).linkage;

        if entry_linkage != linkage {
            self.add_diag(Diag::err((), Diagnosis::ConflictingLinkage(name)), span);
        }

        let old_ty = entry_symbol.resolve_with(self).ty;
        if let Some((old_ty, new_ty)) = Option::zip(old_ty, new_ty)
            && let Some(merged) = old_ty.composite(self, &new_ty)
        {
            entry_symbol.resolve_mut(self).ty = Some(merged);
        }

        let old_definition = entry_symbol.resolve_with(self).definition;
        entry_symbol.resolve_mut(self).definition = promote_definition(old_definition, definition);

        let entry = self.externals.get_mut(&name.id).unwrap();
        match definition {
            Definition::Definition if entry.defined.is_none() => entry.defined = Some(*span),
            Definition::Tentative if entry.tentative.is_none() => entry.tentative = Some(*span),
            _ => (),
        }

        entry_symbol
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
