use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, Enum, Expression,
    ExpressionNode, ExpressionStatementNode, ExternalDeclaration, ExternalDeclarationNode, FunctionDefinitionNode,
    FunctionParameters, FunctionParametersNode, InitDeclaratorNode, Initializer, InitializerNode, IterationStatement,
    IterationStatementNode, JumpStatement, JumpStatementNode, Labeled, LabeledStatementNode, Name,
    ParameterDeclaration, Qualifier, SelectionStatement, SelectionStatementNode, Statement, StatementNode, Struct,
    StructDeclaration, StructDeclarator, TranslationUnitNode, Type, TypeSpecifier, Union, Variant,
};
use crate::parser::Arenas;

pub trait Visitor {
    fn visit_translation_unit(&mut self, arenas: &Arenas, node: &TranslationUnitNode, is_last: bool) {
        walk_translation_unit(self, arenas, node, is_last);
    }

    fn visit_external_declaration(&mut self, arenas: &Arenas, node: &ExternalDeclarationNode, is_last: bool) {
        walk_external_declaration(self, arenas, node, is_last);
    }

    fn visit_function_definition(&mut self, arenas: &Arenas, node: &FunctionDefinitionNode, is_last: bool) {
        walk_function_definition(self, arenas, node, is_last);
    }

    fn visit_declaration(&mut self, arenas: &Arenas, node: &DeclarationNode, is_last: bool) {
        walk_declaration(self, arenas, node, is_last);
    }

    fn visit_init_declarator(&mut self, arenas: &Arenas, node: &InitDeclaratorNode, is_last: bool) {
        walk_init_declarator(self, arenas, node, is_last);
    }

    fn visit_declarator(&mut self, arenas: &Arenas, node: &DeclaratorNode, is_last: bool) {
        walk_declarator(self, arenas, node, is_last);
    }

    fn visit_initializer(&mut self, arenas: &Arenas, node: &InitializerNode, is_last: bool) {
        walk_initializer(self, arenas, node, is_last);
    }

    fn visit_statement(&mut self, arenas: &Arenas, node: &StatementNode, is_last: bool) {
        walk_statement(self, arenas, node, is_last);
    }

    fn visit_labeled_statement(&mut self, arenas: &Arenas, node: &LabeledStatementNode, is_last: bool) {
        walk_labeled_statement(self, arenas, node, is_last);
    }

    fn visit_compound_statement(&mut self, arenas: &Arenas, node: &CompoundStatementNode, is_last: bool) {
        walk_compound_statement(self, arenas, node, is_last);
    }

    fn visit_expression_statement(&mut self, arenas: &Arenas, node: &ExpressionStatementNode, is_last: bool) {
        walk_expression_statement(self, arenas, node, is_last);
    }

    fn visit_selection_statement(&mut self, arenas: &Arenas, node: &SelectionStatementNode, is_last: bool) {
        walk_selection_statement(self, arenas, node, is_last);
    }

    fn visit_iteration_statement(&mut self, arenas: &Arenas, node: &IterationStatementNode, is_last: bool) {
        walk_iteration_statement(self, arenas, node, is_last);
    }

    fn visit_jump_statement(&mut self, arenas: &Arenas, node: &JumpStatementNode, is_last: bool) {
        walk_jump_statement(self, arenas, node, is_last);
    }

    fn visit_expression(&mut self, arenas: &Arenas, node: &ExpressionNode, is_last: bool) {
        walk_expression(self, arenas, node, is_last);
    }

    fn visit_type(&mut self, arenas: &Arenas, node: &Type, is_last: bool) {
        walk_type(self, arenas, node, is_last);
    }

    fn visit_function_parameters(&mut self, arenas: &Arenas, node: &FunctionParametersNode, is_last: bool) {
        walk_function_parameters(self, arenas, node, is_last);
    }

    fn visit_parameter_declaration(&mut self, arenas: &Arenas, node: &ParameterDeclaration, is_last: bool) {
        walk_parameter_declaration(self, arenas, node, is_last);
    }

    fn visit_struct(&mut self, arenas: &Arenas, node: &Struct, is_last: bool) {
        walk_struct(self, arenas, node, is_last);
    }

    fn visit_union(&mut self, arenas: &Arenas, node: &Union, is_last: bool) {
        walk_union(self, arenas, node, is_last);
    }

    fn visit_enum(&mut self, arenas: &Arenas, node: &Enum, is_last: bool) {
        walk_enum(self, arenas, node, is_last);
    }

    fn visit_variant(&mut self, arenas: &Arenas, node: &Variant, is_last: bool) {
        walk_variant(self, arenas, node, is_last);
    }

    fn visit_struct_declaration(&mut self, arenas: &Arenas, node: &StructDeclaration, is_last: bool) {
        walk_struct_declaration(self, arenas, node, is_last);
    }

    fn visit_struct_declarator(&mut self, arenas: &Arenas, node: &StructDeclarator, is_last: bool) {
        walk_struct_declarator(self, arenas, node, is_last);
    }

    fn visit_qualifier(&mut self, _arenas: &Arenas, _qualifier: &Qualifier) {}

    fn visit_name(&mut self, _arenas: &Arenas, _node: &Name) {}
}

pub fn walk_translation_unit<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &TranslationUnitNode,
    _is_last: bool,
) {
    for (is_last, decl) in node.declarations.iter().with_last() {
        v.visit_external_declaration(arenas, decl, is_last);
    }
}

pub fn walk_external_declaration<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &ExternalDeclarationNode,
    is_last: bool,
) {
    match &node.decl {
        ExternalDeclaration::Function(function) => v.visit_function_definition(arenas, function, is_last),
        ExternalDeclaration::Declaration(decl) => v.visit_declaration(arenas, decl, is_last),
    }
}

pub fn walk_function_definition<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &FunctionDefinitionNode,
    _is_last: bool,
) {
    walk_specifiers(v, arenas, &node.specifiers);
    v.visit_declarator(arenas, &node.declarator, false);
    for arg in &node.arguments {
        v.visit_declaration(arenas, arg, false);
    }
    v.visit_compound_statement(arenas, &node.body, true);
}

pub fn walk_declaration<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &DeclarationNode, _is_last: bool) {
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
                    v.visit_struct(arenas, arenas.structs.get(*id), i == total);
                }
                TypeSpecifier::Union(id) => {
                    i += 1;
                    v.visit_union(arenas, arenas.unions.get(*id), i == total);
                }
                TypeSpecifier::Enum(id) => {
                    i += 1;
                    v.visit_enum(arenas, arenas.enums.get(*id), i == total);
                }
                TypeSpecifier::TypedefName(name) => v.visit_name(arenas, name),
                _ => {}
            }
        }
    }
    for (is_last, init) in node.init_declarators.iter().with_last() {
        v.visit_init_declarator(arenas, init, is_last);
    }
}

pub fn walk_init_declarator<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &InitDeclaratorNode,
    _is_last: bool,
) {
    v.visit_declarator(arenas, &node.declarator, node.initializer.is_none());
    if let Some(init) = &node.initializer {
        v.visit_initializer(arenas, init, true);
    }
}

pub fn walk_declarator<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &DeclaratorNode, _is_last: bool) {
    match arenas.declarators.get(node.id) {
        Declarator::Ident(name) => v.visit_name(arenas, name),
        Declarator::Abstract => {}
        Declarator::Pointer { qualifiers, inner } => {
            for qualifier in qualifiers {
                v.visit_qualifier(arenas, qualifier);
            }
            if let Some(inner) = inner {
                v.visit_declarator(arenas, inner, true);
            }
        }
        Declarator::Array { declarator, size } => {
            v.visit_declarator(arenas, declarator, size.is_none());
            if let Some(size) = size {
                v.visit_expression(arenas, size, true);
            }
        }
        Declarator::Function { declarator, params } => {
            v.visit_declarator(arenas, declarator, false);
            v.visit_function_parameters(arenas, params, true);
        }
    }
}

pub fn walk_initializer<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &InitializerNode, _is_last: bool) {
    match &node.init {
        Initializer::Single(expr) => v.visit_expression(arenas, expr, true),
        Initializer::List(nodes) => {
            for (is_last, init) in nodes.iter().with_last() {
                v.visit_initializer(arenas, init, is_last);
            }
        }
    }
}

pub fn walk_statement<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &StatementNode, is_last: bool) {
    match arenas.statements.get(node.id) {
        Statement::Labeled(node) => v.visit_labeled_statement(arenas, node, is_last),
        Statement::Compound(node) => v.visit_compound_statement(arenas, node, is_last),
        Statement::Expression(node) => v.visit_expression_statement(arenas, node, is_last),
        Statement::Selection(node) => v.visit_selection_statement(arenas, node, is_last),
        Statement::Iteration(node) => v.visit_iteration_statement(arenas, node, is_last),
        Statement::Jump(node) => v.visit_jump_statement(arenas, node, is_last),
    }
}

pub fn walk_labeled_statement<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &LabeledStatementNode,
    _is_last: bool,
) {
    match &node.inner {
        Labeled::Identifier(name, stmt) => {
            v.visit_name(arenas, name);
            v.visit_statement(arenas, stmt, true);
        }
        Labeled::Case(expr, stmt) => {
            v.visit_expression(arenas, expr, false);
            v.visit_statement(arenas, stmt, true);
        }
        Labeled::Default(stmt) => v.visit_statement(arenas, stmt, true),
    }
}

pub fn walk_compound_statement<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &CompoundStatementNode,
    _is_last: bool,
) {
    for decl in &node.declarations {
        v.visit_declaration(arenas, decl, false);
    }
    for (is_last, stmt) in node.statements.iter().with_last() {
        v.visit_statement(arenas, stmt, is_last);
    }
}

pub fn walk_expression_statement<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &ExpressionStatementNode,
    _is_last: bool,
) {
    if let Some(expr) = &node.expr {
        v.visit_expression(arenas, expr, true);
    }
}

pub fn walk_selection_statement<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &SelectionStatementNode,
    _is_last: bool,
) {
    match &node.stmt {
        SelectionStatement::If(cond, then, otherwise) => {
            v.visit_expression(arenas, cond, false);
            v.visit_statement(arenas, then, otherwise.is_none());
            if let Some(otherwise) = otherwise {
                v.visit_statement(arenas, otherwise, true);
            }
        }
        SelectionStatement::Switch(cond, stmt) => {
            v.visit_expression(arenas, cond, false);
            v.visit_statement(arenas, stmt, true);
        }
    }
}

pub fn walk_iteration_statement<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &IterationStatementNode,
    _is_last: bool,
) {
    match &node.stmt {
        IterationStatement::While(cond, body) => {
            v.visit_expression(arenas, cond, false);
            v.visit_statement(arenas, body, true);
        }
        IterationStatement::Do(body, cond) => {
            v.visit_statement(arenas, body, false);
            v.visit_expression(arenas, cond, true);
        }
        IterationStatement::For(init, cond, inc, body) => {
            v.visit_expression_statement(arenas, init, false);
            v.visit_expression_statement(arenas, cond, false);
            if let Some(inc) = inc {
                v.visit_expression(arenas, inc, false);
            }
            v.visit_statement(arenas, body, true);
        }
    }
}

pub fn walk_jump_statement<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &JumpStatementNode, _is_last: bool) {
    if let JumpStatement::Return(Some(expr)) = &node.stmt {
        v.visit_expression(arenas, expr, true);
    }
}

pub fn walk_expression<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &ExpressionNode, _is_last: bool) {
    match arenas.expressions.get(node.id) {
        Expression::Identifier(name) | Expression::Constant(name) | Expression::StringLiteral(name) => {
            v.visit_name(arenas, name)
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
        | Expression::FunctionCall(expr, None) => v.visit_expression(arenas, expr, true),
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
            v.visit_expression(arenas, lhs, false);
            v.visit_expression(arenas, rhs, true);
        }
        Expression::Ternary(cond, then, otherwise) => {
            v.visit_expression(arenas, cond, false);
            v.visit_expression(arenas, then, false);
            v.visit_expression(arenas, otherwise, true);
        }
        Expression::DotAcces(tag, ident) | Expression::PtrAcces(tag, ident) => {
            v.visit_expression(arenas, tag, false);
            v.visit_name(arenas, ident);
        }
        Expression::Cast(ty, expr) => {
            v.visit_type(arenas, ty, false);
            v.visit_expression(arenas, expr, true);
        }
        Expression::SizeofType(ty) => v.visit_type(arenas, ty, true),
    }
}

pub fn walk_type<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &Type, _is_last: bool) {
    walk_specifiers(v, arenas, &node.specifiers);
    v.visit_declarator(arenas, &node.declarator, true);
}

pub fn walk_function_parameters<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &FunctionParametersNode,
    _is_last: bool,
) {
    match &node.param {
        FunctionParameters::Empty => {}
        FunctionParameters::OldStyle(names) => {
            for name in names {
                v.visit_name(arenas, name);
            }
        }
        FunctionParameters::ParameterTypeList(decls) | FunctionParameters::Variadic(decls) => {
            for (is_last, decl) in decls.iter().with_last() {
                v.visit_parameter_declaration(arenas, decl, is_last);
            }
        }
    }
}

pub fn walk_parameter_declaration<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &ParameterDeclaration,
    _is_last: bool,
) {
    walk_specifiers(v, arenas, &node.specifiers);
    v.visit_declarator(arenas, &node.declarator, true);
}

pub fn walk_struct<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &Struct, _is_last: bool) {
    if let Some(name) = &node.name {
        v.visit_name(arenas, name);
    }
    for (is_last, field) in node.fields.iter().with_last() {
        v.visit_struct_declaration(arenas, field, is_last);
    }
}

pub fn walk_union<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &Union, _is_last: bool) {
    if let Some(name) = &node.name {
        v.visit_name(arenas, name);
    }
    for (is_last, field) in node.fields.iter().with_last() {
        v.visit_struct_declaration(arenas, field, is_last);
    }
}

pub fn walk_enum<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &Enum, _is_last: bool) {
    if let Some(name) = &node.name {
        v.visit_name(arenas, name);
    }
    for (is_last, variant_id) in node.variants.iter().with_last() {
        v.visit_variant(arenas, arenas.variants.get(*variant_id), is_last);
    }
}

pub fn walk_variant<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &Variant, _is_last: bool) {
    v.visit_name(arenas, &node.name);
    if let Some(value) = &node.value {
        v.visit_expression(arenas, value, true);
    }
}

pub fn walk_struct_declaration<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &StructDeclaration,
    _is_last: bool,
) {
    walk_specifiers(v, arenas, &node.specifiers);
    for (is_last, declarator) in node.struct_declarators.iter().with_last() {
        v.visit_struct_declarator(arenas, declarator, is_last);
    }
}

pub fn walk_struct_declarator<V: Visitor + ?Sized>(
    v: &mut V,
    arenas: &Arenas,
    node: &StructDeclarator,
    _is_last: bool,
) {
    v.visit_declarator(arenas, &node.declarator, node.bit_width.is_none());
    if let Some(bit_width) = &node.bit_width {
        v.visit_expression(arenas, bit_width, true);
    }
}

pub fn walk_specifiers<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, specifiers: &[DeclarationSpecifier]) {
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
                    v.visit_struct(arenas, arenas.structs.get(*id), i == tags);
                }
                TypeSpecifier::Union(id) => {
                    i += 1;
                    v.visit_union(arenas, arenas.unions.get(*id), i == tags);
                }
                TypeSpecifier::Enum(id) => {
                    i += 1;
                    v.visit_enum(arenas, arenas.enums.get(*id), i == tags);
                }
                TypeSpecifier::TypedefName(name) => v.visit_name(arenas, name),
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
