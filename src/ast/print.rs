use std::fmt::Display;
use std::io::{self, Write, stdout};

use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_declaration, walk_declarator, walk_enum, walk_expression,
    walk_expression_statement, walk_function_parameters, walk_init_declarator, walk_initializer,
    walk_iteration_statement, walk_jump_statement, walk_labeled_statement, walk_selection_statement, walk_struct,
    walk_struct_declarator, walk_translation_unit, walk_union, walk_variant,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, DeclaratorNode, Enum, Expression, ExpressionId,
    ExpressionNode, ExpressionStatementNode, FunctionDefinitionNode, FunctionParameters, FunctionParametersNode,
    InitDeclaratorNode, InitializerNode, IterationStatementNode, JumpStatementNode, LabeledStatementNode, Name,
    ParameterDeclaration, Qualifier, SelectionStatementNode, Struct, StructDeclaration, StructMemberDeclarator,
    TranslationUnitNode, Type, TypeSpecifier, Union, Variant,
};
use crate::context::Context;
use crate::semantic::ExpressionKind;
use crate::utils::{CYAN, GRAY, GREEN, RESET};

pub struct AstPrinter {
    depth: usize,
    lines: Vec<(usize, String)>,
}

impl AstPrinter {
    pub fn print(ctx: &Context) {
        let _ = Self::write_ast(stdout(), ctx);
    }

    pub fn write_ast<W: Write>(mut w: W, ctx: &Context) -> io::Result<()> {
        let mut printer = Self {
            depth: 0,
            lines: Vec::new(),
        };
        printer.visit_translation_unit(ctx, &ctx.ast);
        printer.render(&mut w)
    }

    fn put(&mut self, args: std::fmt::Arguments) {
        use std::fmt::Write;
        let _ = self.lines.last_mut().expect("root line").1.write_fmt(args);
    }

    fn print_node<T, F>(&mut self, node: &T, f: F)
    where
        T: Display,
        F: FnOnce(&mut Self),
    {
        self.lines.push((self.depth, format!("{node} {CYAN}")));
        self.depth += 1;
        f(self);
        self.depth -= 1;
    }

    fn print_name_node(&mut self, ctx: &Context, name: &Name) {
        self.print_node(name, |printer| {
            printer.put(format_args!("{}", name.id.resolve(ctx)));
        });
    }

    fn print_expression_type(&mut self, ctx: &Context, id: ExpressionId) {
        let Some(resolved) = ctx.sema.expr_resolved(id) else { return };
        self.put(format_args!("{GREEN}'{}'{CYAN}", resolved.ty.describe(&ctx.sema, ctx)));
        if matches!(resolved.kind, ExpressionKind::LValue) {
            self.put(format_args!(" lvalue"));
        }
        for cast in &resolved.casts {
            self.put(format_args!(
                " {GRAY}{}{CYAN} {GREEN}'{}'{CYAN}",
                cast.kind,
                cast.to.describe(&ctx.sema, ctx)
            ));
        }
        self.put(format_args!(" "));
    }

    fn print_specifier(&mut self, ctx: &Context, spec: &DeclarationSpecifier) {
        let s = match spec {
            DeclarationSpecifier::Type(t) => match t {
                TypeSpecifier::Struct(id) => match &id.resolve(ctx).name {
                    Some(name) => format!("struct {}", name.id.resolve(ctx)),
                    None => "struct".to_string(),
                },
                TypeSpecifier::Union(id) => match &id.resolve(ctx).name {
                    Some(name) => format!("union {}", name.id.resolve(ctx)),
                    None => "union".to_string(),
                },
                TypeSpecifier::Enum(id) => match &id.resolve(ctx).name {
                    Some(name) => format!("enum {}", name.id.resolve(ctx)),
                    None => "enum".to_string(),
                },
                TypeSpecifier::TypedefName(name) => name.id.resolve(ctx).clone(),
                other => other.to_string(),
            },
            other => other.to_string(),
        };
        self.put(format_args!("{s}"));
    }

    fn render<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let mut last = vec![false; self.lines.len()];
        let mut seen: Vec<bool> = Vec::new();
        for (i, (depth, _)) in self.lines.iter().enumerate().rev() {
            seen.resize(depth + 1, false);
            last[i] = !seen[*depth];
            seen[*depth] = true;
        }

        let mut ancestors: Vec<bool> = Vec::new();
        for (i, (depth, text)) in self.lines.iter().enumerate() {
            if *depth == 0 {
                write!(w, "{text}")?;
                continue;
            }
            ancestors.resize(*depth, false);
            ancestors[*depth - 1] = last[i];
            let branch = if last[i] { "`-" } else { "|-" };
            write!(w, "\n{RESET}")?;
            for level in &ancestors[..*depth - 1] {
                write!(w, "{}", if *level { "  " } else { "| " })?;
            }
            write!(w, "{branch} {text}")?;
        }
        write!(w, "{RESET}")?;
        writeln!(w)
    }
}

impl Visitor for AstPrinter {
    fn visit_translation_unit(&mut self, ctx: &Context, node: &TranslationUnitNode) {
        self.lines.push((0, format!("{node}")));
        self.depth += 1;
        walk_translation_unit(self, ctx, node);
        self.depth -= 1;
    }

    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        self.print_node(node, |printer| {
            for spec in &node.specifiers {
                printer.print_specifier(ctx, spec);
                printer.put(format_args!(" "));
            }
            printer.visit_declarator(ctx, &node.declarator);
            printer.visit_compound_statement(ctx, &node.body);
        });
    }

    fn visit_declaration(&mut self, ctx: &Context, node: &DeclarationNode) {
        self.print_node(node, |printer| {
            for spec in &node.specifiers {
                printer.put(format_args!(" "));
                printer.print_specifier(ctx, spec);
            }
            walk_declaration(printer, ctx, node);
        });
    }

    fn visit_init_declarator(&mut self, ctx: &Context, node: &InitDeclaratorNode) {
        self.print_node(node, |printer| {
            walk_init_declarator(printer, ctx, node);
        });
    }

    fn visit_declarator(&mut self, ctx: &Context, node: &DeclaratorNode) {
        self.print_node(node, |printer| {
            printer.put(format_args!("{} ", node.id.resolve(ctx)));
            walk_declarator(printer, ctx, node);
        });
    }

    fn visit_initializer(&mut self, ctx: &Context, node: &InitializerNode) {
        self.print_node(node, |printer| {
            walk_initializer(printer, ctx, node);
        });
    }

    fn visit_labeled_statement(&mut self, ctx: &Context, node: &LabeledStatementNode) {
        self.print_node(node, |printer| {
            printer.put(format_args!("{} ", node.inner));
            walk_labeled_statement(printer, ctx, node);
        });
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode) {
        self.print_node(node, |printer| {
            walk_compound_statement(printer, ctx, node);
        });
    }

    fn visit_expression_statement(&mut self, ctx: &Context, node: &ExpressionStatementNode) {
        self.print_node(node, |printer| {
            walk_expression_statement(printer, ctx, node);
        });
    }

    fn visit_selection_statement(&mut self, ctx: &Context, node: &SelectionStatementNode) {
        self.print_node(node, |printer| {
            printer.put(format_args!("{} ", node.stmt));
            walk_selection_statement(printer, ctx, node);
        });
    }

    fn visit_iteration_statement(&mut self, ctx: &Context, node: &IterationStatementNode) {
        self.print_node(node, |printer| {
            printer.put(format_args!("{} ", node.stmt));
            walk_iteration_statement(printer, ctx, node);
        });
    }

    fn visit_jump_statement(&mut self, ctx: &Context, node: &JumpStatementNode) {
        self.print_node(node, |printer| {
            printer.put(format_args!("{} ", node.stmt));
            walk_jump_statement(printer, ctx, node);
        });
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        self.print_node(node, |printer| {
            let expr = node.id.resolve(ctx);
            printer.put(format_args!("{} ", expr));
            printer.print_expression_type(ctx, node.id);
            if let Expression::DotAcces(tag, ident) | Expression::PtrAcces(tag, ident) = expr {
                printer.visit_expression(ctx, tag);
                printer.put(format_args!(" "));
                printer.visit_name(ctx, ident);
            } else {
                walk_expression(printer, ctx, node);
            }
        });
    }

    fn visit_type(&mut self, ctx: &Context, node: &Type) {
        self.print_node(node, |printer| {
            for spec in &node.specifiers {
                printer.print_specifier(ctx, spec);
            }
            printer.visit_declarator(ctx, &node.declarator);
        });
    }

    fn visit_function_parameters(&mut self, ctx: &Context, node: &FunctionParametersNode) {
        self.print_node(node, |printer| {
            if let FunctionParameters::OldStyle(names) = &node.param {
                for name in names {
                    printer.print_name_node(ctx, name);
                }
            } else {
                walk_function_parameters(printer, ctx, node);
            }
        });
    }

    fn visit_parameter_declaration(&mut self, ctx: &Context, node: &ParameterDeclaration) {
        self.print_node(node, |printer| {
            for spec in &node.specifiers {
                printer.put(format_args!(" "));
                printer.print_specifier(ctx, spec);
            }
            printer.visit_declarator(ctx, &node.declarator);
        });
    }

    fn visit_struct(&mut self, ctx: &Context, node: &Struct) {
        self.print_node(node, |printer| walk_struct(printer, ctx, node));
    }

    fn visit_union(&mut self, ctx: &Context, node: &Union) {
        self.print_node(node, |printer| walk_union(printer, ctx, node));
    }

    fn visit_enum(&mut self, ctx: &Context, node: &Enum) {
        self.print_node(node, |printer| walk_enum(printer, ctx, node));
    }

    fn visit_variant(&mut self, ctx: &Context, node: &Variant) {
        self.print_node(node, |printer| walk_variant(printer, ctx, node));
    }

    fn visit_struct_declaration(&mut self, ctx: &Context, node: &StructDeclaration) {
        self.print_node(node, |printer| {
            for spec in &node.specifiers {
                printer.put(format_args!(" "));
                printer.print_specifier(ctx, spec);
            }
            for declarator in &node.struct_declarators {
                printer.visit_struct_declarator(ctx, declarator);
            }
        });
    }

    fn visit_struct_declarator(&mut self, ctx: &Context, node: &StructMemberDeclarator) {
        self.print_node(node, |printer| {
            walk_struct_declarator(printer, ctx, node);
        });
    }

    fn visit_qualifier(&mut self, _ctx: &Context, qualifier: &Qualifier) {
        self.put(format_args!(" {}", qualifier));
    }

    fn visit_name(&mut self, ctx: &Context, node: &Name) {
        self.put(format_args!("{}", node.id.resolve(ctx)));
    }
}
