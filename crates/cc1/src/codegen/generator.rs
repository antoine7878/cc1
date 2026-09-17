use std::io::{Write, stdout};

use libft::Span;

use crate::ast::visit::{walk_statement, walk_translation_unit};
use crate::ast::{
    BinaryOp, Expression, ExpressionNode, FunctionDefinitionNode, InitDeclaratorNode, Statement, StatementNode,
    TranslationUnitNode, UnaryOp, Visitor,
};
use crate::codegen::{Builder, Globals, LlvmName, LlvmSymbol, Locals};
use crate::context::ctx;
use crate::semantic::{Definition, Diagnosis, DiagnosisNode, Duration, Initializer, SymbolId, sema};

pub fn generate() -> Vec<DiagnosisNode> {
    generate_to(stdout())
}

pub fn generate_to<W: Write>(w: W) -> Vec<DiagnosisNode> {
    let mut generator = Generator::new(w);
    generator.visit_translation_unit(&ctx().ast);
    generator.diagnosis
}

#[derive(Debug)]
pub struct Generator<W: Write> {
    pub b: Builder<W>,
    pub locals: Locals,
    pub globals: Globals,
    pub diagnosis: Vec<DiagnosisNode>,
    pub break_label: LlvmName,
    pub continue_label: LlvmName,
}

impl<W: Write> Generator<W> {
    fn new(w: W) -> Self {
        Self {
            b: Builder::new(w),
            locals: Locals::default(),
            globals: Globals::default(),
            diagnosis: Vec::new(),
            break_label: LlvmName::label(0),
            continue_label: LlvmName::label(0),
        }
    }

    fn collect_diag(&mut self, res: Result<(), Diagnosis>, span: &Span) {
        if let Err(diagnosis) = res {
            self.diagnosis.push(DiagnosisNode::new(diagnosis, *span));
        }
    }

    fn emit_init(&mut self, id: SymbolId) {
        let sym = id.resolve();
        let Some(init) = sym.initializer else { return };
        let qty = sym.ty;
        match init.resolve() {
            Initializer::Zero => self.b.store(LlvmSymbol::zero(qty), self.locals[id]),
            Initializer::Value(v) => {
                let v = self.constant(qty, *v);
                self.b.store(v, self.locals[id]);
            }
            Initializer::String(s) => {
                let &a = self.globals.get_literal(*s).unwrap();
                self.b.store(a, self.locals[id]);
            }
            Initializer::Expr(e) => {
                let res = self.emit_expression(e).map(|v| self.b.store(v, self.locals[id]));
                self.collect_diag(res, &e.span);
            }
            Initializer::List(_) => todo!("list init"),
            Initializer::Address(_) => todo!("address init"),
        }
    }

    pub fn emit_condition(&mut self, node: &ExpressionNode) -> Result<LlvmSymbol, Diagnosis> {
        let re = &sema().expr_types[node.id];
        if re.casts.is_empty() && sema().expr_consts.get(node.id).is_none() {
            match node.id.resolve() {
                Expression::Binary(op, lhs, rhs) if op.is_comparison() => return self.comparison(op, lhs, rhs),
                Expression::Binary(op @ (BinaryOp::LogicalAnd | BinaryOp::LogicalOr), lhs, rhs) => {
                    return self.logical(op, lhs, rhs);
                }
                Expression::Unary(UnaryOp::LogicalNot, e) => return self.logic_not(e),
                _ => {}
            }
        }
        let v = self.emit_expression(node)?;
        self.neq_zero(v, re.casted_ty())
    }

    fn emit_globals(&mut self) {
        let mut strings: Vec<_> = self.globals.strings.iter().collect();
        strings.sort_by_key(|(id, _)| usize::from(**id));
        for (id, sym) in strings {
            self.b.string_literal(sym.name, id.resolve());
        }
        for index in 0..sema().tags.len() {
            self.b.type_def(index.into());
        }
        for sym in &self.globals.order {
            self.b.global(*sym);
        }
        for sym in &self.globals.functions {
            if sym.resolve().definition != Definition::Definition {
                self.b.declare(*sym);
            }
        }
    }
}

impl<W: Write> Visitor for Generator<W> {
    fn visit_translation_unit(&mut self, node: &TranslationUnitNode) {
        self.b.target(ctx().target.datalayout, ctx().target.triple);
        self.globals.collect(node);
        self.emit_globals();

        walk_translation_unit(self, node);
    }

    fn visit_function_definition(&mut self, node: &FunctionDefinitionNode) {
        let sym = sema().declarations[&node.declarator.id];
        let f = self.globals.get_symbol(sym).unwrap();
        self.locals.collect_locals(node);
        self.b.define(*f, self.locals.parameters(), self.locals.is_variadic());
        self.locals.emit_decl(&mut self.b);
        self.visit_compound_statement(&node.body);
        self.b.end_function(sym);
    }

    fn visit_expression(&mut self, node: &ExpressionNode) {
        let res = self.emit_expression(node).map(|_| ());
        self.collect_diag(res, &node.span);
    }

    fn visit_statement(&mut self, node: &StatementNode) {
        let res = match node.id.resolve() {
            Statement::Jump(node) => self.jump_statement(node),
            Statement::Selection(selection_node) => self.selection_statement(node, selection_node),
            Statement::Iteration(node) => self.iteration_statement(node),
            Statement::Labeled(node) => self.labeled_statement(node),
            _ => return walk_statement(self, node),
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
