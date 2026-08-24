use std::collections::HashMap;

use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_declaration, walk_expression, walk_jump_statement, walk_labeled_statement,
    walk_translation_unit,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, EnumId, Expression,
    ExpressionId, ExpressionNode, FunctionDefinitionNode, FunctionParameters, FunctionParametersNode, JumpStatement,
    JumpStatementNode, Labeled, LabeledStatementNode, Name, ParameterDeclaration, StructDeclaration, Tag, Value,
};
use crate::ast::{Storage, StringId};
use crate::parser::{Context, Span};
use crate::semantic::diagnosis::Diagnosis;
use crate::semantic::symbol::Symbol;
use crate::semantic::{
    Diag, DiagCollector, DiagnosisNode, QualifiedType, ResolvedType, ResolvedTypeArena, SymbolArena, SymbolId,
    SymbolKind, TagDefArena, TagDefId, constrain,
};
use crate::utils::{BLUE, RESET};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Block,
    Function,
    File,
    Prototype,
}

#[derive(Debug)]
struct Scope {
    tags: HashMap<StringId, TagDefId>,
    labels: HashMap<StringId, SymbolId>,
    ordinaries: HashMap<StringId, SymbolId>,
    kind: ScopeKind,
}

impl Scope {
    pub fn new(kind: ScopeKind) -> Self {
        Self {
            tags: HashMap::new(),
            labels: HashMap::new(),
            ordinaries: HashMap::new(),
            kind,
        }
    }
}

#[derive(Default, Debug)]
pub struct SymbolResolver {
    scopes: Vec<Scope>,
    diagnosis: Vec<DiagnosisNode>,
    symbols: SymbolArena,
    pub types: ResolvedTypeArena,
    tags: TagDefArena,
    bindings: HashMap<ExpressionId, Option<SymbolId>>,
}

impl DiagCollector for SymbolResolver {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        &mut self.diagnosis
    }
}

impl SymbolResolver {
    fn symbol_size(&mut self, ctx: &Context, id: SymbolId) -> Option<u64> {
        let symbol = self.symbols.get(id);
        self.type_size(ctx, symbol.ty?)
    }

    fn type_size(&mut self, ctx: &Context, qualified_type: QualifiedType) -> Option<u64> {
        let ty = self.types.get(qualified_type.ty);
        match ty {
            ResolvedType::Tag(id) => self.tag_size(ctx, *id),
            _ => Some(ctx.target.scalar(ty)?.size as u64),
        }
    }

    fn tag_size(&mut self, ctx: &Context, id: TagDefId) -> Option<u64> {
        let tag = self.tags.get(id).clone();
        if !tag.is_complete {
            return self.add_diag(Diag::none_diag(Diagnosis::InvalidSizeof), &Span::default());
        }
        match tag.kind {
            Tag::Struct => self.struct_size(ctx, &tag.members),
            Tag::Union => tag.members.iter().filter_map(|id| self.symbol_size(ctx, *id)).max(),
            Tag::Enum => Some(ctx.target.int.size.into()),
        }
    }

    fn struct_size(&mut self, ctx: &Context, members: &[SymbolId]) -> Option<u64> {
        let layouts = members
            .iter()
            .map(|id| {
                let sym = self.symbols.get(*id);
                if !sym.is_complete {
                    return None;
                }
                let t = self.types.get(sym.ty?.ty);
                ctx.target.scalar(t)
            })
            .collect::<Option<Vec<_>>>()?;
        let align = layouts.iter().map(|l| l.align).max()?;
        let mut size = 0;
        for l in &layouts {
            if l.size > align - size % align {
                size += size % align
            }
            size += l.size;
        }
        size += size % align;
        Some(size as u64)
    }

    fn scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    fn scope(&self) -> &Scope {
        self.scopes.last().unwrap()
    }

    fn get_map_mut(&mut self, kind: SymbolKind) -> &mut HashMap<StringId, SymbolId> {
        match kind {
            SymbolKind::Label => &mut self.scope_mut().labels,
            SymbolKind::Typedef
            | SymbolKind::Variable
            | SymbolKind::Parameter
            | SymbolKind::Function
            | SymbolKind::Variant => &mut self.scope_mut().ordinaries,
            _ => unimplemented!(),
        }
    }

    fn get_map(&self, kind: SymbolKind) -> &HashMap<StringId, SymbolId> {
        match kind {
            SymbolKind::Label => &self.scope().labels,
            SymbolKind::Typedef
            | SymbolKind::Variable
            | SymbolKind::Parameter
            | SymbolKind::Function
            | SymbolKind::Variant => &self.scope().ordinaries,
            _ => unimplemented!(),
        }
    }

    fn lookup_ordinary(&self, name: StringId) -> Option<SymbolId> {
        self.scopes.iter().rev().find_map(|s| s.ordinaries.get(&name)).copied()
    }

    pub fn resolve_typedef(
        &mut self,
        name: Name,
        is_const: bool,
        is_volatile: bool,
        span: &Span,
    ) -> Option<QualifiedType> {
        let sym_id = self.lookup_ordinary(name.id)?;
        let sym = self.symbols.get(sym_id);
        if sym.kind != SymbolKind::Typedef {
            self.diagnosis
                .push(DiagnosisNode::new(Diagnosis::UndeclaredIdentifier(name), *span));
            return None;
        }
        let base = sym.ty?;
        if (is_const && base.is_const) || (is_volatile && base.is_volatile) {
            self.diagnosis
                .push(DiagnosisNode::new(Diagnosis::DuplicateTypeQualifers, *span));
        }
        Some(QualifiedType::new(
            base.ty,
            base.is_const || is_const,
            base.is_volatile || is_volatile,
        ))
    }

    fn make_qualified_type(
        &mut self,
        ctx: &Context,
        inner_most: Option<QualifiedType>,
        decl: &DeclaratorNode,
    ) -> Option<(QualifiedType, DeclaratorNode)> {
        let (ty, decl) = self.extract_pointer(ctx, decl, inner_most?);
        // match decl.id.resolve(ctx) {
        //     Declarator::Ident(name) => (),
        //     Declarator::Abstract => (),
        //     Declarator::Pointer { qualifiers, inner } => (),
        //     Declarator::Array { declarator, size } => (),
        //     Declarator::Function { declarator, params } => (),
        // }
        Some((ty, decl))
    }

    fn extract_pointer(
        &mut self,
        ctx: &Context,
        declarator: &DeclaratorNode,
        inner_most: QualifiedType,
    ) -> (QualifiedType, DeclaratorNode) {
        match declarator.id.resolve(ctx) {
            Declarator::Pointer { qualifiers, inner } => {
                let (qty, decl) = self.extract_pointer(ctx, inner, inner_most);
                let (is_const, is_volatile) =
                    constrain::declaration::check_qualifier(qualifiers).collect(self, &declarator.span);
                let id = self.types.pointer(qty);
                (QualifiedType::new(id, is_const, is_volatile), decl)
            }
            _ => (inner_most, declarator.clone()),
        }
    }

    fn declare_tag(&mut self, kind: Tag, name: Option<Name>, is_definition: bool, span: &Span) -> TagDefId {
        let Some(name) = name else { return self.tags.declare(kind, None) };

        let found = if is_definition {
            self.scope().tags.get(&name.id).copied()
        } else {
            self.scopes.iter().rev().find_map(|s| s.tags.get(&name.id)).copied()
        };

        if let Some(id) = found {
            let def = self.tags.get(id);
            if def.kind != kind || (is_definition && def.is_complete) {
                let kind = def.kind();
                self.diagnosis
                    .push(DiagnosisNode::new(Diagnosis::DuplicateDeclaration(kind, name), *span));
            }
            return id;
        }

        let id = self.tags.declare(kind, Some(name));
        self.scope_mut().tags.insert(name.id, id);
        id
    }

    pub fn resolve_struct_or_union(
        &mut self,
        ctx: &Context,
        kind: Tag,
        name: Option<Name>,
        fields: &[StructDeclaration],
        span: &Span,
    ) -> TagDefId {
        let is_definition = !fields.is_empty();
        let tag = self.declare_tag(kind, name, is_definition, span);
        if !is_definition {
            return tag;
        }

        let mut members = Vec::new();
        for field in fields {
            let qual = constrain::declaration::resolve_type(self, ctx, &field.specifiers, &field.span);
            for declarator in &field.struct_declarators {
                let decl = &declarator.declarator;
                let Some((ty, node)) = self.make_qualified_type(ctx, qual, decl) else { continue };
                let Some(name) = node.ident(ctx) else { continue };
                if members.iter().any(|&m| self.symbols.get(m).name.id == name.id) {
                    self.add_diag(
                        Diag::with_diag((), Diagnosis::DuplicateDeclaration(SymbolKind::Member, name)),
                        &decl.span,
                    );
                    continue;
                }
                members.push(self.symbols.with_size(
                    name,
                    Some(ty),
                    None,
                    SymbolKind::Member,
                    declarator.bit_width.clone(),
                ));
            }
        }
        self.tags.complete(tag, members);
        tag
    }

    pub fn resolve_enum(&mut self, ctx: &Context, id: EnumId) -> Option<TagDefId> {
        let enum_node = id.resolve(ctx);
        let is_definition = !enum_node.variants.is_empty();
        let tag = self.declare_tag(Tag::Enum, enum_node.name, is_definition, &enum_node.span);
        if !is_definition {
            return Some(tag);
        }

        let mut members = Vec::new();
        let mut value: i64 = 0;

        for variant_id in &enum_node.variants {
            let variant = variant_id.resolve(ctx);
            let ty = QualifiedType::new(self.types.int(), false, false);
            if let Some(expr) = &variant.value {
                self.visit_expression(ctx, expr);
                if let Some(val) = self.const_eval(ctx, expr) {
                    match val.get_integer_value() {
                        Some(v) => value = v as i64,
                        None => {
                            self.add_diag(
                                Diag::with_diag((), Diagnosis::NonIntegerConstantExpression),
                                &variant.span,
                            );
                        }
                    }
                }
            }
            if value < i32::MIN as i64 || value > i32::MAX as i64 {
                self.add_diag(Diag::with_diag((), Diagnosis::VariantBadValue), &variant.span);
                value = 0
            }
            members.push(self.add_variant_symbol(
                variant.name,
                ty,
                None,
                SymbolKind::Variant,
                &variant.span,
                true,
                value as i32,
            ));
            value += 1;
        }
        self.tags.complete(tag, members);
        Some(tag)
    }

    fn dedup(&mut self, sym: &Symbol, span: &Span) -> Option<SymbolId> {
        let old_id = *self.get_map(sym.kind).get(&sym.name.id)?;
        let old_symbol = self.symbols.get(old_id);
        if self.scope().kind == ScopeKind::File && old_symbol.is_compatible(sym) && !(sym.is_init && old_symbol.is_init)
        {
            return Some(old_id);
        }
        self.add_diag(
            Diag::some_diag(old_id, Diagnosis::DuplicateDeclaration(sym.kind, sym.name)),
            span,
        )
    }

    fn add_symbol(
        &mut self,
        name: Name,
        ty: QualifiedType,
        storage: Option<Storage>,
        kind: SymbolKind,
        span: &Span,
        is_init: bool,
    ) -> SymbolId {
        let sym = Symbol {
            name,
            ty: Some(ty),
            storage,
            kind,
            bit_width: None,
            is_complete: true,
            value: None,
            is_init,
        };
        let sym_id = self.dedup(&sym, span).unwrap_or(self.symbols.alloc(sym));
        self.get_map_mut(kind).insert(name.id, sym_id);
        sym_id
    }

    fn add_variant_symbol(
        &mut self,
        name: Name,
        ty: QualifiedType,
        storage: Option<Storage>,
        kind: SymbolKind,
        span: &Span,
        is_init: bool,
        value: i32,
    ) -> SymbolId {
        let sym = Symbol {
            name,
            ty: Some(ty),
            storage,
            kind,
            bit_width: None,
            is_complete: true,
            is_init,
            value: Some(value),
        };
        let sym_id = self.dedup(&sym, span).unwrap_or(self.symbols.alloc(sym));
        self.get_map_mut(kind).insert(name.id, sym_id);
        sym_id
    }

    fn add_label_symbol(&mut self, name: Name, span: &Span, is_init: bool) {
        if let Some(&old) = self.scopes.iter().rev().find_map(|s| s.labels.get(&name.id)) {
            let old_init = self.symbols.get(old).is_init;
            if !old_init && is_init {
                self.symbols.get_mut(old).is_init = true;
                return;
            }
            if !(is_init && old_init) {
                return;
            }
            return self.add_diag(
                Diag::with_diag((), Diagnosis::DuplicateDeclaration(SymbolKind::Label, name)),
                span,
            );
        };
        let sym_id = self.symbols.add(name, None, None, SymbolKind::Label, is_init);
        self.scopes
            .iter_mut()
            .find(|s| s.kind == ScopeKind::Function)
            .unwrap()
            .labels
            .insert(name.id, sym_id);
    }

    fn add_function<'a>(
        &mut self,
        ctx: &'a Context,
        node: &FunctionDefinitionNode,
    ) -> Option<&'a FunctionParametersNode> {
        let span = &node.span;

        let qualif = constrain::declaration::resolve_type(self, ctx, &node.specifiers, span);
        let (ty, fn_decl) = self.make_qualified_type(ctx, qualif, &node.declarator)?;
        let (decl, parameters) =
            constrain::external::extract_function_declarator(fn_decl.id.resolve(ctx)).collect(self, span)?;

        let storage = constrain::declaration::get_storage(&node.specifiers)
            .collect(self, span)
            .unwrap_or(Storage::Extern);

        constrain::external::check_function_storage(storage).collect(self, span);
        constrain::external::check_external_specifiers(&node.specifiers).collect(self, span);

        let name = decl.ident(ctx)?;
        self.add_symbol(
            name,
            ty,
            Some(storage),
            SymbolKind::Function,
            &node.declarator.span,
            true,
        );
        Some(parameters)
    }

    fn param_empty(&mut self, lst: &[DeclarationNode], span: &Span) {
        if !lst.is_empty() {
            self.diagnosis
                .push(DiagnosisNode::new(Diagnosis::ParameterTypeListWithList, *span))
        }
    }

    #[rustfmt::skip]
    fn param_ident_style(
        &mut self,
        ctx: &Context,
        params: &[ParameterDeclaration],
        lst: &[DeclarationNode],
        span: &Span,
    ) {
        let a = constrain::external::is_valid_parameter_style(ctx, params, lst);
        match a {
            Diag { res: true, diagnosis: None, } => (),
            Diag { res: false, diagnosis: None, } => return,
            _ => { let _ = a.collect(self, span); }
        }

        for param in params {
            let ParameterDeclaration {
                span,
                specifiers,
                declarator,
            } = param;
            self.add_parameter_declarator(ctx, specifiers, declarator, span);
        }
    }

    fn param_old_style(&mut self, ctx: &Context, names: &[Name], lst: &[DeclarationNode], span: &Span) {
        // if names
        //     .iter()
        //     .any(|name| !constrain::external::check_typedef(ctx, name).collect(self, span))
        // {
        //     return;
        // }
        let declarations: Vec<_> = lst
            .iter()
            .flat_map(
                |DeclarationNode {
                     span,
                     specifiers,
                     init_declarators,
                 }| {
                    init_declarators
                        .iter()
                        .map(|decl| {
                            self.add_parameter_declarator(ctx, specifiers, &decl.declarator, span)
                                .map(|sym_id| self.symbols.get(sym_id).name.id)
                        })
                        .collect::<Vec<_>>()
                },
            )
            .collect();

        let names_id: Vec<_> = names.iter().map(|n| n.id).collect();

        let Some(missing_id) = constrain::external::is_valid_old_style(&names_id, declarations).collect(self, span)
        else {
            return;
        };
        for string_id in missing_id {
            let ty_id = self.types.int();
            self.add_symbol(
                Name::new(string_id, Span::default()),
                QualifiedType::new(ty_id, false, false),
                Some(Storage::Register),
                SymbolKind::Parameter,
                &Span::default(),
                false,
            );
        }
    }

    fn add_parameter_declarator(
        &mut self,
        ctx: &Context,
        specifiers: &[DeclarationSpecifier],
        decl: &DeclaratorNode,
        span: &Span,
    ) -> Option<SymbolId> {
        let qualif = constrain::declaration::resolve_type(self, ctx, specifiers, span);
        let (ty, decl) = self.make_qualified_type(ctx, qualif, decl)?;
        let storage = constrain::declaration::get_storage(specifiers)
            .collect(self, span)
            .unwrap_or(Storage::Register);
        constrain::external::param_storage_only_register(storage).collect(self, span)?;
        let name = decl.ident(ctx)?;
        Some(self.add_symbol(name, ty, Some(storage), SymbolKind::Parameter, &decl.span, false))
    }

    pub fn const_eval(&mut self, ctx: &Context, expr: &ExpressionNode) -> Option<Value> {
        match expr.id.resolve(ctx) {
            Expression::ConstantExpression(expr) => self.const_eval(ctx, expr),
            Expression::Identifier(_) => {
                let id = self.bindings.get(&expr.id).copied().flatten()?;
                let s = self.symbols.get(id);
                if s.kind != SymbolKind::Variant {
                    return self.add_diag(Diag::none_diag(Diagnosis::NonConstantExpression), &expr.span);
                }
                s.value.map(Value::Int)
            }
            Expression::Constant(value_node) => Some(value_node.value),
            Expression::Plus(expr) => self.const_eval(ctx, expr),
            Expression::Minus(expr) => Some(-self.const_eval(ctx, expr)?),
            Expression::BitNot(expr) => Some(!self.const_eval(ctx, expr)?),
            Expression::LogicalNot(expr) => Some(self.const_eval(ctx, expr)?.logical_not()),
            Expression::Add(e1, e2) => Some(self.const_eval(ctx, e1)? + self.const_eval(ctx, e2)?),
            Expression::Sub(e1, e2) => Some(self.const_eval(ctx, e1)? - self.const_eval(ctx, e2)?),
            Expression::Mul(e1, e2) => Some(self.const_eval(ctx, e1)? * self.const_eval(ctx, e2)?),
            Expression::Div(e1, e2) => Some(self.const_eval(ctx, e1)? / self.const_eval(ctx, e2)?),
            Expression::Mod(e1, e2) => Some(self.const_eval(ctx, e1)? % self.const_eval(ctx, e2)?),
            Expression::Left(e1, e2) => Some(self.const_eval(ctx, e1)? << self.const_eval(ctx, e2)?),
            Expression::Right(e1, e2) => Some(self.const_eval(ctx, e1)? >> self.const_eval(ctx, e2)?),
            Expression::BitAnd(e1, e2) => Some(self.const_eval(ctx, e1)? & self.const_eval(ctx, e2)?),
            Expression::BitOr(e1, e2) => Some(self.const_eval(ctx, e1)? | self.const_eval(ctx, e2)?),
            Expression::BitXor(e1, e2) => Some(self.const_eval(ctx, e1)? ^ self.const_eval(ctx, e2)?),
            Expression::Greater(e1, e2) => Some(Value::from(self.const_eval(ctx, e1)? > self.const_eval(ctx, e2)?)),
            Expression::Lower(e1, e2) => Some(Value::from(self.const_eval(ctx, e1)? < self.const_eval(ctx, e2)?)),
            Expression::GreaterEq(e1, e2) => Some(Value::from(self.const_eval(ctx, e1)? >= self.const_eval(ctx, e2)?)),
            Expression::LowerEq(e1, e2) => Some(Value::from(self.const_eval(ctx, e1)? <= self.const_eval(ctx, e2)?)),
            Expression::Eq(e1, e2) => Some(Value::from(self.const_eval(ctx, e1)? == self.const_eval(ctx, e2)?)),
            Expression::Neq(e1, e2) => Some(Value::from(self.const_eval(ctx, e1)? != self.const_eval(ctx, e2)?)),
            Expression::LogicalAnd(e1, e2) => Some(Value::from(
                self.const_eval(ctx, e1)?.is_true() && self.const_eval(ctx, e2)?.is_true(),
            )),
            Expression::LogicalOr(e1, e2) => Some(Value::from(
                self.const_eval(ctx, e1)?.is_true() || self.const_eval(ctx, e2)?.is_true(),
            )),
            Expression::Ternary(condition, e1, e2) => {
                if self.const_eval(ctx, condition)?.is_true() {
                    self.const_eval(ctx, e1)
                } else {
                    self.const_eval(ctx, e2)
                }
            }
            Expression::SizeofExpr(_expr) => None,
            Expression::SizeofType(ty) => {
                let qualif = constrain::declaration::resolve_type(self, ctx, &ty.specifiers, &expr.span)?;
                Some(Value::UnsignedLong(self.type_size(ctx, qualif)?))
            }
            // Expression::Cast(ty, expr) => {
            //     let val = self.const_eval(ctx, expr)?;
            //     None
            // }
            Expression::StringLiteral(_)
            | Expression::PostInc(_)
            | Expression::PostDec(_)
            | Expression::PreInc(_)
            | Expression::Deref(_)
            | Expression::Addr(_)
            | Expression::Assign(_, _)
            | Expression::MulAssign(_, _)
            | Expression::DivAssign(_, _)
            | Expression::ModAssign(_, _)
            | Expression::AddAssign(_, _)
            | Expression::SubAssign(_, _)
            | Expression::LeftAssign(_, _)
            | Expression::RightAssign(_, _)
            | Expression::AndAssign(_, _)
            | Expression::XorAssign(_, _)
            | Expression::OrAssign(_, _)
            | Expression::List(_, _)
            | Expression::ArrayAcces(_, _)
            | Expression::FunctionCall(_, _)
            | Expression::DotAcces(_, _)
            | Expression::PtrAcces(_, _)
            | Expression::PreDec(_) => self.add_diag(Diag::none_diag(Diagnosis::NonConstantExpression), &expr.span),
            _ => None,
        }
    }
}

impl Visitor for SymbolResolver {
    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        let Some(parameters) = self.add_function(ctx, node) else { return };

        self.scopes.push(Scope::new(ScopeKind::Prototype));
        let span = &parameters.span;
        match &parameters.param {
            FunctionParameters::Empty => self.param_empty(&node.old_style_declarations, span),
            FunctionParameters::ParameterTypeList(params) => {
                self.param_ident_style(ctx, params, &node.old_style_declarations, span)
            }
            FunctionParameters::OldStyle(names) => self.param_old_style(ctx, names, &node.old_style_declarations, span),
            FunctionParameters::Variadic(_) => unimplemented!(),
        }

        self.visit_compound_statement(ctx, &node.body);
    }

    fn visit_declaration(&mut self, ctx: &Context, node: &DeclarationNode) {
        let specifiers = &node.specifiers;
        let span = &node.span;
        let qualif = constrain::declaration::resolve_type(self, ctx, specifiers, span);
        for init_declarator in &node.init_declarators {
            let decl = &init_declarator.declarator;
            let Some((ty, decl)) = self.make_qualified_type(ctx, qualif, decl) else { continue };
            let Some(name) = decl.ident(ctx) else { continue };
            let storage = constrain::declaration::get_storage(&node.specifiers)
                .collect(self, span)
                .unwrap_or(Storage::Auto);
            let kind = if storage == Storage::Typedef { SymbolKind::Typedef } else { SymbolKind::Variable };
            self.add_symbol(
                name,
                ty,
                Some(storage),
                kind,
                &decl.span,
                init_declarator.initializer.is_some(),
            );
        }
        walk_declaration(self, ctx, node);
    }

    fn visit_labeled_statement(&mut self, ctx: &Context, node: &LabeledStatementNode) {
        if let Labeled::Identifier(name, _) = node.inner {
            self.add_label_symbol(name, &node.span, true);
        }
        walk_labeled_statement(self, ctx, node);
    }

    fn visit_jump_statement(&mut self, ctx: &Context, node: &JumpStatementNode) {
        if let JumpStatement::Goto(name) = node.stmt {
            self.add_label_symbol(name, &node.span, false);
        }
        walk_jump_statement(self, ctx, node);
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode) {
        match self.scopes.last().unwrap().kind {
            ScopeKind::Prototype => self.scopes.last_mut().unwrap().kind = ScopeKind::Function,
            _ => self.scopes.push(Scope::new(ScopeKind::Block)),
        }
        walk_compound_statement(self, ctx, node);
        self.scopes.pop();
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        if self.bindings.contains_key(&node.id) {
            return;
        }
        match node.id.resolve(ctx) {
            Expression::Identifier(name) => {
                let sym = self.lookup_ordinary(name.id);
                if sym.is_none() {
                    self.add_diag(Diag::with_diag((), Diagnosis::UndeclaredIdentifier(*name)), &node.span);
                }
                self.bindings.insert(node.id, sym);
            }
            _ => walk_expression(self, ctx, node),
        }
    }
}

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(ctx: Context) -> Context {
        Self::resolve_names(&ctx);
        ctx
    }

    fn resolve_names(ctx: &Context) {
        let mut collector = SymbolResolver::default();
        collector.scopes.push(Scope::new(ScopeKind::File));
        walk_translation_unit(&mut collector, ctx, &ctx.ast);
        collector.scopes.pop();
        assert!(collector.scopes.is_empty());
        collector.report(ctx);
    }
}

impl SymbolResolver {
    fn describe(&self, ctx: &Context, qt: QualifiedType) -> String {
        let mut out = String::new();
        if qt.is_const {
            out.push_str("const ");
        }
        if qt.is_volatile {
            out.push_str("volatile ");
        }
        match *self.types.get(qt.ty) {
            ResolvedType::Void => out.push_str("void"),
            ResolvedType::Char => out.push_str("char"),
            ResolvedType::SignedChar => out.push_str("signed char"),
            ResolvedType::UnsignedChar => out.push_str("unsigned char"),
            ResolvedType::Short => out.push_str("short"),
            ResolvedType::UnsignedShort => out.push_str("unsigned short"),
            ResolvedType::Int => out.push_str("int"),
            ResolvedType::UnsignedInt => out.push_str("unsigned int"),
            ResolvedType::Long => out.push_str("long"),
            ResolvedType::UnsignedLong => out.push_str("unsigned long"),
            ResolvedType::Float => out.push_str("float"),
            ResolvedType::Double => out.push_str("double"),
            ResolvedType::LongDouble => out.push_str("long double"),
            ResolvedType::Pointer(inner) => {
                out.push_str("pointer to ");
                out.push_str(&self.describe(ctx, inner));
            }
            ResolvedType::Tag(id) => {
                let def = self.tags.get(id);
                let name = def.name.map(|n| n.id.resolve(ctx).as_str()).unwrap_or("<anonymous>");
                out.push_str(&format!("{} {}", def.kind(), name));
                if !def.is_complete {
                    out.push_str(" (incomplete)");
                }
            }
            ResolvedType::Array { elem, len } => {
                out.push_str("array");
                out.push_str(&self.describe(ctx, elem));
                if let Some(len) = len {
                    out.push('[');
                    out.push_str(&len.to_string());
                    out.push('[');
                }
            }
        }
        out
    }

    fn report(&self, ctx: &Context) {
        let rows: Vec<[String; 5]> = self
            .symbols
            .data
            .iter()
            .map(|symbol| {
                [
                    symbol.kind.to_string(),
                    symbol.name.id.resolve(ctx).clone(),
                    symbol.storage.map(|s| s.to_string()).unwrap_or_default(),
                    symbol.value.map(|v| v.to_string()).unwrap_or("-".to_string()),
                    symbol.ty.map(|ty| self.describe(ctx, ty)).unwrap_or_default(),
                ]
            })
            .collect();

        println!("Symbols:");
        print_table(&["KIND", "NAME", "STORAGE", "INIT", "TYPE"], &rows);

        println!("Diagnosis:");
        for diag in &self.diagnosis {
            let _ = diag.print(ctx);
        }
    }
}

fn print_table<const N: usize>(headers: &[&str; N], rows: &[[String; N]]) {
    if rows.is_empty() {
        return;
    }

    let mut widths = headers.map(str::len);
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }

    let rule: [String; N] = std::array::from_fn(|i| "-".repeat(widths[i]));

    println!("{}{}{}", BLUE, table_row(&widths, headers), RESET);
    println!("{}", table_row(&widths, &rule.each_ref().map(String::as_str)));
    for row in rows {
        println!("{}", table_row(&widths, &row.each_ref().map(String::as_str)));
    }
}

fn table_row<const N: usize>(widths: &[usize; N], cells: &[&str; N]) -> String {
    let mut out = String::new();
    for (i, cell) in cells.iter().enumerate() {
        out.push_str(cell);
        if i + 1 < N {
            out.push_str(&" ".repeat(widths[i] - cell.chars().count() + 2));
        }
    }
    out
}
