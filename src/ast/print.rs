use std::fmt::Display;
use std::io::{self, Stdout, Write, stdout};

use crate::ast::visit::{
    Visitor, WithLast, walk_compound_statement, walk_declaration, walk_declarator, walk_enum, walk_expression,
    walk_expression_statement, walk_function_parameters, walk_init_declarator, walk_initializer,
    walk_iteration_statement, walk_jump_statement, walk_labeled_statement, walk_selection_statement, walk_struct,
    walk_struct_declarator, walk_translation_unit, walk_union, walk_variant,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclaratorNode, DeclarationSpecifier, Enum, Expression, ExpressionNode,
    ExpressionStatementNode, FunctionDefinitionNode, FunctionParameters, FunctionParametersNode, InitDeclaratorNode,
    InitializerNode, IterationStatementNode, JumpStatementNode, LabeledStatementNode, Name, ParameterDeclaration,
    Qualifier, SelectionStatementNode, Struct, StructDeclaration, StructDeclarator, TranslationUnitNode, Type,
    TypeSpecifier, Union, Variant,
};
use crate::parser::{Arenas, Context};
use crate::utils::{CYAN, RESET};

pub struct AstPrinter<W: Write> {
    prefix: String,
    w: W,
    err: Option<io::Error>,
}

impl AstPrinter<Stdout> {
    pub fn print(ctx: &Context) -> io::Result<()> {
        Self::write_ast(stdout(), ctx)
    }
}

impl<W: Write> AstPrinter<W> {
    pub fn write_ast(w: W, ctx: &Context) -> io::Result<()> {
        let mut printer = Self {
            prefix: String::new(),
            w,
            err: None,
        };
        printer.visit_translation_unit(&ctx.arenas, &ctx.ast, false);
        let result = writeln!(printer.w);
        printer.ok(result);
        let _ = write!(printer.w, "{RESET}");
        match printer.err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn ok(&mut self, result: io::Result<()>) {
        if let Err(e) = result {
            self.err.get_or_insert(e);
        }
    }

    fn print_node<T, F>(&mut self, node: &T, is_last: bool, f: F)
    where
        T: Display,
        F: FnOnce(&mut Self),
    {
        let (branch, extend) = if is_last { ("`-", "  ") } else { ("|-", "| ") };
        let prefix = &self.prefix;
        let result = write!(self.w, "\n{RESET}{prefix}{branch} {node} ");
        self.prefix.push_str(extend);

        self.ok(result);
        let result = write!(self.w, "{CYAN}");
        self.ok(result);
        f(self);

        self.prefix.pop();
        self.prefix.pop();
    }

    fn print_name_node(&mut self, arenas: &Arenas, name: &Name, is_last: bool) {
        self.print_node(name, is_last, |printer| {
            let result = write!(printer.w, "{}", arenas.names.get(name.id));
            printer.ok(result);
        });
    }

    fn print_specifier(&mut self, arenas: &Arenas, spec: &DeclarationSpecifier) {
        let s = match spec {
            DeclarationSpecifier::Type(t) => match t {
                TypeSpecifier::Struct(id) => match &arenas.structs.get(*id).name {
                    Some(name) => format!("struct {}", arenas.names.get(name.id)),
                    None => "struct".to_string(),
                },
                TypeSpecifier::Union(id) => match &arenas.unions.get(*id).name {
                    Some(name) => format!("union {}", arenas.names.get(name.id)),
                    None => "union".to_string(),
                },
                TypeSpecifier::Enum(id) => match &arenas.enums.get(*id).name {
                    Some(name) => format!("enum {}", arenas.names.get(name.id)),
                    None => "enum".to_string(),
                },
                TypeSpecifier::TypedefName(name) => arenas.names.get(name.id).clone(),
                other => other.to_string(),
            },
            other => other.to_string(),
        };
        let result = write!(self.w, "{s}");
        self.ok(result);
    }
}

impl<W: Write> Visitor for AstPrinter<W> {
    fn visit_translation_unit(&mut self, arenas: &Arenas, node: &TranslationUnitNode, _is_last: bool) {
        let result = write!(self.w, "{}", node);
        self.ok(result);
        walk_translation_unit(self, arenas, node, false);
    }

    fn visit_function_definition(&mut self, arenas: &Arenas, node: &FunctionDefinitionNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            for spec in &node.specifiers {
                printer.print_specifier(arenas, spec);
                let result = write!(printer.w, " ");
                printer.ok(result);
            }
            printer.visit_declarator(arenas, &node.declarator, false);
            printer.visit_compound_statement(arenas, &node.body, true);
        });
    }

    fn visit_declaration(&mut self, arenas: &Arenas, node: &DeclarationNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            for spec in &node.specifiers {
                let result = write!(printer.w, " ");
                printer.ok(result);
                printer.print_specifier(arenas, spec);
            }
            walk_declaration(printer, arenas, node, is_last);
        });
    }

    fn visit_init_declarator(&mut self, arenas: &Arenas, node: &InitDeclaratorNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            walk_init_declarator(printer, arenas, node, is_last);
        });
    }

    fn visit_declarator(&mut self, arenas: &Arenas, node: &DeclaratorNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            let result = write!(printer.w, "{} ", arenas.declarators.get(node.id));
            printer.ok(result);
            walk_declarator(printer, arenas, node, is_last);
        });
    }

    fn visit_initializer(&mut self, arenas: &Arenas, node: &InitializerNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            walk_initializer(printer, arenas, node, is_last);
        });
    }

    fn visit_labeled_statement(&mut self, arenas: &Arenas, node: &LabeledStatementNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            let result = write!(printer.w, "{} ", &node.inner);
            printer.ok(result);
            walk_labeled_statement(printer, arenas, node, is_last);
        });
    }

    fn visit_compound_statement(&mut self, arenas: &Arenas, node: &CompoundStatementNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            walk_compound_statement(printer, arenas, node, is_last);
        });
    }

    fn visit_expression_statement(&mut self, arenas: &Arenas, node: &ExpressionStatementNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            walk_expression_statement(printer, arenas, node, is_last);
        });
    }

    fn visit_selection_statement(&mut self, arenas: &Arenas, node: &SelectionStatementNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            let result = write!(printer.w, "{} ", &node.stmt);
            printer.ok(result);
            walk_selection_statement(printer, arenas, node, is_last);
        });
    }

    fn visit_iteration_statement(&mut self, arenas: &Arenas, node: &IterationStatementNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            let result = write!(printer.w, "{} ", &node.stmt);
            printer.ok(result);
            walk_iteration_statement(printer, arenas, node, is_last);
        });
    }

    fn visit_jump_statement(&mut self, arenas: &Arenas, node: &JumpStatementNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            let result = write!(printer.w, "{} ", &node.stmt);
            printer.ok(result);
            walk_jump_statement(printer, arenas, node, is_last);
        });
    }

    fn visit_expression(&mut self, arenas: &Arenas, node: &ExpressionNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            let expr = arenas.expressions.get(node.id);
            let result = write!(printer.w, "{} ", expr);
            printer.ok(result);
            if let Expression::DotAcces(tag, ident) | Expression::PtrAcces(tag, ident) = expr {
                printer.visit_expression(arenas, tag, false);
                let result = write!(printer.w, " ");
                printer.ok(result);
                printer.visit_name(arenas, ident);
            } else {
                walk_expression(printer, arenas, node, is_last);
            }
        });
    }

    fn visit_type(&mut self, arenas: &Arenas, node: &Type, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            for spec in &node.specifiers {
                printer.print_specifier(arenas, spec);
            }
            printer.visit_declarator(arenas, &node.declarator, true);
        });
    }

    fn visit_function_parameters(&mut self, arenas: &Arenas, node: &FunctionParametersNode, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            if let FunctionParameters::OldStyle(names) = &node.param {
                for (is_last, name) in names.iter().with_last() {
                    printer.print_name_node(arenas, name, is_last);
                }
            } else {
                walk_function_parameters(printer, arenas, node, is_last);
            }
        });
    }

    fn visit_parameter_declaration(&mut self, arenas: &Arenas, node: &ParameterDeclaration, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            for spec in &node.specifiers {
                let result = write!(printer.w, " ");
                printer.ok(result);
                printer.print_specifier(arenas, spec);
            }
            printer.visit_declarator(arenas, &node.declarator, true);
        });
    }

    fn visit_struct(&mut self, arenas: &Arenas, node: &Struct, is_last: bool) {
        self.print_node(node, is_last, |printer| walk_struct(printer, arenas, node, is_last));
    }

    fn visit_union(&mut self, arenas: &Arenas, node: &Union, is_last: bool) {
        self.print_node(node, is_last, |printer| walk_union(printer, arenas, node, is_last));
    }

    fn visit_enum(&mut self, arenas: &Arenas, node: &Enum, is_last: bool) {
        self.print_node(node, is_last, |printer| walk_enum(printer, arenas, node, is_last));
    }

    fn visit_variant(&mut self, arenas: &Arenas, node: &Variant, is_last: bool) {
        self.print_node(node, is_last, |printer| walk_variant(printer, arenas, node, is_last));
    }

    fn visit_struct_declaration(&mut self, arenas: &Arenas, node: &StructDeclaration, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            for spec in &node.specifiers {
                let result = write!(printer.w, " ");
                printer.ok(result);
                printer.print_specifier(arenas, spec);
            }
            for (is_last, declarator) in node.struct_declarators.iter().with_last() {
                printer.visit_struct_declarator(arenas, declarator, is_last);
            }
        });
    }

    fn visit_struct_declarator(&mut self, arenas: &Arenas, node: &StructDeclarator, is_last: bool) {
        self.print_node(node, is_last, |printer| {
            walk_struct_declarator(printer, arenas, node, is_last);
        });
    }

    fn visit_qualifier(&mut self, _arenas: &Arenas, qualifier: &Qualifier) {
        let result = write!(self.w, " {}", qualifier);
        self.ok(result);
    }

    fn visit_name(&mut self, arenas: &Arenas, node: &Name) {
        let result = write!(self.w, "{}", arenas.names.get(node.id));
        self.ok(result);
    }
}
