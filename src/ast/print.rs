use std::fmt::Display;
use std::io::{self, Write};

use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, Enum,
    Expression, ExpressionNode, ExpressionStatementNode, ExternalDeclaration,
    ExternalDeclarationNode, FunctionDefinitionNode, FunctionParameters, FunctionParametersNode,
    InitDeclaratorNode, Initializer, InitializerNode, IterationStatement, IterationStatementNode,
    JumpStatement, JumpStatementNode, Labeled, LabeledStatementNode, Name, ParameterDeclaration,
    SelectionStatement, SelectionStatementNode, Statement, StatementNode, Struct,
    StructDeclaration, StructDeclarator, TranslationUnitNode, Type, TypeSpecifier, Union, Variant,
};
use crate::parser::Context;
use crate::utils::{CYAN, RESET};

pub struct AstPrinter<'a, W: Write> {
    ctx: &'a Context,
    prefix: String,
    w: W,
}

impl<'a, W: Write> AstPrinter<'a, W> {
    fn print_node<T, F>(&mut self, node: &T, is_last: bool, f: F) -> io::Result<()>
    where
        T: Display,
        F: FnOnce(&mut Self) -> io::Result<()>,
    {
        let (branch, extend) = if is_last { ("`-", "  ") } else { ("|-", "| ") };
        let prefix = &self.prefix;
        write!(self.w, "\n{RESET}{prefix}{branch} {node} ")?;
        self.prefix.push_str(extend);

        write!(self.w, "{}", CYAN)?;
        let result = f(self);

        self.prefix.pop();
        self.prefix.pop();
        result
    }

    fn print_name(&mut self, name: &Name) -> io::Result<()> {
        write!(self.w, "{}", self.ctx.arenas.names.get(name.id))
    }

    pub fn print_ast(w: W, ctx: &'a Context) -> io::Result<()> {
        let mut printer = Self {
            prefix: String::new(),
            ctx,
            w,
        };
        printer.print_translation_unit(&ctx.ast)?;
        Ok(())
    }

    fn print_translation_unit(&mut self, node: &TranslationUnitNode) -> io::Result<()> {
        write!(self.w, "{}", node)?;
        for (is_last, external) in node.declarations.iter().with_last() {
            self.print_external_declaration_node(external, is_last)?;
        }
        writeln!(self.w)
    }

    fn print_external_declaration_node(
        &mut self,
        node: &ExternalDeclarationNode,
        is_last: bool,
    ) -> io::Result<()> {
        match &node.decl {
            ExternalDeclaration::Declaration(declaration) => {
                self.print_declaration_node(declaration, is_last)
            }
            ExternalDeclaration::Function(function) => {
                self.print_function_definition_node(function, is_last)
            }
        }
    }

    fn print_function_definition_node(
        &mut self,
        node: &FunctionDefinitionNode,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            for decl in &node.specifiers {
                write!(printer.w, "{} ", decl)?;
            }
            printer.print_declarator_node(&node.declarator, false)?;
            // for arg in &node.arguments {
            //     printer.print_declaration_node(arg, false)?;
            // }
            printer.print_coumpound_statement_node(&node.body, true)
        })
    }

    fn print_coumpound_statement_node(
        &mut self,
        node: &CompoundStatementNode,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            for decl in &node.declarations {
                printer.print_declaration_node(decl, false)?;
            }
            for (is_last, stmt) in node.statements.iter().with_last() {
                printer.print_statement_node(stmt, is_last)?;
            }
            Ok(())
        })
    }

    fn print_statement_node(&mut self, node: &StatementNode, is_last: bool) -> io::Result<()> {
        let stmt = self.ctx.arenas.statements.get(node.id);
        match stmt {
            Statement::Labeled(node) => self.print_labeled_statement_node(node, is_last),
            Statement::Compound(node) => self.print_coumpound_statement_node(node, is_last),
            Statement::Expression(node) => self.print_expression_statement_node(node, is_last),
            Statement::Selection(node) => self.print_selection_statement_node(node, is_last),
            Statement::Iteration(node) => self.print_iteration_statement_node(node, is_last),
            Statement::Jump(node) => self.print_jump_statement_node(node, is_last),
        }
    }

    fn print_labeled_statement_node(
        &mut self,
        node: &LabeledStatementNode,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            write!(printer.w, "{} ", &node.inner)?;
            match &node.inner {
                Labeled::Identifier(name, stmt) => {
                    printer.print_name(name)?;
                    printer.print_statement_node(stmt, true)
                }
                Labeled::Case(expr, stmt) => {
                    printer.print_expression_node(expr, false)?;
                    printer.print_statement_node(stmt, true)
                }
                Labeled::Default(stmt) => printer.print_statement_node(stmt, true),
            }
        })
    }

    fn print_expression_statement_node(
        &mut self,
        node: &ExpressionStatementNode,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            if let Some(expr) = &node.expr {
                printer.print_expression_node(expr, true)?;
            }
            Ok(())
        })
    }

    fn print_selection_statement_node(
        &mut self,
        node: &SelectionStatementNode,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            write!(printer.w, "{} ", &node.stmt)?;
            match &node.stmt {
                SelectionStatement::If(cond, then, otherwise) => {
                    printer.print_expression_node(cond, false)?;
                    printer.print_statement_node(then, otherwise.is_none())?;
                    if let Some(otherwise) = otherwise {
                        printer.print_statement_node(otherwise, true)?;
                    }
                    Ok(())
                }
                SelectionStatement::Switch(cond, stmt) => {
                    printer.print_expression_node(cond, false)?;
                    printer.print_statement_node(stmt, true)
                }
            }
        })
    }

    fn print_iteration_statement_node(
        &mut self,
        node: &IterationStatementNode,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            write!(printer.w, "{} ", &node.stmt)?;
            match &node.stmt {
                IterationStatement::While(cond, body) => {
                    printer.print_expression_node(cond, false)?;
                    printer.print_statement_node(body, true)
                }
                IterationStatement::Do(body, cond) => {
                    printer.print_statement_node(body, false)?;
                    printer.print_expression_node(cond, true)
                }
                IterationStatement::For(init, cond, inc, body) => {
                    printer.print_expression_statement_node(init, false)?;
                    printer.print_expression_statement_node(cond, false)?;
                    if let Some(inc) = inc {
                        printer.print_expression_node(inc, false)?;
                    }
                    printer.print_statement_node(body, true)
                }
            }
        })
    }

    fn print_jump_statement_node(
        &mut self,
        node: &JumpStatementNode,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            write!(printer.w, "{} ", &node.stmt)?;
            match &node.stmt {
                JumpStatement::Goto | JumpStatement::Continue | JumpStatement::Break => Ok(()),
                JumpStatement::Return(expr) => {
                    if let Some(expr) = expr {
                        printer.print_expression_node(expr, true)?;
                    }
                    Ok(())
                }
            }
        })
    }

    fn print_declaration_node(&mut self, node: &DeclarationNode, is_last: bool) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            for spec in &node.specifiers {
                write!(printer.w, " {}", spec)?;
            }
            let tags = node
                .specifiers
                .iter()
                .filter(|s| {
                    matches!(
                        s,
                        DeclarationSpecifier::Type(
                            TypeSpecifier::Struct(_)
                                | TypeSpecifier::Union(_)
                                | TypeSpecifier::Enum(_)
                        )
                    )
                })
                .count();
            let total = tags + node.init_declarators.len();
            let mut i = 0;
            for spec in &node.specifiers {
                if let DeclarationSpecifier::Type(TypeSpecifier::Struct(id)) = spec {
                    i += 1;
                    printer.print_struct_node(printer.ctx.arenas.structs.get(*id), i == total)?;
                } else if let DeclarationSpecifier::Type(TypeSpecifier::Union(id)) = spec {
                    i += 1;
                    printer.print_union_node(printer.ctx.arenas.unions.get(*id), i == total)?;
                } else if let DeclarationSpecifier::Type(TypeSpecifier::Enum(id)) = spec {
                    i += 1;
                    printer.print_enum_node(printer.ctx.arenas.enums.get(*id), i == total)?;
                }
            }
            for (is_last, init_declarator) in node.init_declarators.iter().with_last() {
                printer.print_init_declaration_node(init_declarator, is_last)?;
            }
            Ok(())
        })
    }

    fn print_init_declaration_node(
        &mut self,
        node: &InitDeclaratorNode,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            printer.print_declarator_node(&node.declarator, node.initializer.is_none())?;
            if let Some(init) = &node.initializer {
                printer.print_initializer_node(init, true)?;
            }
            Ok(())
        })
    }

    fn print_declarator_node(&mut self, node: &DeclaratorNode, is_last: bool) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            let decl = printer.ctx.arenas.declarators.get(node.id);
            printer.print_declarator(decl)?;
            Ok(())
        })
    }

    fn print_declarator(&mut self, decl: &Declarator) -> io::Result<()> {
        write!(self.w, "{} ", decl)?;
        match decl {
            Declarator::Ident(name) => self.print_name(name),
            Declarator::Abstract => Ok(()),
            Declarator::Pointer { qualifiers, inner } => {
                for spec in qualifiers {
                    write!(self.w, " {}", spec)?;
                }
                if let Some(node) = inner {
                    self.print_declarator_node(node, true)?;
                }
                Ok(())
            }
            Declarator::Array { declarator, size } => {
                self.print_declarator_node(declarator, size.is_none())?;
                if let Some(node) = size {
                    self.print_expression_node(node, true)?;
                }
                Ok(())
            }
            Declarator::Function { declarator, params } => {
                self.print_declarator_node(declarator, false)?;
                self.print_function_parameters_node(params, true)
            }
        }
    }

    fn print_function_parameters_node(
        &mut self,
        node: &FunctionParametersNode,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            printer.print_function_parameters(&node.param)
        })
    }

    fn print_function_parameters(&mut self, params: &FunctionParameters) -> io::Result<()> {
        match params {
            FunctionParameters::Empty => Ok(()),
            FunctionParameters::OldStyle(names) => {
                for (is_last, name) in names.iter().with_last() {
                    self.print_node(name, is_last, |printer| printer.print_name(name))?;
                }
                Ok(())
            }
            FunctionParameters::Variadic(decls) | FunctionParameters::ParameterTypeList(decls) => {
                for (is_last, decl) in decls.iter().with_last() {
                    self.print_parameter_declaration(decl, is_last)?;
                }
                Ok(())
            }
        }
    }

    fn print_parameter_declaration(
        &mut self,
        node: &ParameterDeclaration,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            for spec in &node.specifiers {
                write!(printer.w, " {}", spec)?;
            }
            printer.print_declarator_node(&node.declarator, true)
        })
    }

    fn print_struct_node(&mut self, node: &Struct, is_last: bool) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            if let Some(name) = &node.name {
                printer.print_name(name)?;
            }
            for (is_last, field) in node.fields.iter().with_last() {
                printer.print_struct_declaration_node(field, is_last)?;
            }
            Ok(())
        })
    }

    fn print_union_node(&mut self, node: &Union, is_last: bool) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            if let Some(name) = &node.name {
                printer.print_name(name)?;
            }
            for (is_last, field) in node.fields.iter().with_last() {
                printer.print_struct_declaration_node(field, is_last)?;
            }
            Ok(())
        })
    }

    fn print_enum_node(&mut self, node: &Enum, is_last: bool) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            if let Some(name) = &node.name {
                printer.print_name(name)?;
            }
            for (is_last, variant_id) in node.variants.iter().with_last() {
                printer
                    .print_variant_node(printer.ctx.arenas.variants.get(*variant_id), is_last)?;
            }
            Ok(())
        })
    }

    fn print_variant_node(&mut self, node: &Variant, is_last: bool) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            printer.print_name(&node.name)?;
            if let Some(value) = &node.value {
                printer.print_expression_node(value, true)?;
            }
            Ok(())
        })
    }

    fn print_struct_declaration_node(
        &mut self,
        node: &StructDeclaration,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            for spec in &node.specifiers {
                write!(printer.w, " {}", spec)?;
            }
            for (is_last, declarator) in node.struct_declarators.iter().with_last() {
                printer.print_struct_declarator_node(declarator, is_last)?;
            }
            Ok(())
        })
    }

    fn print_struct_declarator_node(
        &mut self,
        node: &StructDeclarator,
        is_last: bool,
    ) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            printer.print_declarator_node(&node.declarator, node.bit_width.is_none())?;
            if let Some(bit_width) = &node.bit_width {
                printer.print_expression_node(bit_width, true)?;
            }
            Ok(())
        })
    }

    fn print_initializer_node(&mut self, node: &InitializerNode, is_last: bool) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            printer.print_initializer(&node.init)
        })
    }

    fn print_initializer(&mut self, init: &Initializer) -> io::Result<()> {
        match init {
            Initializer::Single(node) => self.print_expression_node(node, true),
            Initializer::List(nodes) => {
                for (is_last, node) in nodes.iter().with_last() {
                    self.print_initializer_node(node, is_last)?;
                }
                Ok(())
            }
        }
    }

    fn print_expression_node(&mut self, node: &ExpressionNode, is_last: bool) -> io::Result<()> {
        self.print_node(node, is_last, |printer| {
            printer.print_expression(printer.ctx.arenas.expressions.get(node.id))
        })
    }

    fn print_expression(&mut self, expr: &Expression) -> io::Result<()> {
        write!(self.w, "{} ", expr)?;
        match expr {
            Expression::Identifier(c) | Expression::StringLiteral(c) | Expression::Constant(c) => {
                self.print_name(c)
            }
            Expression::ConstantExpression(e)
            | Expression::PostInc(e)
            | Expression::PostDec(e)
            | Expression::PreInc(e)
            | Expression::PreDec(e)
            | Expression::Addr(e)
            | Expression::Deref(e)
            | Expression::Plus(e)
            | Expression::Minus(e)
            | Expression::BitNot(e)
            | Expression::Not(e)
            | Expression::SizeofExpr(e)
            | Expression::FunctionCall(e, None) => self.print_expression_node(e, true),
            Expression::Add(e1, e2)
            | Expression::Sub(e1, e2)
            | Expression::Mul(e1, e2)
            | Expression::Div(e1, e2)
            | Expression::Mod(e1, e2)
            | Expression::Right(e1, e2)
            | Expression::Left(e1, e2)
            | Expression::Greater(e1, e2)
            | Expression::Lower(e1, e2)
            | Expression::GreaterEq(e1, e2)
            | Expression::LowerEq(e1, e2)
            | Expression::Eq(e1, e2)
            | Expression::Neq(e1, e2)
            | Expression::BitAnd(e1, e2)
            | Expression::BitOr(e1, e2)
            | Expression::BitXor(e1, e2)
            | Expression::And(e1, e2)
            | Expression::Or(e1, e2)
            | Expression::Assign(e1, e2)
            | Expression::AddAssign(e1, e2)
            | Expression::SubAssign(e1, e2)
            | Expression::MulAssign(e1, e2)
            | Expression::DivAssign(e1, e2)
            | Expression::ModAssign(e1, e2)
            | Expression::RightAssign(e1, e2)
            | Expression::LeftAssign(e1, e2)
            | Expression::AndAssign(e1, e2)
            | Expression::OrAssign(e1, e2)
            | Expression::XorAssign(e1, e2)
            | Expression::ArrayAcces(e1, e2)
            | Expression::FunctionCall(e1, Some(e2))
            | Expression::List(e1, e2) => {
                self.print_expression_node(e1, false)?;
                self.print_expression_node(e2, true)
            }
            Expression::Ternary(e1, e2, e3) => {
                self.print_expression_node(e1, false)?;
                self.print_expression_node(e2, false)?;
                self.print_expression_node(e3, true)
            }
            Expression::DotAcces(tag, ident) | Expression::PtrAcces(tag, ident) => {
                self.print_expression_node(tag, false)?;
                write!(self.w, " ")?;
                self.print_name(ident)
            }
            Expression::Cast(ty, e1) => {
                self.print_type(ty, false)?;
                self.print_expression_node(e1, true)
            }
            Expression::SizeofType(ty) => self.print_type(ty, true),
        }
    }

    fn print_type(&mut self, ty: &Type, is_last: bool) -> io::Result<()> {
        self.print_node(ty, is_last, |printer| {
            for spec in &ty.specifiers {
                write!(printer.w, " {}", spec)?;
            }
            printer.print_declarator_node(&ty.declarator, true)
        })
    }
}

trait WithLast: Iterator {
    fn with_last(self) -> WithLastIter<Self>
    where
        Self: Sized,
    {
        WithLastIter {
            iter: self.peekable(),
        }
    }
}

impl<I: Iterator> WithLast for I {}

struct WithLastIter<I: Iterator> {
    iter: std::iter::Peekable<I>,
}

impl<I: Iterator> Iterator for WithLastIter<I> {
    type Item = (bool, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next()?;
        let is_last = self.iter.peek().is_none();
        Some((is_last, item))
    }
}
