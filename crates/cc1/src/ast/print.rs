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
use crate::semantic::{ExpressionKind, sema};
use libft::{CYAN, GRAY, GREEN, RESET};

pub struct AstPrinter {
    depth: usize,
    lines: Vec<Line>,
    open: Vec<usize>,
}

struct Line {
    depth: usize,
    text: String,
    children: usize,
}

impl AstPrinter {
    pub fn print(ctx: &Context) {
        let _ = Self::write_ast(stdout(), ctx);
    }

    pub fn write_ast<W: Write>(mut w: W, ctx: &Context) -> io::Result<()> {
        let mut printer = Self {
            depth: 0,
            lines: Vec::new(),
            open: Vec::new(),
        };
        printer.visit_translation_unit(ctx, &ctx.ast);
        printer.render(&mut w)
    }

    fn put(&mut self, args: std::fmt::Arguments) {
        use std::fmt::Write;
        let _ = self.lines.last_mut().expect("root line").text.write_fmt(args);
    }

    fn print_node<T, F>(&mut self, node: &T, f: F)
    where
        T: Display,
        F: FnOnce(&mut Self),
    {
        let idx = self.lines.len();
        self.lines.push(Line {
            depth: self.depth,
            text: format!("{node} {CYAN}"),
            children: 0,
        });
        if let Some(siblings) = self.open.last_mut() {
            *siblings += 1;
        }
        self.open.push(0);
        self.depth += 1;
        f(self);
        self.depth -= 1;
        self.lines[idx].children = self.open.pop().expect("open node");
    }

    fn print_name_node(&mut self, ctx: &Context, name: &Name) {
        self.print_node(name, |printer| {
            printer.put(format_args!("{}", name.id.resolve()));
        });
    }

    fn print_expression_type(&mut self, ctx: &Context, id: ExpressionId) {
        let sema = sema();
        let Some(resolved) = sema.expr_types.get(id) else { return };
        self.put(format_args!("{GREEN}'{}'{CYAN}", resolved.ty.describe(sema, ctx)));
        if matches!(resolved.kind, ExpressionKind::LValue) {
            self.put(format_args!(" lvalue"));
        }
        for cast in &resolved.casts {
            self.put(format_args!(
                " {GRAY}{}{CYAN} {GREEN}'{}'{CYAN}",
                cast.kind,
                cast.to.describe(sema, ctx)
            ));
        }
        self.put(format_args!(" "));
    }

    fn print_specifier(&mut self, ctx: &Context, spec: &DeclarationSpecifier) {
        let tagged = |keyword: &str, name: Option<&Name>| match name {
            Some(name) => format!("{keyword} {}", name.id.resolve()),
            None => keyword.to_string(),
        };
        let s = match spec {
            DeclarationSpecifier::Type(t) => match t {
                TypeSpecifier::Struct(id) => tagged("struct", id.resolve().name.as_ref()),
                TypeSpecifier::Union(id) => tagged("union", id.resolve().name.as_ref()),
                TypeSpecifier::Enum(id) => tagged("enum", id.resolve().name.as_ref()),
                TypeSpecifier::TypedefName(name) => name.id.resolve().clone(),
                other => other.to_string(),
            },
            other => other.to_string(),
        };
        self.put(format_args!("{s}"));
    }

    fn render<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let mut remaining: Vec<usize> = Vec::new();
        let mut ancestors: Vec<bool> = Vec::new();
        for line in &self.lines {
            let depth = line.depth;
            if depth == 0 {
                write!(w, "{}", line.text)?;
                remaining.clear();
                remaining.push(line.children);
                continue;
            }
            remaining.truncate(depth);
            remaining[depth - 1] -= 1;
            let last = remaining[depth - 1] == 0;
            ancestors.resize(depth, false);
            ancestors[depth - 1] = last;
            write!(w, "\n{RESET}")?;
            for level in &ancestors[..depth - 1] {
                write!(w, "{}", if *level { "  " } else { "| " })?;
            }
            write!(w, "{} {}", if last { "`-" } else { "|-" }, line.text)?;
            remaining.push(line.children);
        }
        write!(w, "{RESET}")?;
        writeln!(w)
    }
}

impl Visitor for AstPrinter {
    fn visit_translation_unit(&mut self, ctx: &Context, node: &TranslationUnitNode) {
        let idx = self.lines.len();
        self.lines.push(Line {
            depth: 0,
            text: format!("{node}"),
            children: 0,
        });
        self.open.push(0);
        self.depth += 1;
        walk_translation_unit(self, ctx, node);
        self.depth -= 1;
        self.lines[idx].children = self.open.pop().expect("root node");
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
            printer.put(format_args!("{} ", node.id.resolve()));
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
            let expr = node.id.resolve();
            printer.put(format_args!("{} ", expr));
            printer.print_expression_type(ctx, node.id);
            if let Expression::Member(op, tag, ident) = expr {
                printer.visit_expression(ctx, tag);
                printer.put(format_args!("{}", op.symbol()));
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
        self.put(format_args!("{}", node.id.resolve()));
    }
}
