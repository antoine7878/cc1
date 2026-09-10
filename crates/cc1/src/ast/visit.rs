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

pub trait Visitor {
    fn visit_translation_unit(&mut self, node: &TranslationUnitNode) {
        walk_translation_unit(self, node);
    }

    fn visit_external_declaration(&mut self, node: &ExternalDeclarationNode) {
        walk_external_declaration(self, node);
    }

    fn visit_function_definition(&mut self, node: &FunctionDefinitionNode) {
        walk_function_definition(self, node);
    }

    fn visit_declaration(&mut self, node: &DeclarationNode) {
        walk_declaration(self, node);
    }

    fn visit_init_declarator(&mut self, node: &InitDeclaratorNode) {
        walk_init_declarator(self, node);
    }

    fn visit_declarator(&mut self, node: &DeclaratorNode) {
        walk_declarator(self, node);
    }

    fn visit_initializer(&mut self, node: &InitializerNode) {
        walk_initializer(self, node);
    }

    fn visit_statement(&mut self, node: &StatementNode) {
        walk_statement(self, node);
    }

    fn visit_labeled_statement(&mut self, node: &LabeledStatementNode) {
        walk_labeled_statement(self, node);
    }

    fn visit_compound_statement(&mut self, node: &CompoundStatementNode) {
        walk_compound_statement(self, node);
    }

    fn visit_expression_statement(&mut self, node: &ExpressionStatementNode) {
        walk_expression_statement(self, node);
    }

    fn visit_selection_statement(&mut self, node: &SelectionStatementNode) {
        walk_selection_statement(self, node);
    }

    fn visit_iteration_statement(&mut self, node: &IterationStatementNode) {
        walk_iteration_statement(self, node);
    }

    fn visit_jump_statement(&mut self, node: &JumpStatementNode) {
        walk_jump_statement(self, node);
    }

    fn visit_expression(&mut self, node: &ExpressionNode) {
        walk_expression(self, node);
    }

    fn visit_type(&mut self, node: &Type) {
        walk_type(self, node);
    }

    fn visit_function_parameters(&mut self, node: &FunctionParametersNode) {
        walk_function_parameters(self, node);
    }

    fn visit_parameter_declaration(&mut self, node: &ParameterDeclaration) {
        walk_parameter_declaration(self, node);
    }

    fn visit_struct(&mut self, node: &Struct) {
        walk_struct(self, node);
    }

    fn visit_union(&mut self, node: &Union) {
        walk_union(self, node);
    }

    fn visit_enum(&mut self, node: &Enum) {
        walk_enum(self, node);
    }

    fn visit_variant(&mut self, node: &Variant) {
        walk_variant(self, node);
    }

    fn visit_struct_declaration(&mut self, node: &StructDeclaration) {
        walk_struct_declaration(self, node);
    }

    fn visit_struct_declarator(&mut self, node: &StructMemberDeclarator) {
        walk_struct_declarator(self, node);
    }

    fn visit_qualifier(&mut self, _qualifier: &Qualifier) {}

    fn visit_name(&mut self, _node: &Name) {}

    fn visit_value(&mut self, _node: &ValueNode) {}

    fn visit_string_literal(&mut self, node: &StringLiteralNode) {
        walk_string_literal(self, node);
    }
}

pub fn walk_string_literal<V: Visitor + ?Sized>(_v: &mut V, _node: &StringLiteralNode) {}

pub fn walk_translation_unit<V: Visitor + ?Sized>(v: &mut V, node: &TranslationUnitNode) {
    for decl in &node.declarations {
        v.visit_external_declaration(decl);
    }
}

pub fn walk_external_declaration<V: Visitor + ?Sized>(v: &mut V, node: &ExternalDeclarationNode) {
    match &node.decl {
        ExternalDeclaration::Function(function) => v.visit_function_definition(function),
        ExternalDeclaration::Declaration(decl) => v.visit_declaration(decl),
    }
}

pub fn walk_function_definition<V: Visitor + ?Sized>(v: &mut V, node: &FunctionDefinitionNode) {
    walk_specifiers(v, &node.specifiers);
    v.visit_declarator(&node.declarator);
    for decl in &node.old_style_declarations {
        v.visit_declaration(decl);
    }
    v.visit_compound_statement(&node.body);
}

pub fn walk_declaration<V: Visitor + ?Sized>(v: &mut V, node: &DeclarationNode) {
    walk_declaration_specifier(v, node);
    for init in &node.init_declarators {
        v.visit_init_declarator(init);
    }
}

pub fn walk_declaration_specifier<V: Visitor + ?Sized>(v: &mut V, node: &DeclarationNode) {
    walk_specifiers(v, &node.specifiers);
}

pub fn walk_init_declarator<V: Visitor + ?Sized>(v: &mut V, node: &InitDeclaratorNode) {
    v.visit_declarator(&node.declarator);
    if let Some(init) = &node.initializer {
        v.visit_initializer(init);
    }
}

pub fn walk_declarator<V: Visitor + ?Sized>(v: &mut V, node: &DeclaratorNode) {
    match node.id.resolve() {
        Declarator::Ident(name) => v.visit_name(name),
        Declarator::Abstract => {}
        Declarator::Pointer { qualifiers, inner } => {
            for qualifier in qualifiers {
                v.visit_qualifier(qualifier);
            }
            v.visit_declarator(inner);
        }
        Declarator::Array { declarator, size } => {
            v.visit_declarator(declarator);
            if let Some(size) = size {
                v.visit_expression(size);
            }
        }
        Declarator::Function { declarator, params } => {
            v.visit_declarator(declarator);
            v.visit_function_parameters(params);
        }
    }
}

pub fn walk_initializer<V: Visitor + ?Sized>(v: &mut V, node: &InitializerNode) {
    match &node.init {
        Initializer::Single(expr) => v.visit_expression(expr),
        Initializer::List(nodes) => {
            for init in nodes {
                v.visit_initializer(init);
            }
        }
    }
}

pub fn walk_statement<V: Visitor + ?Sized>(v: &mut V, node: &StatementNode) {
    match node.id.resolve() {
        Statement::Labeled(node) => v.visit_labeled_statement(node),
        Statement::Compound(node) => v.visit_compound_statement(node),
        Statement::Expression(node) => v.visit_expression_statement(node),
        Statement::Selection(node) => v.visit_selection_statement(node),
        Statement::Iteration(node) => v.visit_iteration_statement(node),
        Statement::Jump(node) => v.visit_jump_statement(node),
    }
}

pub fn walk_labeled_statement<V: Visitor + ?Sized>(v: &mut V, node: &LabeledStatementNode) {
    match &node.inner {
        Labeled::Identifier(name, stmt) => {
            v.visit_name(name);
            v.visit_statement(stmt);
        }
        Labeled::Case(expr, stmt) => {
            v.visit_expression(expr);
            v.visit_statement(stmt);
        }
        Labeled::Default(stmt) => v.visit_statement(stmt),
    }
}

pub fn walk_compound_statement<V: Visitor + ?Sized>(v: &mut V, node: &CompoundStatementNode) {
    for decl in &node.declarations {
        v.visit_declaration(decl);
    }
    for stmt in &node.statements {
        v.visit_statement(stmt);
    }
}

pub fn walk_expression_statement<V: Visitor + ?Sized>(v: &mut V, node: &ExpressionStatementNode) {
    if let Some(expr) = &node.expr {
        v.visit_expression(expr);
    }
}

pub fn walk_selection_statement<V: Visitor + ?Sized>(v: &mut V, node: &SelectionStatementNode) {
    match &node.stmt {
        SelectionStatement::If(cond, then, otherwise) => {
            v.visit_expression(cond);
            v.visit_statement(then);
            if let Some(otherwise) = otherwise {
                v.visit_statement(otherwise);
            }
        }
        SelectionStatement::Switch(cond, stmt) => {
            v.visit_expression(cond);
            v.visit_statement(stmt);
        }
    }
}

pub fn walk_iteration_statement<V: Visitor + ?Sized>(v: &mut V, node: &IterationStatementNode) {
    match &node.stmt {
        IterationStatement::While(cond, body) => {
            v.visit_expression(cond);
            v.visit_statement(body);
        }
        IterationStatement::Do(body, cond) => {
            v.visit_statement(body);
            v.visit_expression(cond);
        }
        IterationStatement::For(b) => {
            let (init, cond, inc, body) = b.deref();
            v.visit_expression_statement(init);
            v.visit_expression_statement(cond);
            if let Some(inc) = inc {
                v.visit_expression(inc);
            }
            v.visit_statement(body);
        }
    }
}

pub fn walk_jump_statement<V: Visitor + ?Sized>(v: &mut V, node: &JumpStatementNode) {
    if let JumpStatement::Return(Some(expr)) = &node.stmt {
        v.visit_expression(expr);
    }
}

pub fn walk_expression<V: Visitor + ?Sized>(v: &mut V, node: &ExpressionNode) {
    match node.id.resolve() {
        Expression::Identifier(name) => v.visit_name(name),
        Expression::StringLiteral(literal) => v.visit_string_literal(literal),
        Expression::Constant(value) => v.visit_value(value),
        Expression::ConstantExpression(expr) | Expression::Unary(_, expr) | Expression::SizeofExpr(expr) => {
            v.visit_expression(expr)
        }
        Expression::Binary(_, lhs, rhs) | Expression::Assign(_, lhs, rhs) | Expression::ArraySubscripting(lhs, rhs) => {
            v.visit_expression(lhs);
            v.visit_expression(rhs);
        }
        Expression::FunctionCall(lhs, args) => {
            v.visit_expression(lhs);
            for arg in args {
                v.visit_expression(arg);
            }
        }
        Expression::Ternary(cond, then, otherwise) => {
            v.visit_expression(cond);
            v.visit_expression(then);
            v.visit_expression(otherwise);
        }
        Expression::Member(_, tag, ident) => {
            v.visit_expression(tag);
            v.visit_name(ident);
        }
        Expression::Cast(ty, expr) => {
            v.visit_type(ty);
            v.visit_expression(expr);
        }
        Expression::SizeofType(ty) => v.visit_type(ty),
        Expression::List(exps) => {
            for e in exps {
                v.visit_expression(e);
            }
        }
    }
}

pub fn walk_type<V: Visitor + ?Sized>(v: &mut V, node: &Type) {
    walk_specifiers(v, &node.specifiers);
    v.visit_declarator(&node.declarator);
}

pub fn walk_function_parameters<V: Visitor + ?Sized>(v: &mut V, node: &FunctionParametersNode) {
    match &node.param {
        FunctionParameters::Empty => {}
        FunctionParameters::OldStyle(names) => {
            for name in names {
                v.visit_name(name);
            }
        }
        FunctionParameters::ParameterTypeList(decls) | FunctionParameters::Variadic(decls) => {
            for decl in decls {
                v.visit_parameter_declaration(decl);
            }
        }
    }
}

pub fn walk_parameter_declaration<V: Visitor + ?Sized>(v: &mut V, node: &ParameterDeclaration) {
    walk_specifiers(v, &node.specifiers);
    v.visit_declarator(&node.declarator);
}

pub fn walk_struct<V: Visitor + ?Sized>(v: &mut V, node: &Struct) {
    if let Some(name) = &node.name {
        v.visit_name(name);
    }
    for field in &node.fields {
        v.visit_struct_declaration(field);
    }
}

pub fn walk_union<V: Visitor + ?Sized>(v: &mut V, node: &Union) {
    if let Some(name) = &node.name {
        v.visit_name(name);
    }
    for field in &node.fields {
        v.visit_struct_declaration(field);
    }
}

pub fn walk_enum<V: Visitor + ?Sized>(v: &mut V, node: &Enum) {
    if let Some(name) = &node.name {
        v.visit_name(name);
    }
    for variant_id in &node.variants {
        v.visit_variant(variant_id.resolve());
    }
}

pub fn walk_variant<V: Visitor + ?Sized>(v: &mut V, node: &Variant) {
    v.visit_name(&node.name);
    if let Some(value) = &node.value {
        v.visit_expression(value);
    }
}

pub fn walk_struct_declaration<V: Visitor + ?Sized>(v: &mut V, node: &StructDeclaration) {
    walk_specifiers(v, &node.specifiers);
    for declarator in &node.struct_declarators {
        v.visit_struct_declarator(declarator);
    }
}

pub fn walk_struct_declarator<V: Visitor + ?Sized>(v: &mut V, node: &StructMemberDeclarator) {
    v.visit_declarator(&node.declarator);
    if let Some(bit_width) = &node.bit_width {
        v.visit_expression(bit_width);
    }
}

pub fn walk_specifiers<V: Visitor + ?Sized>(v: &mut V, specifiers: &[DeclarationSpecifier]) {
    for spec in specifiers {
        if let DeclarationSpecifier::Type(type_specifier) = spec {
            match type_specifier {
                TypeSpecifier::Struct(id) => v.visit_struct(id.resolve()),
                TypeSpecifier::Union(id) => v.visit_union(id.resolve()),
                TypeSpecifier::Enum(id) => v.visit_enum(id.resolve()),
                TypeSpecifier::TypedefName(name) => v.visit_name(name),
                _ => {}
            }
        }
    }
}
