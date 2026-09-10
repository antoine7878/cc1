use cc1::ast::visit::{Visitor, walk_expression, walk_statement, walk_translation_unit};
use cc1::ast::{Expression, ExpressionNode, Statement, StatementNode};
use cc1::semantic::Sema;

use crate::common::Unit;

/// Walks a whole translation unit and reports every fact an LLVM emitter would ask for and not
/// find. Diagnostic tests cannot catch a missing fact, only a wrong one, so this is the check
/// that says the side tables are actually complete.
struct FactChecker<'a> {
    sema: &'a Sema,
    missing: Vec<String>,
}

impl FactChecker<'_> {
    fn require(&mut self, present: bool, what: &str, at: usize) {
        if !present {
            self.missing.push(format!("{what} #{at}"));
        }
    }
}

impl Visitor for FactChecker<'_> {
    fn visit_expression(&mut self, node: &ExpressionNode) {
        walk_expression(self, node);

        let sema = self.sema;
        let id = node.id;
        let at = usize::from(id);
        self.require(sema.expr_types.get(id).is_some(), "type of expression", at);

        match id.resolve() {
            Expression::Identifier(_) => {
                self.require(sema.expr_bindings.get(id).is_some(), "binding of identifier", at)
            }
            Expression::Member(_, _, _) => self.require(sema.member_refs.get(id).is_some(), "member reference", at),
            Expression::StringLiteral(_) | Expression::Constant(_) => (),
            _ => (),
        }
    }

    fn visit_statement(&mut self, node: &StatementNode) {
        walk_statement(self, node);

        let at = usize::from(node.id);
        let recorded = self.sema.stmts.get(node.id).is_some();
        match node.id.resolve() {
            Statement::Iteration(_) => self.require(recorded, "loop facts", at),
            Statement::Labeled(_) => self.require(recorded, "label facts", at),
            Statement::Jump(inner) => match inner.stmt {
                cc1::ast::JumpStatement::Return(_) => (),
                _ => self.require(recorded, "jump target", at),
            },
            Statement::Selection(inner) => match inner.stmt {
                cc1::ast::SelectionStatement::Switch(_, _) => self.require(recorded, "switch table", at),
                cc1::ast::SelectionStatement::If(_, _, _) => (),
            },
            _ => (),
        }
    }
}

impl Unit {
    /// Every fact a code generator would need and not find, empty when the unit is complete.
    pub fn missing_facts(&self) -> Vec<String> {
        let mut checker = FactChecker {
            sema: self.sema,
            missing: Vec::new(),
        };
        walk_translation_unit(&mut checker, &self.ctx.ast);
        checker.missing
    }
}

pub fn run_facts(name: &str, src: &str) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(
        unit.diagnosis().is_empty(),
        "`{name}` unexpected diagnosis:\n{src}\n{}",
        unit.render()
    );
    assert_eq!(
        unit.missing_facts(),
        Vec::<String>::new(),
        "`{name}` is missing facts a code generator needs:\n{src}"
    );
}
