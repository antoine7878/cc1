use std::ops::Deref;

use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, Enum, Expression,
    ExpressionNode, ExpressionStatementNode, ExternalDeclaration, ExternalDeclarationNode, FunctionDefinitionNode,
    FunctionParameters, FunctionParametersNode, InitDeclaratorNode, Initializer, InitializerNode, IterationStatement,
    IterationStatementNode, JumpStatement, JumpStatementNode, Labeled, LabeledStatementNode, Name,
    ParameterDeclaration, Qualifier, SelectionStatement, SelectionStatementNode, Statement, StatementNode,
    StringLiteralNode, Struct, StructDeclaration, StructMemberDeclarator, TranslationUnitNode, Type, TypeSpecifier,
    Union, ValueNode, Variant,
};
use crate::context::Context;

pub trait Visitor {
    fn visit_translation_unit(&mut self, ctx: &Context, node: &TranslationUnitNode) {
        walk_translation_unit(self, ctx, node);
    }

    fn visit_external_declaration(&mut self, ctx: &Context, node: &ExternalDeclarationNode) {
        walk_external_declaration(self, ctx, node);
    }

    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        walk_function_definition(self, ctx, node);
    }

    fn visit_declaration(&mut self, ctx: &Context, node: &DeclarationNode) {
        walk_declaration(self, ctx, node);
    }

    fn visit_init_declarator(&mut self, ctx: &Context, node: &InitDeclaratorNode) {
        walk_init_declarator(self, ctx, node);
    }

    fn visit_declarator(&mut self, ctx: &Context, node: &DeclaratorNode) {
        walk_declarator(self, ctx, node);
    }

    fn visit_initializer(&mut self, ctx: &Context, node: &InitializerNode) {
        walk_initializer(self, ctx, node);
    }

    fn visit_statement(&mut self, ctx: &Context, node: &StatementNode) {
        walk_statement(self, ctx, node);
    }

    fn visit_labeled_statement(&mut self, ctx: &Context, node: &LabeledStatementNode) {
        walk_labeled_statement(self, ctx, node);
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode) {
        walk_compound_statement(self, ctx, node);
    }

    fn visit_expression_statement(&mut self, ctx: &Context, node: &ExpressionStatementNode) {
        walk_expression_statement(self, ctx, node);
    }

    fn visit_selection_statement(&mut self, ctx: &Context, node: &SelectionStatementNode) {
        walk_selection_statement(self, ctx, node);
    }

    fn visit_iteration_statement(&mut self, ctx: &Context, node: &IterationStatementNode) {
        walk_iteration_statement(self, ctx, node);
    }

    fn visit_jump_statement(&mut self, ctx: &Context, node: &JumpStatementNode) {
        walk_jump_statement(self, ctx, node);
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        walk_expression(self, ctx, node);
    }

    fn visit_type(&mut self, ctx: &Context, node: &Type) {
        walk_type(self, ctx, node);
    }

    fn visit_function_parameters(&mut self, ctx: &Context, node: &FunctionParametersNode) {
        walk_function_parameters(self, ctx, node);
    }

    fn visit_parameter_declaration(&mut self, ctx: &Context, node: &ParameterDeclaration) {
        walk_parameter_declaration(self, ctx, node);
    }

    fn visit_struct(&mut self, ctx: &Context, node: &Struct) {
        walk_struct(self, ctx, node);
    }

    fn visit_union(&mut self, ctx: &Context, node: &Union) {
        walk_union(self, ctx, node);
    }

    fn visit_enum(&mut self, ctx: &Context, node: &Enum) {
        walk_enum(self, ctx, node);
    }

    fn visit_variant(&mut self, ctx: &Context, node: &Variant) {
        walk_variant(self, ctx, node);
    }

    fn visit_struct_declaration(&mut self, ctx: &Context, node: &StructDeclaration) {
        walk_struct_declaration(self, ctx, node);
    }

    fn visit_struct_declarator(&mut self, ctx: &Context, node: &StructMemberDeclarator) {
        walk_struct_declarator(self, ctx, node);
    }

    fn visit_qualifier(&mut self, _ctx: &Context, _qualifier: &Qualifier) {}

    fn visit_name(&mut self, _ctx: &Context, _node: &Name) {}

    fn visit_value(&mut self, _ctx: &Context, _node: &ValueNode) {}

    fn visit_string_literal(&mut self, ctx: &Context, node: &StringLiteralNode) {
        walk_string_literal(self, ctx, node);
    }
}

pub fn walk_string_literal<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &StringLiteralNode) {
    v.visit_name(ctx, &node.name());
}

pub fn walk_translation_unit<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &TranslationUnitNode) {
    for decl in &node.declarations {
        v.visit_external_declaration(ctx, decl);
    }
}

pub fn walk_external_declaration<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &ExternalDeclarationNode) {
    match &node.decl {
        ExternalDeclaration::Function(function) => v.visit_function_definition(ctx, function),
        ExternalDeclaration::Declaration(decl) => v.visit_declaration(ctx, decl),
    }
}

pub fn walk_function_definition<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &FunctionDefinitionNode) {
    walk_specifiers(v, ctx, &node.specifiers);
    v.visit_declarator(ctx, &node.declarator);
    for decl in &node.old_style_declarations {
        v.visit_declaration(ctx, decl);
    }
    v.visit_compound_statement(ctx, &node.body);
}

pub fn walk_declaration<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &DeclarationNode) {
    walk_declaration_specifier(v, ctx, node);
    for init in &node.init_declarators {
        v.visit_init_declarator(ctx, init);
    }
}

pub fn walk_declaration_specifier<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &DeclarationNode) {
    walk_specifiers(v, ctx, &node.specifiers);
}

pub fn walk_init_declarator<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &InitDeclaratorNode) {
    v.visit_declarator(ctx, &node.declarator);
    if let Some(init) = &node.initializer {
        v.visit_initializer(ctx, init);
    }
}

pub fn walk_declarator<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &DeclaratorNode) {
    match node.id.resolve(ctx) {
        Declarator::Ident(name) => v.visit_name(ctx, name),
        Declarator::Abstract => {}
        Declarator::Pointer { qualifiers, inner } => {
            for qualifier in qualifiers {
                v.visit_qualifier(ctx, qualifier);
            }
            v.visit_declarator(ctx, inner);
        }
        Declarator::Array { declarator, size } => {
            v.visit_declarator(ctx, declarator);
            if let Some(size) = size {
                v.visit_expression(ctx, size);
            }
        }
        Declarator::Function { declarator, params } => {
            v.visit_declarator(ctx, declarator);
            v.visit_function_parameters(ctx, params);
        }
    }
}

pub fn walk_initializer<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &InitializerNode) {
    match &node.init {
        Initializer::Single(expr) => v.visit_expression(ctx, expr),
        Initializer::List(nodes) => {
            for init in nodes {
                v.visit_initializer(ctx, init);
            }
        }
    }
}

pub fn walk_statement<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &StatementNode) {
    match node.id.resolve(ctx) {
        Statement::Labeled(node) => v.visit_labeled_statement(ctx, node),
        Statement::Compound(node) => v.visit_compound_statement(ctx, node),
        Statement::Expression(node) => v.visit_expression_statement(ctx, node),
        Statement::Selection(node) => v.visit_selection_statement(ctx, node),
        Statement::Iteration(node) => v.visit_iteration_statement(ctx, node),
        Statement::Jump(node) => v.visit_jump_statement(ctx, node),
    }
}

pub fn walk_labeled_statement<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &LabeledStatementNode) {
    match &node.inner {
        Labeled::Identifier(name, stmt) => {
            v.visit_name(ctx, name);
            v.visit_statement(ctx, stmt);
        }
        Labeled::Case(expr, stmt) => {
            v.visit_expression(ctx, expr);
            v.visit_statement(ctx, stmt);
        }
        Labeled::Default(stmt) => v.visit_statement(ctx, stmt),
    }
}

pub fn walk_compound_statement<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &CompoundStatementNode) {
    for decl in &node.declarations {
        v.visit_declaration(ctx, decl);
    }
    for stmt in &node.statements {
        v.visit_statement(ctx, stmt);
    }
}

pub fn walk_expression_statement<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &ExpressionStatementNode) {
    if let Some(expr) = &node.expr {
        v.visit_expression(ctx, expr);
    }
}

pub fn walk_selection_statement<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &SelectionStatementNode) {
    match &node.stmt {
        SelectionStatement::If(cond, then, otherwise) => {
            v.visit_expression(ctx, cond);
            v.visit_statement(ctx, then);
            if let Some(otherwise) = otherwise {
                v.visit_statement(ctx, otherwise);
            }
        }
        SelectionStatement::Switch(cond, stmt) => {
            v.visit_expression(ctx, cond);
            v.visit_statement(ctx, stmt);
        }
    }
}

pub fn walk_iteration_statement<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &IterationStatementNode) {
    match &node.stmt {
        IterationStatement::While(cond, body) => {
            v.visit_expression(ctx, cond);
            v.visit_statement(ctx, body);
        }
        IterationStatement::Do(body, cond) => {
            v.visit_statement(ctx, body);
            v.visit_expression(ctx, cond);
        }
        IterationStatement::For(b) => {
            let (init, cond, inc, body) = b.deref();
            v.visit_expression_statement(ctx, init);
            v.visit_expression_statement(ctx, cond);
            if let Some(inc) = inc {
                v.visit_expression(ctx, inc);
            }
            v.visit_statement(ctx, body);
        }
    }
}

pub fn walk_jump_statement<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &JumpStatementNode) {
    if let JumpStatement::Return(Some(expr)) = &node.stmt {
        v.visit_expression(ctx, expr);
    }
}

pub fn walk_expression<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &ExpressionNode) {
    match node.id.resolve(ctx) {
        Expression::Identifier(name) => v.visit_name(ctx, name),
        Expression::StringLiteral(literal) => v.visit_string_literal(ctx, literal),
        Expression::Constant(value) => v.visit_value(ctx, value),
        Expression::ConstantExpression(expr) | Expression::Unary(_, expr) | Expression::SizeofExpr(expr) => {
            v.visit_expression(ctx, expr)
        }
        Expression::Binary(_, lhs, rhs) | Expression::Assign(_, lhs, rhs) | Expression::ArraySubscripting(lhs, rhs) => {
            v.visit_expression(ctx, lhs);
            v.visit_expression(ctx, rhs);
        }
        Expression::FunctionCall(lhs, args) => {
            v.visit_expression(ctx, lhs);
            for arg in args {
                v.visit_expression(ctx, arg);
            }
        }
        Expression::Ternary(cond, then, otherwise) => {
            v.visit_expression(ctx, cond);
            v.visit_expression(ctx, then);
            v.visit_expression(ctx, otherwise);
        }
        Expression::Member(_, tag, ident) => {
            v.visit_expression(ctx, tag);
            v.visit_name(ctx, ident);
        }
        Expression::Cast(ty, expr) => {
            v.visit_type(ctx, ty);
            v.visit_expression(ctx, expr);
        }
        Expression::SizeofType(ty) => v.visit_type(ctx, ty),
        Expression::List(exps) => {
            for e in exps {
                v.visit_expression(ctx, e);
            }
        }
    }
}

pub fn walk_type<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &Type) {
    walk_specifiers(v, ctx, &node.specifiers);
    v.visit_declarator(ctx, &node.declarator);
}

pub fn walk_function_parameters<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &FunctionParametersNode) {
    match &node.param {
        FunctionParameters::Empty => {}
        FunctionParameters::OldStyle(names) => {
            for name in names {
                v.visit_name(ctx, name);
            }
        }
        FunctionParameters::ParameterTypeList(decls) | FunctionParameters::Variadic(decls) => {
            for decl in decls {
                v.visit_parameter_declaration(ctx, decl);
            }
        }
    }
}

pub fn walk_parameter_declaration<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &ParameterDeclaration) {
    walk_specifiers(v, ctx, &node.specifiers);
    v.visit_declarator(ctx, &node.declarator);
}

pub fn walk_struct<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &Struct) {
    if let Some(name) = &node.name {
        v.visit_name(ctx, name);
    }
    for field in &node.fields {
        v.visit_struct_declaration(ctx, field);
    }
}

pub fn walk_union<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &Union) {
    if let Some(name) = &node.name {
        v.visit_name(ctx, name);
    }
    for field in &node.fields {
        v.visit_struct_declaration(ctx, field);
    }
}

pub fn walk_enum<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &Enum) {
    if let Some(name) = &node.name {
        v.visit_name(ctx, name);
    }
    for variant_id in &node.variants {
        v.visit_variant(ctx, variant_id.resolve(ctx));
    }
}

pub fn walk_variant<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &Variant) {
    v.visit_name(ctx, &node.name);
    if let Some(value) = &node.value {
        v.visit_expression(ctx, value);
    }
}

pub fn walk_struct_declaration<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &StructDeclaration) {
    walk_specifiers(v, ctx, &node.specifiers);
    for declarator in &node.struct_declarators {
        v.visit_struct_declarator(ctx, declarator);
    }
}

pub fn walk_struct_declarator<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &StructMemberDeclarator) {
    v.visit_declarator(ctx, &node.declarator);
    if let Some(bit_width) = &node.bit_width {
        v.visit_expression(ctx, bit_width);
    }
}

pub fn walk_specifiers<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, specifiers: &[DeclarationSpecifier]) {
    for spec in specifiers {
        if let DeclarationSpecifier::Type(type_specifier) = spec {
            match type_specifier {
                TypeSpecifier::Struct(id) => v.visit_struct(ctx, id.resolve(ctx)),
                TypeSpecifier::Union(id) => v.visit_union(ctx, id.resolve(ctx)),
                TypeSpecifier::Enum(id) => v.visit_enum(ctx, id.resolve(ctx)),
                TypeSpecifier::TypedefName(name) => v.visit_name(ctx, name),
                _ => {}
            }
        }
    }
}
