use std::collections::HashMap;
use std::mem::take;

use crate::arena::{HasTable, ResolveMutWith, ResolveWith, SideTable};
use crate::ast::statement::StatementId;
use crate::ast::{AstArenas, DeclaratorId, ExpressionId, StringId, Value};
use crate::context::Context;
use crate::parser::Span;
use crate::semantic::{
    Builtins, Definition, Diag, DiagCollector, Diagnosis, DiagnosisNode, FunctionDefArena, InitializerArena, Linkage,
    MemberRef, ResolvedExpression, ResolvedStatement, ResolvedTypeArena, ResolvedTypeId, Symbol, SymbolArena, SymbolId,
    TagDefArena,
};
use crate::target::{Layout, Target};

#[derive(Debug, Clone)]
pub struct External {
    pub symbol: SymbolId,
    pub defined: Option<Span>,
    pub tentative: Option<Span>,
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
        let mut types = ResolvedTypeArena::default();
        let target = Target::default();
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
    pub fn new(target: Target) -> Self {
        Self {
            target,
            ..Self::default()
        }
    }

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
        Some(id.resolve(self).linkage)
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

    pub fn with_sema(mut ctx: Context, pass: impl FnOnce(&mut Sema, &Context)) -> Context {
        let mut sema = take(&mut ctx.sema);
        pass(&mut sema, &ctx);
        ctx.diagnosis.append(&mut sema.diagnosis);
        ctx.sema = sema;
        ctx
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
