use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, Enum, Expression,
    ExpressionNode, ExpressionStatementNode, ExternalDeclaration, ExternalDeclarationNode, FunctionDefinitionNode,
    FunctionParameters, FunctionParametersNode, InitDeclaratorNode, Initializer, InitializerNode, IterationStatement,
    IterationStatementNode, JumpStatement, JumpStatementNode, Labeled, LabeledStatementNode, Name,
    ParameterDeclaration, Qualifier, SelectionStatement, SelectionStatementNode, Statement, StatementNode, Struct,
    StructDeclaration, StructDeclarator, TranslationUnitNode, Type, TypeSpecifier, Union, Variant,
};
use crate::parser::Context;

pub trait Visitor {
    fn visit_translation_unit(&mut self, ctx: &Context, node: &TranslationUnitNode, is_last: bool) {
        walk_translation_unit(self, ctx, node, is_last);
    }

    fn visit_external_declaration(&mut self, ctx: &Context, node: &ExternalDeclarationNode, is_last: bool) {
        walk_external_declaration(self, ctx, node, is_last);
    }

    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode, is_last: bool) {
        walk_function_definition(self, ctx, node, is_last);
    }

    fn visit_declaration(&mut self, ctx: &Context, node: &DeclarationNode, is_last: bool) {
        walk_declaration(self, ctx, node, is_last);
    }

    fn visit_init_declarator(&mut self, ctx: &Context, node: &InitDeclaratorNode, is_last: bool) {
        walk_init_declarator(self, ctx, node, is_last);
    }

    fn visit_declarator(&mut self, ctx: &Context, node: &DeclaratorNode, is_last: bool) {
        walk_declarator(self, ctx, node, is_last);
    }

    fn visit_initializer(&mut self, ctx: &Context, node: &InitializerNode, is_last: bool) {
        walk_initializer(self, ctx, node, is_last);
    }

    fn visit_statement(&mut self, ctx: &Context, node: &StatementNode, is_last: bool) {
        walk_statement(self, ctx, node, is_last);
    }

    fn visit_labeled_statement(&mut self, ctx: &Context, node: &LabeledStatementNode, is_last: bool) {
        walk_labeled_statement(self, ctx, node, is_last);
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode, is_last: bool) {
        walk_compound_statement(self, ctx, node, is_last);
    }

    fn visit_expression_statement(&mut self, ctx: &Context, node: &ExpressionStatementNode, is_last: bool) {
        walk_expression_statement(self, ctx, node, is_last);
    }

    fn visit_selection_statement(&mut self, ctx: &Context, node: &SelectionStatementNode, is_last: bool) {
        walk_selection_statement(self, ctx, node, is_last);
    }

    fn visit_iteration_statement(&mut self, ctx: &Context, node: &IterationStatementNode, is_last: bool) {
        walk_iteration_statement(self, ctx, node, is_last);
    }

    fn visit_jump_statement(&mut self, ctx: &Context, node: &JumpStatementNode, is_last: bool) {
        walk_jump_statement(self, ctx, node, is_last);
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode, is_last: bool) {
        walk_expression(self, ctx, node, is_last);
    }

    fn visit_type(&mut self, ctx: &Context, node: &Type, is_last: bool) {
        walk_type(self, ctx, node, is_last);
    }

    fn visit_function_parameters(&mut self, ctx: &Context, node: &FunctionParametersNode, is_last: bool) {
        walk_function_parameters(self, ctx, node, is_last);
    }

    fn visit_parameter_declaration(&mut self, ctx: &Context, node: &ParameterDeclaration, is_last: bool) {
        walk_parameter_declaration(self, ctx, node, is_last);
    }

    fn visit_struct(&mut self, ctx: &Context, node: &Struct, is_last: bool) {
        walk_struct(self, ctx, node, is_last);
    }

    fn visit_union(&mut self, ctx: &Context, node: &Union, is_last: bool) {
        walk_union(self, ctx, node, is_last);
    }

    fn visit_enum(&mut self, ctx: &Context, node: &Enum, is_last: bool) {
        walk_enum(self, ctx, node, is_last);
    }

    fn visit_variant(&mut self, ctx: &Context, node: &Variant, is_last: bool) {
        walk_variant(self, ctx, node, is_last);
    }

    fn visit_struct_declaration(&mut self, ctx: &Context, node: &StructDeclaration, is_last: bool) {
        walk_struct_declaration(self, ctx, node, is_last);
    }

    fn visit_struct_declarator(&mut self, ctx: &Context, node: &StructDeclarator, is_last: bool) {
        walk_struct_declarator(self, ctx, node, is_last);
    }

    fn visit_qualifier(&mut self, _ctx: &Context, _qualifier: &Qualifier) {}

    fn visit_name(&mut self, _ctx: &Context, _node: &Name) {}
}

pub fn walk_translation_unit<V: Visitor + ?Sized>(
    v: &mut V,
    ctx: &Context,
    node: &TranslationUnitNode,
    _is_last: bool,
) {
    for (is_last, decl) in node.declarations.iter().with_last() {
        v.visit_external_declaration(ctx, decl, is_last);
    }
}

pub fn walk_external_declaration<V: Visitor + ?Sized>(
    v: &mut V,
    ctx: &Context,
    node: &ExternalDeclarationNode,
    is_last: bool,
) {
    match &node.decl {
        ExternalDeclaration::Function(function) => v.visit_function_definition(ctx, function, is_last),
        ExternalDeclaration::Declaration(decl) => v.visit_declaration(ctx, decl, is_last),
    }
}

pub fn walk_function_definition<V: Visitor + ?Sized>(
    v: &mut V,
    ctx: &Context,
    node: &FunctionDefinitionNode,
    _is_last: bool,
) {
    walk_specifiers(v, ctx, &node.specifiers);
    v.visit_declarator(ctx, &node.declarator, false);
    for arg in &node.arguments {
        v.visit_declaration(ctx, arg, false);
    }
    v.visit_compound_statement(ctx, &node.body, true);
}

pub fn walk_declaration<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &DeclarationNode, _is_last: bool) {
    let tags = node
        .specifiers
        .iter()
        .filter(|s| {
            matches!(
                s,
                DeclarationSpecifier::Type(TypeSpecifier::Struct(_) | TypeSpecifier::Union(_) | TypeSpecifier::Enum(_))
            )
        })
        .count();
    let total = tags + node.init_declarators.len();
    let mut i = 0;
    for spec in &node.specifiers {
        if let DeclarationSpecifier::Type(type_specifier) = spec {
            match type_specifier {
                TypeSpecifier::Struct(id) => {
                    i += 1;
                    v.visit_struct(ctx, id.resolve(&ctx.arenas), i == total);
                }
                TypeSpecifier::Union(id) => {
                    i += 1;
                    v.visit_union(ctx, id.resolve(&ctx.arenas), i == total);
                }
                TypeSpecifier::Enum(id) => {
                    i += 1;
                    v.visit_enum(ctx, id.resolve(&ctx.arenas), i == total);
                }
                TypeSpecifier::TypedefName(name) => v.visit_name(ctx, name),
                _ => {}
            }
        }
    }
    for (is_last, init) in node.init_declarators.iter().with_last() {
        v.visit_init_declarator(ctx, init, is_last);
    }
}

pub fn walk_init_declarator<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &InitDeclaratorNode, _is_last: bool) {
    v.visit_declarator(ctx, &node.declarator, node.initializer.is_none());
    if let Some(init) = &node.initializer {
        v.visit_initializer(ctx, init, true);
    }
}

pub fn walk_declarator<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &DeclaratorNode, _is_last: bool) {
    match node.id.resolve(&ctx.arenas) {
        Declarator::Ident(name) => v.visit_name(ctx, name),
        Declarator::Abstract => {}
        Declarator::Pointer { qualifiers, inner } => {
            for qualifier in qualifiers {
                v.visit_qualifier(ctx, qualifier);
            }
            if let Some(inner) = inner {
                v.visit_declarator(ctx, inner, true);
            }
        }
        Declarator::Array { declarator, size } => {
            v.visit_declarator(ctx, declarator, size.is_none());
            if let Some(size) = size {
                v.visit_expression(ctx, size, true);
            }
        }
        Declarator::Function { declarator, params } => {
            v.visit_declarator(ctx, declarator, false);
            v.visit_function_parameters(ctx, params, true);
        }
    }
}

pub fn walk_initializer<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &InitializerNode, _is_last: bool) {
    match &node.init {
        Initializer::Single(expr) => v.visit_expression(ctx, expr, true),
        Initializer::List(nodes) => {
            for (is_last, init) in nodes.iter().with_last() {
                v.visit_initializer(ctx, init, is_last);
            }
        }
    }
}

pub fn walk_statement<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &StatementNode, is_last: bool) {
    match node.id.resolve(&ctx.arenas) {
        Statement::Labeled(node) => v.visit_labeled_statement(ctx, node, is_last),
        Statement::Compound(node) => v.visit_compound_statement(ctx, node, is_last),
        Statement::Expression(node) => v.visit_expression_statement(ctx, node, is_last),
        Statement::Selection(node) => v.visit_selection_statement(ctx, node, is_last),
        Statement::Iteration(node) => v.visit_iteration_statement(ctx, node, is_last),
        Statement::Jump(node) => v.visit_jump_statement(ctx, node, is_last),
    }
}

pub fn walk_labeled_statement<V: Visitor + ?Sized>(
    v: &mut V,
    ctx: &Context,
    node: &LabeledStatementNode,
    _is_last: bool,
) {
    match &node.inner {
        Labeled::Identifier(name, stmt) => {
            v.visit_name(ctx, name);
            v.visit_statement(ctx, stmt, true);
        }
        Labeled::Case(expr, stmt) => {
            v.visit_expression(ctx, expr, false);
            v.visit_statement(ctx, stmt, true);
        }
        Labeled::Default(stmt) => v.visit_statement(ctx, stmt, true),
    }
}

pub fn walk_compound_statement<V: Visitor + ?Sized>(
    v: &mut V,
    ctx: &Context,
    node: &CompoundStatementNode,
    _is_last: bool,
) {
    for decl in &node.declarations {
        v.visit_declaration(ctx, decl, false);
    }
    for (is_last, stmt) in node.statements.iter().with_last() {
        v.visit_statement(ctx, stmt, is_last);
    }
}

pub fn walk_expression_statement<V: Visitor + ?Sized>(
    v: &mut V,
    ctx: &Context,
    node: &ExpressionStatementNode,
    _is_last: bool,
) {
    if let Some(expr) = &node.expr {
        v.visit_expression(ctx, expr, true);
    }
}

pub fn walk_selection_statement<V: Visitor + ?Sized>(
    v: &mut V,
    ctx: &Context,
    node: &SelectionStatementNode,
    _is_last: bool,
) {
    match &node.stmt {
        SelectionStatement::If(cond, then, otherwise) => {
            v.visit_expression(ctx, cond, false);
            v.visit_statement(ctx, then, otherwise.is_none());
            if let Some(otherwise) = otherwise {
                v.visit_statement(ctx, otherwise, true);
            }
        }
        SelectionStatement::Switch(cond, stmt) => {
            v.visit_expression(ctx, cond, false);
            v.visit_statement(ctx, stmt, true);
        }
    }
}

pub fn walk_iteration_statement<V: Visitor + ?Sized>(
    v: &mut V,
    ctx: &Context,
    node: &IterationStatementNode,
    _is_last: bool,
) {
    match &node.stmt {
        IterationStatement::While(cond, body) => {
            v.visit_expression(ctx, cond, false);
            v.visit_statement(ctx, body, true);
        }
        IterationStatement::Do(body, cond) => {
            v.visit_statement(ctx, body, false);
            v.visit_expression(ctx, cond, true);
        }
        IterationStatement::For(init, cond, inc, body) => {
            v.visit_expression_statement(ctx, init, false);
            v.visit_expression_statement(ctx, cond, false);
            if let Some(inc) = inc {
                v.visit_expression(ctx, inc, false);
            }
            v.visit_statement(ctx, body, true);
        }
    }
}

pub fn walk_jump_statement<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &JumpStatementNode, _is_last: bool) {
    if let JumpStatement::Return(Some(expr)) = &node.stmt {
        v.visit_expression(ctx, expr, true);
    }
}

pub fn walk_expression<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &ExpressionNode, _is_last: bool) {
    match node.id.resolve(&ctx.arenas) {
        Expression::Identifier(name) | Expression::Constant(name) | Expression::StringLiteral(name) => {
            v.visit_name(ctx, name)
        }
        Expression::ConstantExpression(expr)
        | Expression::PostInc(expr)
        | Expression::PostDec(expr)
        | Expression::PreInc(expr)
        | Expression::PreDec(expr)
        | Expression::Addr(expr)
        | Expression::Deref(expr)
        | Expression::Plus(expr)
        | Expression::Minus(expr)
        | Expression::BitNot(expr)
        | Expression::Not(expr)
        | Expression::SizeofExpr(expr)
        | Expression::FunctionCall(expr, None) => v.visit_expression(ctx, expr, true),
        Expression::Add(lhs, rhs)
        | Expression::Sub(lhs, rhs)
        | Expression::Mul(lhs, rhs)
        | Expression::Div(lhs, rhs)
        | Expression::Mod(lhs, rhs)
        | Expression::Right(lhs, rhs)
        | Expression::Left(lhs, rhs)
        | Expression::Greater(lhs, rhs)
        | Expression::Lower(lhs, rhs)
        | Expression::GreaterEq(lhs, rhs)
        | Expression::LowerEq(lhs, rhs)
        | Expression::Eq(lhs, rhs)
        | Expression::Neq(lhs, rhs)
        | Expression::BitAnd(lhs, rhs)
        | Expression::BitOr(lhs, rhs)
        | Expression::BitXor(lhs, rhs)
        | Expression::And(lhs, rhs)
        | Expression::Or(lhs, rhs)
        | Expression::Assign(lhs, rhs)
        | Expression::AddAssign(lhs, rhs)
        | Expression::SubAssign(lhs, rhs)
        | Expression::MulAssign(lhs, rhs)
        | Expression::DivAssign(lhs, rhs)
        | Expression::ModAssign(lhs, rhs)
        | Expression::RightAssign(lhs, rhs)
        | Expression::LeftAssign(lhs, rhs)
        | Expression::AndAssign(lhs, rhs)
        | Expression::OrAssign(lhs, rhs)
        | Expression::XorAssign(lhs, rhs)
        | Expression::List(lhs, rhs)
        | Expression::ArrayAcces(lhs, rhs)
        | Expression::FunctionCall(lhs, Some(rhs)) => {
            v.visit_expression(ctx, lhs, false);
            v.visit_expression(ctx, rhs, true);
        }
        Expression::Ternary(cond, then, otherwise) => {
            v.visit_expression(ctx, cond, false);
            v.visit_expression(ctx, then, false);
            v.visit_expression(ctx, otherwise, true);
        }
        Expression::DotAcces(tag, ident) | Expression::PtrAcces(tag, ident) => {
            v.visit_expression(ctx, tag, false);
            v.visit_name(ctx, ident);
        }
        Expression::Cast(ty, expr) => {
            v.visit_type(ctx, ty, false);
            v.visit_expression(ctx, expr, true);
        }
        Expression::SizeofType(ty) => v.visit_type(ctx, ty, true),
    }
}

pub fn walk_type<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &Type, _is_last: bool) {
    walk_specifiers(v, ctx, &node.specifiers);
    v.visit_declarator(ctx, &node.declarator, true);
}

pub fn walk_function_parameters<V: Visitor + ?Sized>(
    v: &mut V,
    ctx: &Context,
    node: &FunctionParametersNode,
    _is_last: bool,
) {
    match &node.param {
        FunctionParameters::Empty => {}
        FunctionParameters::OldStyle(names) => {
            for name in names {
                v.visit_name(ctx, name);
            }
        }
        FunctionParameters::ParameterTypeList(decls) | FunctionParameters::Variadic(decls) => {
            for (is_last, decl) in decls.iter().with_last() {
                v.visit_parameter_declaration(ctx, decl, is_last);
            }
        }
    }
}

pub fn walk_parameter_declaration<V: Visitor + ?Sized>(
    v: &mut V,
    ctx: &Context,
    node: &ParameterDeclaration,
    _is_last: bool,
) {
    walk_specifiers(v, ctx, &node.specifiers);
    v.visit_declarator(ctx, &node.declarator, true);
}

pub fn walk_struct<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &Struct, _is_last: bool) {
    if let Some(name) = &node.name {
        v.visit_name(ctx, name);
    }
    for (is_last, field) in node.fields.iter().with_last() {
        v.visit_struct_declaration(ctx, field, is_last);
    }
}

pub fn walk_union<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &Union, _is_last: bool) {
    if let Some(name) = &node.name {
        v.visit_name(ctx, name);
    }
    for (is_last, field) in node.fields.iter().with_last() {
        v.visit_struct_declaration(ctx, field, is_last);
    }
}

pub fn walk_enum<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &Enum, _is_last: bool) {
    if let Some(name) = &node.name {
        v.visit_name(ctx, name);
    }
    for (is_last, variant_id) in node.variants.iter().with_last() {
        v.visit_variant(ctx, variant_id.resolve(&ctx.arenas), is_last);
    }
}

pub fn walk_variant<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &Variant, _is_last: bool) {
    v.visit_name(ctx, &node.name);
    if let Some(value) = &node.value {
        v.visit_expression(ctx, value, true);
    }
}

pub fn walk_struct_declaration<V: Visitor + ?Sized>(
    v: &mut V,
    ctx: &Context,
    node: &StructDeclaration,
    _is_last: bool,
) {
    walk_specifiers(v, ctx, &node.specifiers);
    for (is_last, declarator) in node.struct_declarators.iter().with_last() {
        v.visit_struct_declarator(ctx, declarator, is_last);
    }
}

pub fn walk_struct_declarator<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, node: &StructDeclarator, _is_last: bool) {
    v.visit_declarator(ctx, &node.declarator, node.bit_width.is_none());
    if let Some(bit_width) = &node.bit_width {
        v.visit_expression(ctx, bit_width, true);
    }
}

pub fn walk_specifiers<V: Visitor + ?Sized>(v: &mut V, ctx: &Context, specifiers: &[DeclarationSpecifier]) {
    let tags = specifiers
        .iter()
        .filter(|s| {
            matches!(
                s,
                DeclarationSpecifier::Type(TypeSpecifier::Struct(_) | TypeSpecifier::Union(_) | TypeSpecifier::Enum(_))
            )
        })
        .count();
    let mut i = 0;
    for spec in specifiers {
        if let DeclarationSpecifier::Type(type_specifier) = spec {
            match type_specifier {
                TypeSpecifier::Struct(id) => {
                    i += 1;
                    v.visit_struct(ctx, id.resolve(&ctx.arenas), i == tags);
                }
                TypeSpecifier::Union(id) => {
                    i += 1;
                    v.visit_union(ctx, id.resolve(&ctx.arenas), i == tags);
                }
                TypeSpecifier::Enum(id) => {
                    i += 1;
                    v.visit_enum(ctx, id.resolve(&ctx.arenas), i == tags);
                }
                TypeSpecifier::TypedefName(name) => v.visit_name(ctx, name),
                _ => {}
            }
        }
    }
}

pub trait WithLast: Iterator {
    fn with_last(self) -> WithLastIter<Self>
    where
        Self: Sized,
    {
        WithLastIter { iter: self.peekable() }
    }
}

impl<I: Iterator> WithLast for I {}

pub struct WithLastIter<I: Iterator> {
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
