use std::io::{Write, stdout};

use libft::{Position, Span};

use crate::ast::statement::StatementId;
use crate::ast::visit::{walk_statement, walk_translation_unit};
use crate::ast::{
    BinaryOp, Expression, ExpressionNode, FunctionDefinitionNode, InitDeclaratorNode, Statement, StatementNode,
    TranslationUnitNode, UnaryOp, Visitor,
};
use crate::codegen::{Builder, Frozen, Globals, LlvmSymbol, Locals};
use crate::context::ctx;
use crate::semantic::{DefinitionState, Diagnostic, DiagnosticNode, Duration, Initializer, SymbolId, sema};

pub fn generate() -> Vec<DiagnosticNode> {
    generate_to(stdout())
}

pub fn generate_to<W: Write>(w: W) -> Vec<DiagnosticNode> {
    let mut generator = Generator::new(w);
    generator.visit_translation_unit(&ctx().ast);
    if let Some(error) = generator.builder.finish() {
        let diagnostic = Diagnostic::OutputError(error.to_string());
        let no_file = Position { file: usize::MAX, ..Position::default() };
        generator.diagnostics.push(DiagnosticNode::new(diagnostic, Span::new(no_file, no_file)));
    }
    generator.diagnostics
}

#[derive(Debug)]
pub struct Generator<W: Write> {
    pub builder: Builder<W>,
    pub locals: Locals,
    pub globals: Globals,
    pub diagnostics: Vec<DiagnosticNode>,
}

impl<W: Write> Generator<W> {
    fn new(w: W) -> Self {
        Self {
            builder: Builder::new(w),
            locals: Locals::default(),
            globals: Globals::default(),
            diagnostics: Vec::new(),
        }
    }

    fn collect_diag(&mut self, res: Result<(), Diagnostic>, span: &Span) {
        if let Err(diagnostic) = res {
            self.diagnostics.push(DiagnosticNode::new(diagnostic, *span));
        }
    }

    fn emit_init(&mut self, id: SymbolId) {
        let sym = id.resolve();
        let Some(init) = sym.initializer else { return };
        let qty = sym.ty;
        match init.resolve() {
            Initializer::Zero => {
                let _ = self.builder.store(LlvmSymbol::zero(qty), self.locals[id], qty.is_volatile);
            }
            Initializer::Value(v) => {
                let v = self.emit_constant(qty, *v);
                self.builder.store(v, self.locals[id], qty.is_volatile);
            }
            Initializer::Expr(e) if qty.is_record(sema()) => {
                let res = self.emit_copy_aggregate(self.locals[id], e, qty).map(|_| ());
                self.collect_diag(res, &e.span);
            }
            Initializer::Expr(e) => {
                let res = self.emit_expression(e).map(|v| {
                    let _ = self.builder.store(v, self.locals[id], qty.is_volatile);
                });
                self.collect_diag(res, &e.span);
            }
            Initializer::List(_) | Initializer::String(_) => {
                let src = self.globals.aggregates[&id];
                let layout = sema().layout(&qty.id);
                self.builder.memcpy(self.locals[id].name, src.name, layout, qty.is_volatile);
            }
            Initializer::Address(_) => unreachable!(),
        }
    }

    pub fn emit_condition(&mut self, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnostic> {
        let re = &sema().expressions[node.id];
        if re.casts.is_empty() && sema().expr_consts.get(node.id).is_none() {
            match node.id.resolve() {
                Expression::Binary(op, lhs, rhs) if op.is_comparison() => return self.emit_comparison(op, lhs, rhs),
                Expression::Binary(op @ (BinaryOp::LogicalAnd | BinaryOp::LogicalOr), lhs, rhs) => {
                    return self.emit_logical(op, lhs, rhs);
                }
                Expression::Unary(UnaryOp::LogicalNot, e) => return self.emit_logical_not(e),
                _ => (),
            }
        }
        let v = self.emit_expression(node)?;
        self.emit_nonzero(v, re.casted_ty())
    }

    fn emit_globals(&mut self) {
        let mut literals: Vec<_> = self.globals.literals.iter().collect();
        literals.sort_by_key(|(id, _)| usize::from(**id));
        for (id, sym) in literals {
            self.builder.string_literal(sym.name, id.resolve());
        }

        for index in 0..sema().tags.len() {
            self.builder.define_type(index.into());
        }
        for sym in &self.globals.order {
            self.builder.define_global(*sym);
        }
        for sym in &self.globals.functions {
            if sym.resolve().definition != DefinitionState::Defined {
                self.builder.declare_function(*sym);
            }
        }

        let mut aggregates: Vec<_> = self.globals.aggregates.iter().collect();
        aggregates.sort_by_key(|(id, _)| usize::from(**id));
        for (id, sym) in aggregates {
            let sem_sym = id.resolve();
            self.builder.aggregate_constant(sym.name, sem_sym.ty, sem_sym.initializer.unwrap().resolve());
        }
    }

    pub fn emit_statement(&mut self, id: StatementId) -> Result<(), Diagnostic> {
        match id.resolve() {
            Statement::Jump(inner) => self.emit_jump_statement(id, inner),
            Statement::Selection(inner) => self.emit_selection_statement(id, inner),
            Statement::Iteration(inner) => self.emit_iteration_statement(id, inner),
            Statement::Labeled(inner) => self.emit_labeled_statement(id, inner),
            _ => Ok(()),
        }
    }
}

impl<W: Write> Visitor for Generator<W> {
    fn visit_translation_unit(&mut self, node: &TranslationUnitNode) {
        self.builder.target();
        self.globals.collect(node);
        self.emit_globals();

        walk_translation_unit(self, node);
    }

    fn visit_function_definition(&mut self, node: &FunctionDefinitionNode) {
        let sym = sema().declarations[&node.declarator.id];
        self.locals.collect_locals(node);
        self.builder.define_function(sym, self.locals.params(), self.locals.is_variadic());
        self.locals.emit_decl(&mut self.builder);
        self.visit_compound_statement(&node.body);
        self.builder.end_function(sym);
    }

    fn visit_expression(&mut self, node: &ExpressionNode) {
        let res = self.emit_expression(node).map(|_| ());
        self.collect_diag(res, &node.span);
    }

    fn visit_statement(&mut self, node: &StatementNode) {
        let res = match node.id.resolve() {
            Statement::Jump(_) | Statement::Selection(_) | Statement::Iteration(_) | Statement::Labeled(_) => {
                self.emit_statement(node.id)
            }
            Statement::Expression(_) | Statement::Compound(_) => return walk_statement(self, node),
        };
        self.collect_diag(res, &node.span);
    }

    fn visit_init_declarator(&mut self, node: &InitDeclaratorNode) {
        let sym_id = sema().declarations[&node.declarator.id];
        if sym_id.resolve().duration != Duration::Automatic {
            return;
        }
        self.emit_init(sym_id);
    }
}
