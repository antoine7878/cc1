use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, Enum, Expression,
    ExpressionNode, ExpressionStatementNode, ExternalDeclaration, ExternalDeclarationNode, FunctionDefinitionNode,
    FunctionParameters, FunctionParametersNode, InitDeclaratorNode, Initializer, InitializerNode, IterationStatement,
    IterationStatementNode, JumpStatement, JumpStatementNode, Labeled, LabeledStatementNode, Name,
    ParameterDeclaration, SelectionStatement, SelectionStatementNode, Statement, StatementNode, Struct,
    StructDeclaration, StructDeclarator, TranslationUnitNode, Type, TypeSpecifier, Union, Variant,
};
use crate::parser::Arenas;

pub trait Visitor {
    fn visit_translation_unit(&mut self, arenas: &Arenas, node: &TranslationUnitNode) {
        walk_translation_unit(self, arenas, node);
    }

    fn visit_external_declaration(&mut self, arenas: &Arenas, node: &ExternalDeclarationNode) {
        walk_external_declaration(self, arenas, node);
    }

    fn visit_function_definition(&mut self, arenas: &Arenas, node: &FunctionDefinitionNode) {
        walk_function_definition(self, arenas, node);
    }

    fn visit_declaration(&mut self, arenas: &Arenas, node: &DeclarationNode) {
        walk_declaration(self, arenas, node);
    }

    fn visit_init_declarator(&mut self, arenas: &Arenas, node: &InitDeclaratorNode) {
        walk_init_declarator(self, arenas, node);
    }

    fn visit_declarator(&mut self, arenas: &Arenas, node: &DeclaratorNode) {
        walk_declarator(self, arenas, node);
    }

    fn visit_initializer(&mut self, arenas: &Arenas, node: &InitializerNode) {
        walk_initializer(self, arenas, node);
    }

    fn visit_statement(&mut self, arenas: &Arenas, node: &StatementNode) {
        walk_statement(self, arenas, node);
    }

    fn visit_labeled_statement(&mut self, arenas: &Arenas, node: &LabeledStatementNode) {
        walk_labeled_statement(self, arenas, node);
    }

    fn visit_compound_statement(&mut self, arenas: &Arenas, node: &CompoundStatementNode) {
        walk_compound_statement(self, arenas, node);
    }

    fn visit_expression_statement(&mut self, arenas: &Arenas, node: &ExpressionStatementNode) {
        walk_expression_statement(self, arenas, node);
    }

    fn visit_selection_statement(&mut self, arenas: &Arenas, node: &SelectionStatementNode) {
        walk_selection_statement(self, arenas, node);
    }

    fn visit_iteration_statement(&mut self, arenas: &Arenas, node: &IterationStatementNode) {
        walk_iteration_statement(self, arenas, node);
    }

    fn visit_jump_statement(&mut self, arenas: &Arenas, node: &JumpStatementNode) {
        walk_jump_statement(self, arenas, node);
    }

    fn visit_expression(&mut self, arenas: &Arenas, node: &ExpressionNode) {
        walk_expression(self, arenas, node);
    }

    fn visit_type(&mut self, arenas: &Arenas, node: &Type) {
        walk_type(self, arenas, node);
    }

    fn visit_function_parameters(&mut self, arenas: &Arenas, node: &FunctionParametersNode) {
        walk_function_parameters(self, arenas, node);
    }

    fn visit_parameter_declaration(&mut self, arenas: &Arenas, node: &ParameterDeclaration) {
        walk_parameter_declaration(self, arenas, node);
    }

    fn visit_struct(&mut self, arenas: &Arenas, node: &Struct) {
        walk_struct(self, arenas, node);
    }

    fn visit_union(&mut self, arenas: &Arenas, node: &Union) {
        walk_union(self, arenas, node);
    }

    fn visit_enum(&mut self, arenas: &Arenas, node: &Enum) {
        walk_enum(self, arenas, node);
    }

    fn visit_variant(&mut self, arenas: &Arenas, node: &Variant) {
        walk_variant(self, arenas, node);
    }

    fn visit_struct_declaration(&mut self, arenas: &Arenas, node: &StructDeclaration) {
        walk_struct_declaration(self, arenas, node);
    }

    fn visit_struct_declarator(&mut self, arenas: &Arenas, node: &StructDeclarator) {
        walk_struct_declarator(self, arenas, node);
    }

    fn visit_name(&mut self, _arenas: &Arenas, _node: &Name) {}
}

pub fn walk_translation_unit<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &TranslationUnitNode) {
    for decl in &node.declarations {
        v.visit_external_declaration(arenas, decl);
    }
}

pub fn walk_external_declaration<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &ExternalDeclarationNode) {
    match &node.decl {
        ExternalDeclaration::Function(function) => v.visit_function_definition(arenas, function),
        ExternalDeclaration::Declaration(decl) => v.visit_declaration(arenas, decl),
    }
}

pub fn walk_function_definition<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &FunctionDefinitionNode) {
    walk_specifiers(v, arenas, &node.specifiers);
    v.visit_declarator(arenas, &node.declarator);
    for arg in &node.arguments {
        v.visit_declaration(arenas, arg);
    }
    v.visit_compound_statement(arenas, &node.body);
}

pub fn walk_declaration<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &DeclarationNode) {
    walk_specifiers(v, arenas, &node.specifiers);
    for init in &node.init_declarators {
        v.visit_init_declarator(arenas, init);
    }
}

pub fn walk_init_declarator<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &InitDeclaratorNode) {
    v.visit_declarator(arenas, &node.declarator);
    if let Some(init) = &node.initializer {
        v.visit_initializer(arenas, init);
    }
}

pub fn walk_declarator<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &DeclaratorNode) {
    match arenas.declarators.get(node.id) {
        Declarator::Ident(name) => v.visit_name(arenas, name),
        Declarator::Abstract => {}
        Declarator::Pointer { inner, .. } => {
            if let Some(inner) = inner {
                v.visit_declarator(arenas, inner);
            }
        }
        Declarator::Array { declarator, size } => {
            v.visit_declarator(arenas, declarator);
            if let Some(size) = size {
                v.visit_expression(arenas, size);
            }
        }
        Declarator::Function { declarator, params } => {
            v.visit_declarator(arenas, declarator);
            v.visit_function_parameters(arenas, params);
        }
    }
}

pub fn walk_initializer<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &InitializerNode) {
    match &node.init {
        Initializer::Single(expr) => v.visit_expression(arenas, expr),
        Initializer::List(nodes) => {
            for init in nodes {
                v.visit_initializer(arenas, init);
            }
        }
    }
}

pub fn walk_statement<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &StatementNode) {
    match arenas.statements.get(node.id) {
        Statement::Labeled(node) => v.visit_labeled_statement(arenas, node),
        Statement::Compound(node) => v.visit_compound_statement(arenas, node),
        Statement::Expression(node) => v.visit_expression_statement(arenas, node),
        Statement::Selection(node) => v.visit_selection_statement(arenas, node),
        Statement::Iteration(node) => v.visit_iteration_statement(arenas, node),
        Statement::Jump(node) => v.visit_jump_statement(arenas, node),
    }
}

pub fn walk_labeled_statement<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &LabeledStatementNode) {
    match &node.inner {
        Labeled::Identifier(name, stmt) => {
            v.visit_name(arenas, name);
            v.visit_statement(arenas, stmt);
        }
        Labeled::Case(expr, stmt) => {
            v.visit_expression(arenas, expr);
            v.visit_statement(arenas, stmt);
        }
        Labeled::Default(stmt) => v.visit_statement(arenas, stmt),
    }
}

pub fn walk_compound_statement<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &CompoundStatementNode) {
    for decl in &node.declarations {
        v.visit_declaration(arenas, decl);
    }
    for stmt in &node.statements {
        v.visit_statement(arenas, stmt);
    }
}

pub fn walk_expression_statement<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &ExpressionStatementNode) {
    if let Some(expr) = &node.expr {
        v.visit_expression(arenas, expr);
    }
}

pub fn walk_selection_statement<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &SelectionStatementNode) {
    match &node.stmt {
        SelectionStatement::If(cond, then, otherwise) => {
            v.visit_expression(arenas, cond);
            v.visit_statement(arenas, then);
            if let Some(otherwise) = otherwise {
                v.visit_statement(arenas, otherwise);
            }
        }
        SelectionStatement::Switch(cond, stmt) => {
            v.visit_expression(arenas, cond);
            v.visit_statement(arenas, stmt);
        }
    }
}

pub fn walk_iteration_statement<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &IterationStatementNode) {
    match &node.stmt {
        IterationStatement::While(cond, body) => {
            v.visit_expression(arenas, cond);
            v.visit_statement(arenas, body);
        }
        IterationStatement::Do(body, cond) => {
            v.visit_statement(arenas, body);
            v.visit_expression(arenas, cond);
        }
        IterationStatement::For(init, cond, inc, body) => {
            v.visit_expression_statement(arenas, init);
            v.visit_expression_statement(arenas, cond);
            if let Some(inc) = inc {
                v.visit_expression(arenas, inc);
            }
            v.visit_statement(arenas, body);
        }
    }
}

pub fn walk_jump_statement<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &JumpStatementNode) {
    if let JumpStatement::Return(Some(expr)) = &node.stmt {
        v.visit_expression(arenas, expr);
    }
}

pub fn walk_expression<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &ExpressionNode) {
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
        | Expression::FunctionCall(expr, None) => v.visit_expression(arenas, expr),
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
            v.visit_expression(arenas, lhs);
            v.visit_expression(arenas, rhs);
        }
        Expression::Ternary(cond, then, otherwise) => {
            v.visit_expression(arenas, cond);
            v.visit_expression(arenas, then);
            v.visit_expression(arenas, otherwise);
        }
        Expression::DotAcces(tag, ident) | Expression::PtrAcces(tag, ident) => {
            v.visit_expression(arenas, tag);
            v.visit_name(arenas, ident);
        }
        Expression::Cast(ty, expr) => {
            v.visit_type(arenas, ty);
            v.visit_expression(arenas, expr);
        }
        Expression::SizeofType(ty) => v.visit_type(arenas, ty),
    }
}

pub fn walk_type<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &Type) {
    walk_specifiers(v, arenas, &node.specifiers);
    v.visit_declarator(arenas, &node.declarator);
}

pub fn walk_function_parameters<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &FunctionParametersNode) {
    match &node.param {
        FunctionParameters::Empty => {}
        FunctionParameters::OldStyle(names) => {
            for name in names {
                v.visit_name(arenas, name);
            }
        }
        FunctionParameters::ParameterTypeList(decls) | FunctionParameters::Variadic(decls) => {
            for decl in decls {
                v.visit_parameter_declaration(arenas, decl);
            }
        }
    }
}

pub fn walk_parameter_declaration<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &ParameterDeclaration) {
    walk_specifiers(v, arenas, &node.specifiers);
    v.visit_declarator(arenas, &node.declarator);
}

pub fn walk_struct<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &Struct) {
    if let Some(name) = &node.name {
        v.visit_name(arenas, name);
    }
    for field in &node.fields {
        v.visit_struct_declaration(arenas, field);
    }
}

pub fn walk_union<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &Union) {
    if let Some(name) = &node.name {
        v.visit_name(arenas, name);
    }
    for field in &node.fields {
        v.visit_struct_declaration(arenas, field);
    }
}

pub fn walk_enum<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &Enum) {
    if let Some(name) = &node.name {
        v.visit_name(arenas, name);
    }
    for variant_id in &node.variants {
        v.visit_variant(arenas, arenas.variants.get(*variant_id));
    }
}

pub fn walk_variant<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &Variant) {
    v.visit_name(arenas, &node.name);
    if let Some(value) = &node.value {
        v.visit_expression(arenas, value);
    }
}

pub fn walk_struct_declaration<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &StructDeclaration) {
    walk_specifiers(v, arenas, &node.specifiers);
    for declarator in &node.struct_declarators {
        v.visit_struct_declarator(arenas, declarator);
    }
}

pub fn walk_struct_declarator<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, node: &StructDeclarator) {
    v.visit_declarator(arenas, &node.declarator);
    if let Some(bit_width) = &node.bit_width {
        v.visit_expression(arenas, bit_width);
    }
}

pub fn walk_specifiers<V: Visitor + ?Sized>(v: &mut V, arenas: &Arenas, specifiers: &[DeclarationSpecifier]) {
    for spec in specifiers {
        if let DeclarationSpecifier::Type(type_specifier) = spec {
            match type_specifier {
                TypeSpecifier::Struct(id) => v.visit_struct(arenas, arenas.structs.get(*id)),
                TypeSpecifier::Union(id) => v.visit_union(arenas, arenas.unions.get(*id)),
                TypeSpecifier::Enum(id) => v.visit_enum(arenas, arenas.enums.get(*id)),
                TypeSpecifier::TypedefName(name) => v.visit_name(arenas, name),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        CompoundStatementNode, ExternalDeclarationNode, FunctionDefinitionNode, FunctionParametersNode,
        InitDeclaratorNode, JumpStatementNode, StructDeclaration, StructDeclarator, TranslationUnitNode,
    };
    use crate::parser::{Context, Span, YYToken};

    fn name(ctx: &mut Context, s: &str) -> Name {
        ctx.arenas.names.add(s.to_string(), Span::default())
    }

    fn int_specifiers() -> Vec<DeclarationSpecifier> {
        vec![DeclarationSpecifier::Type(TypeSpecifier::Int)]
    }

    fn sample_ctx() -> Context {
        let mut ctx = Context::default();
        let span = Span::default();

        let s_tag = name(&mut ctx, "S");
        let a = name(&mut ctx, "a");
        let struct_decl = StructDeclaration::new(
            int_specifiers(),
            vec![StructDeclarator {
                span,
                declarator: ctx.arenas.declarators.ident(a, span),
                bit_width: None,
            }],
            span,
        );
        let sid = ctx.arenas.structs.add(Some(s_tag), vec![struct_decl], span);

        let x = name(&mut ctx, "x");
        let x_decl = DeclarationNode::new(
            int_specifiers(),
            vec![InitDeclaratorNode {
                span,
                declarator: ctx.arenas.declarators.ident(x.clone(), span),
                initializer: None,
            }],
            span,
        );

        let lhs = ctx.arenas.expressions.identifier(x, span);
        let one = name(&mut ctx, "1");
        let rhs = ctx.arenas.expressions.constant(one, span);
        let sum = ctx.arenas.expressions.binary(lhs, YYToken::Char('+'), rhs, span);
        let ret = ctx
            .arenas
            .statements
            .jump(JumpStatementNode::new_return(Some(sum), span), span);
        let body = CompoundStatementNode::new(Vec::new(), vec![ret], span);
        let f = name(&mut ctx, "f");
        let f_ident = ctx.arenas.declarators.ident(f, span);
        let f_decl = ctx
            .arenas
            .declarators
            .function(f_ident, FunctionParametersNode::empty(span), span);
        let f_def = FunctionDefinitionNode::new(int_specifiers(), f_decl, Vec::new(), body, span);

        let struct_declaration = DeclarationNode::new(
            vec![DeclarationSpecifier::Type(TypeSpecifier::Struct(sid))],
            Vec::new(),
            span,
        );

        ctx.ast = TranslationUnitNode::new(
            vec![
                ExternalDeclarationNode::declaration(struct_declaration, span),
                ExternalDeclarationNode::declaration(x_decl, span),
                ExternalDeclarationNode::function(f_def, span),
            ],
            span,
        );
        ctx
    }

    #[derive(Default)]
    struct NameRecorder {
        names: Vec<String>,
    }

    impl Visitor for NameRecorder {
        fn visit_name(&mut self, arenas: &Arenas, node: &Name) {
            self.names.push(arenas.names.get(node.id).to_string());
        }
    }

    #[test]
    fn walks_all_names_in_source_order() {
        let ctx = sample_ctx();
        let mut recorder = NameRecorder::default();
        walk_translation_unit(&mut recorder, &ctx.arenas, &ctx.ast);
        assert_eq!(recorder.names, ["S", "a", "x", "f", "x", "1"]);
    }

    #[derive(Default)]
    struct FunctionOnlyRecorder {
        names: Vec<String>,
        functions: usize,
    }

    impl Visitor for FunctionOnlyRecorder {
        fn visit_name(&mut self, arenas: &Arenas, node: &Name) {
            self.names.push(arenas.names.get(node.id).to_string());
        }

        fn visit_function_definition(&mut self, _arenas: &Arenas, _node: &FunctionDefinitionNode) {
            self.functions += 1;
        }
    }

    #[test]
    fn overriding_function_definition_prunes_body() {
        let ctx = sample_ctx();
        let mut recorder = FunctionOnlyRecorder::default();
        walk_translation_unit(&mut recorder, &ctx.arenas, &ctx.ast);
        assert_eq!(recorder.functions, 1);
        assert_eq!(recorder.names, ["S", "a", "x"]);
    }

    #[derive(Default)]
    struct ExprRecorder {
        adds: usize,
    }

    impl Visitor for ExprRecorder {
        fn visit_expression(&mut self, arenas: &Arenas, node: &ExpressionNode) {
            if matches!(arenas.expressions.get(node.id), Expression::Add(..)) {
                self.adds += 1;
            }
            walk_expression(self, arenas, node);
        }
    }

    #[test]
    fn override_can_recurse_manually() {
        let ctx = sample_ctx();
        let mut recorder = ExprRecorder::default();
        walk_translation_unit(&mut recorder, &ctx.arenas, &ctx.ast);
        assert_eq!(recorder.adds, 1);
    }
}
