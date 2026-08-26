use std::collections::HashMap;

use crate::ast::visit::{Visitor, walk_expression};
use crate::ast::{
    Declarator, DeclaratorNode, EnumId, Expression, ExpressionId, ExpressionNode, Name, StructDeclaration, Tag, Value,
};
use crate::parser::{Context, Span};
use crate::semantic::diagnosis::Diagnosis;
use crate::semantic::symbol::Symbol;
use crate::semantic::{
    Diag, DiagCollector, DiagnosisNode, QualifiedType, ResolvedType, ResolvedTypeArena, ResolvedTypeId, ScopeKind,
    Scopes, SymbolArena, SymbolId, SymbolKind, TagDefArena, TagDefId, constrain, layout,
};
use crate::target::{Layout, Target};

#[derive(Default, Debug)]
pub struct Sema {
    pub scopes: Scopes,
    pub diagnosis: Vec<DiagnosisNode>,
    pub symbols: SymbolArena,
    pub types: ResolvedTypeArena,
    pub tags: TagDefArena,
    pub bindings: HashMap<ExpressionId, Option<SymbolId>>,
    pub const_values: HashMap<ExpressionId, Option<Value>>,
    pub layouts: HashMap<ResolvedTypeId, Layout>,
    pub target: Target,
}

impl DiagCollector for Sema {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        &mut self.diagnosis
    }
}

impl Sema {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            ..Self::default()
        }
    }

    pub fn into_context(self, mut ctx: Context) -> Context {
        let Sema {
            scopes: _,
            diagnosis,
            symbols,
            types,
            tags,
            bindings,
            const_values,
            layouts: _,
            target: _,
        } = self;
        ctx.arenas.symbols = symbols;
        ctx.arenas.resolved_type = types;
        ctx.arenas.tags = tags;
        ctx.bindings = bindings;
        ctx.const_values = const_values;
        ctx.diagnosis.extend(diagnosis);
        ctx
    }

    // ----- Resolution --------------------

    pub fn resolve_typedef(
        &mut self,
        name: Name,
        is_const: bool,
        is_volatile: bool,
        span: &Span,
    ) -> Option<QualifiedType> {
        let sym_id = self.scopes.lookup_ordinary(name.id)?;
        let sym = self.symbols.get(sym_id);
        if sym.kind != SymbolKind::Typedef {
            return self.add_diag(Diag::none_diag(Diagnosis::UndeclaredIdentifier(name)), span);
        }
        let base = sym.ty?;
        if (is_const && base.is_const) || (is_volatile && base.is_volatile) {
            self.add_diag(Diag::only_diag(Diagnosis::DuplicateTypeQualifers), span)
        }
        Some(QualifiedType::new(
            base.ty,
            base.is_const || is_const,
            base.is_volatile || is_volatile,
        ))
    }

    pub fn make_qualified_type(
        &mut self,
        ctx: &Context,
        inner_most: Option<QualifiedType>,
        decl: &DeclaratorNode,
    ) -> Option<(QualifiedType, DeclaratorNode)> {
        Some(self.extract_declarator(ctx, decl, inner_most?))
    }

    fn extract_declarator(
        &mut self,
        ctx: &Context,
        declarator: &DeclaratorNode,
        inner_most: QualifiedType,
    ) -> (QualifiedType, DeclaratorNode) {
        match declarator.id.resolve(ctx) {
            Declarator::Pointer { qualifiers, inner } => {
                let (qty, decl) = self.extract_declarator(ctx, inner, inner_most);
                let (is_const, is_volatile) =
                    constrain::declaration::check_qualifier(qualifiers).collect(self, &declarator.span);
                let id = self.types.pointer(qty);
                (QualifiedType::new(id, is_const, is_volatile), decl)
            }
            Declarator::Array { declarator, size } => {
                let len = size.as_ref().and_then(|e| {
                    self.visit_expression(ctx, e);
                    match self.const_values[&e.id]?.get_integer_value() {
                        Some(v) => Some(v as u32),
                        None => self.add_diag(Diag::none_diag(Diagnosis::NonIntegerConstantExpression), &e.span)?,
                    }
                });
                let (qty, decl) = self.extract_declarator(ctx, declarator, inner_most);
                let id = self.types.array(qty, len);
                (QualifiedType::new(id, false, false), decl)
            }
            _ => (inner_most, declarator.clone()),
        }
    }

    fn declare_tag(&mut self, kind: Tag, name: Option<Name>, is_definition: bool, span: &Span) -> TagDefId {
        let Some(name) = name else { return self.tags.declare(kind, None) };

        if let Some(id) = self.scopes.lookup_tag(name.id, is_definition) {
            let def = self.tags.get(id);
            if def.kind != kind || (is_definition && def.is_complete) {
                self.add_diag(Diag::only_diag(Diagnosis::DuplicateDeclaration(def.kind(), name)), span)
            }
            return id;
        }

        let id = self.tags.declare(kind, Some(name));
        self.scopes.insert_tag(name.id, id);
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
                        Diag::only_diag(Diagnosis::DuplicateDeclaration(SymbolKind::Member, name)),
                        &decl.span,
                    );
                    continue;
                }
                if let Some(expr) = &declarator.bit_width {
                    self.visit_expression(ctx, expr);
                    constrain::declaration::check_bit_width(
                        self.types.get(ty.ty),
                        self.const_values.get(&expr.id).cloned().flatten(),
                    )
                    .collect(self, span);
                }
                let bit_width = declarator
                    .bit_width
                    .clone()
                    .and_then(|e| self.const_values.get(&e.id))
                    .cloned()
                    .flatten()
                    .and_then(|v| v.get_integer_value())
                    .map(|v| v as i32);
                let memb = Symbol::member(name, ty, bit_width);
                members.push(self.symbols.alloc(memb))
            }
        }
        self.tags.complete(tag, members);
        tag
    }

    fn variant_value(&mut self, ctx: &Context, expr: &ExpressionNode) -> Diag<Option<i64>> {
        self.visit_expression(ctx, expr);

        let Some(val) = self.const_values.get(&expr.id).cloned().flatten() else { return Diag::none() };
        match val.get_integer_value() {
            Some(v) => Diag::some(v as i64),
            None => Diag::none_diag(Diagnosis::NonIntegerConstantExpression),
        }
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
            if let Some(expr) = &variant.value
                && let Some(v) = self.variant_value(ctx, expr).collect(self, &expr.span)
            {
                value = v;
            }
            if value < i32::MIN as i64 || value > i32::MAX as i64 {
                self.add_diag(Diag::only_diag(Diagnosis::VariantBadValue), &variant.span);
                value = 0
            }
            let ty = QualifiedType::new(self.types.int(), false, false);
            members.push(self.declare(Symbol::variant(variant.name, ty, value as i32), &variant.span));
            value += 1;
        }
        self.tags.complete(tag, members);
        Some(tag)
    }

    fn dedup(&mut self, sym: &Symbol, span: &Span) -> Option<SymbolId> {
        let old_id = self.scopes.current(sym.kind, sym.name.id)?;
        let old_symbol = self.symbols.get(old_id);
        if self.scopes.kind() == ScopeKind::File
            && old_symbol.is_compatible(sym)
            && !(sym.is_init && old_symbol.is_init)
        {
            return Some(old_id);
        }
        self.add_diag(
            Diag::some_diag(old_id, Diagnosis::DuplicateDeclaration(sym.kind, sym.name)),
            span,
        )
    }

    pub fn declare(&mut self, sym: Symbol, span: &Span) -> SymbolId {
        let (name, kind) = (sym.name, sym.kind);
        let sym_id = match self.dedup(&sym, span) {
            Some(id) => id,
            None => self.symbols.alloc(sym),
        };
        self.scopes.insert(kind, name.id, sym_id);
        sym_id
    }

    pub fn add_label_symbol(&mut self, name: Name, span: &Span, is_init: bool) {
        if let Some(old) = self.scopes.lookup_label(name.id) {
            let old_init = self.symbols.get(old).is_init;
            if !old_init && is_init {
                self.symbols.get_mut(old).is_init = true;
                return;
            }
            if !(is_init && old_init) {
                return;
            }
            return self.add_diag(
                Diag::only_diag(Diagnosis::DuplicateDeclaration(SymbolKind::Label, name)),
                span,
            );
        };
        let sym_id = self.symbols.add(name, None, None, SymbolKind::Label, is_init);
        self.scopes.insert(SymbolKind::Label, name.id, sym_id);
    }

    fn cast_value(&mut self, qualif: QualifiedType, val: Value, span: &Span) -> Option<Value> {
        let ty = *self.types.get(qualif.ty);
        if let ResolvedType::Tag(id) = ty {
            return match self.tags.get(id).kind {
                Tag::Enum => self.target.cast(&ResolvedType::Int, val),
                _ => self.add_diag(Diag::none_diag(Diagnosis::CastToNonScalar), span),
            };
        }
        if let Some(casted) = self.target.cast(&ty, val) {
            return Some(casted);
        }
        match ty {
            ResolvedType::Array { .. } => self.add_diag(Diag::none_diag(Diagnosis::CastToNonScalar), span),
            _ => self.add_diag(Diag::none_diag(Diagnosis::NonIntegerConstantExpression), span),
        }
    }

    pub fn const_eval(&mut self, ctx: &Context, expr: &ExpressionNode) -> Option<Value> {
        match expr.id.resolve(ctx) {
            Expression::ConstantExpression(expr) => self.const_eval(ctx, expr),
            Expression::Identifier(_) => {
                let id = self.bindings.get(&expr.id).copied().flatten()?;
                let s = self.symbols.get(id);
                if s.kind != SymbolKind::Variant {
                    return None;
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
            Expression::SizeofExpr(_expr) => unimplemented!(),
            Expression::SizeofType(ty) => {
                let qualif = constrain::declaration::resolve_type(self, ctx, &ty.specifiers, &expr.span)?;
                match layout::of(self, qualif) {
                    Ok(layout) => Some(Value::UnsignedLong(layout.size.into())),
                    Err(diagnosis) => self.add_diag(Diag::none_diag(diagnosis), &expr.span),
                }
            }
            Expression::Cast(ty, operand) => {
                let base = constrain::declaration::resolve_type(self, ctx, &ty.specifiers, &expr.span);
                let (qualif, _) = self.make_qualified_type(ctx, base, &ty.declarator)?;
                let val = self.const_eval(ctx, operand)?;
                if val.is_floating() && !matches!(operand.id.resolve(ctx), Expression::Constant(_)) {
                    return None;
                }
                self.cast_value(qualif, val, &expr.span)
            }
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
            | Expression::PreDec(_) => None,
        }
    }
}

impl Visitor for Sema {
    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        if self.const_values.contains_key(&node.id) || self.bindings.contains_key(&node.id) {
            return;
        }
        walk_expression(self, ctx, node);
        match node.id.resolve(ctx) {
            Expression::ConstantExpression(expr) => {
                let value = self.const_eval(ctx, expr);
                if value.is_none() {
                    self.add_diag(Diag::only_diag(Diagnosis::NonConstantExpression), &node.span);
                }
                self.const_values.insert(node.id, value);
            }
            Expression::Identifier(name) => {
                let sym = self.scopes.lookup_ordinary(name.id);
                if sym.is_none() {
                    self.add_diag(Diag::only_diag(Diagnosis::UndeclaredIdentifier(*name)), &node.span);
                }
                self.bindings.insert(node.id, sym);
            }
            _ => (),
        }
    }
}
