use std::collections::HashMap;

use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_expression, walk_function_definition, walk_translation_unit,
};
use crate::ast::{CompoundStatementNode, DeclarationNode, ExpressionNode, FunctionDefinitionNode};
use crate::ast::{Expression, Storage, StringId};
use crate::parser::{Context, Span};
use crate::semantic::{Diag, Diagnosis, SymbolArena, SymbolId, declarations};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
    Block,
    Function,
    File,
    Prototype,
}

#[derive(Debug)]
struct Scope {
    tags: HashMap<String, SymbolId>,
    members: HashMap<String, SymbolId>,
    labels: HashMap<String, SymbolId>,
    ordinaries: HashMap<StringId, SymbolId>,
    ty: ScopeType,
}

impl Scope {
    pub fn new(ty: ScopeType) -> Self {
        Self {
            tags: HashMap::new(),
            members: HashMap::new(),
            labels: HashMap::new(),
            ordinaries: HashMap::new(),
            ty,
        }
    }
}

#[derive(Default, Debug)]
struct SymbolResolver {
    scopes: Vec<Scope>,
    diagnosis: Vec<Diagnosis>,
    symbols: SymbolArena,
}

impl SymbolResolver {
    fn scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    fn extract_diag<T>(&mut self, diag: Diag<T>, span: &Span) -> T {
        let Diag::<T> { res, diagnosis } = diag;
        if let Some(d) = diagnosis {
            self.diagnosis.push(Diagnosis::new(d, *span));
        }
        res
    }

    fn scope(&self) -> &Scope {
        self.scopes.last().unwrap()
    }

    // fn find_symbol(&mut self, name: &Name) -> bool {
    //     self.scopes
    //         .iter()
    //         .rev()
    //         .any(|scope| scope.ordinaries.contains_key(&name.id))
    // }

    // fn check_symbol(&mut self, ctx: &Context, name: &Name, span: &Span) {
    //     if !self.find_symbol(ctx, name) {
    //         self.diagnosis.push(Diagnosis::undeclared_identifier(name, span))
    //     }
    // }

    fn add_function(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        let Some(name) = node.declarator.ident(ctx) else { return };
        let span = &node.span;
        let storage = self
            .extract_diag(declarations::get_storage(&node.specifiers), span)
            .unwrap_or(Storage::Extern);
        self.extract_diag(declarations::extern_function_only(self.scope().ty, storage), span);
        let ty = self.extract_diag(declarations::resolve_type(&node.specifiers), span);
        let (is_const, is_volatile) = self.extract_diag(declarations::get_qualifier(&node.specifiers), span);

        if let Some(ty) = ty {
            let sym_id = self
                .symbols
                .add(name, ty, storage, is_const, is_volatile, node.declarator.clone());
            self.scope_mut().ordinaries.insert(name.id, sym_id);
        }
    }

    fn add_arguments(&mut self, ctx: &Context, node: &DeclarationNode) {
        // let span = &node.span;
        // let Some(name) = node.declarator.ident(ctx) else { return };
    }
}

impl Visitor for SymbolResolver {
    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode, _is_last: bool) {
        // let a = node.declarator
        self.add_function(ctx, node);

        self.scopes.push(Scope::new(ScopeType::Prototype));
        for arg in &node.old_style_declarations {
            self.add_arguments(ctx, arg);
        }
        self.scopes.pop();

        self.visit_compound_statement(ctx, &node.body, true);
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode, is_last: bool) {
        if let Expression::Identifier(_name) = node.id.resolve(&ctx.arenas) {
            // self.check_symbol(ctx, name, node.span);
        }
        walk_expression(self, ctx, node, is_last);
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode, is_last: bool) {
        self.scopes.push(Scope::new(ScopeType::Block));
        walk_compound_statement(self, ctx, node, is_last);
        self.scopes.pop();
    }
}

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(ctx: Context) -> Context {
        Self::resolve_names(&ctx);
        ctx
    }

    fn resolve_names(ctx: &Context) -> Vec<DeclarationNode> {
        let mut collector = SymbolResolver::default();
        collector.scopes.push(Scope::new(ScopeType::File));
        walk_translation_unit(&mut collector, ctx, &ctx.ast, false);
        collector.scopes.pop();
        assert!(collector.scopes.is_empty());
        for symbol in &collector.symbols.data {
            println!("{}", symbol.name.id.resolve(&ctx.arenas));
        }
        for diag in &collector.diagnosis {
            let _ = diag.print(ctx);
        }
        Vec::new()
    }
}
