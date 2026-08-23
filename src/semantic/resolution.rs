use std::collections::HashMap;

use crate::ast::visit::{
    Visitor, walk_compound_statement, walk_declaration, walk_jump_statement, walk_labeled_statement,
    walk_translation_unit,
};
use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, EnumId, ExpressionNode,
    FunctionDefinitionNode, FunctionParameters, FunctionParametersNode, JumpStatement, JumpStatementNode, Labeled,
    LabeledStatementNode, Name, ParameterDeclaration, StructId, UnionId,
};
use crate::ast::{Storage, StringId};
use crate::parser::{Context, Span};
use crate::semantic::diagnosis::Diagnosis;
use crate::semantic::symbol::Symbol;
use crate::semantic::{
    Diag, DiagCollector, DiagnosisNode, QualifiedType, ResolvedType, ResolvedTypeArena, SymbolArena, SymbolId,
    SymbolKind, constrain,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Block,
    Function,
    File,
    Prototype,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TagId {
    StructId(StructId),
    UnionId(UnionId),
}

#[derive(Debug)]
struct Scope {
    tags: HashMap<StringId, SymbolId>,
    members: HashMap<(TagId, StringId), SymbolId>,
    labels: HashMap<StringId, SymbolId>,
    ordinaries: HashMap<StringId, SymbolId>,
    kind: ScopeKind,
}

impl Scope {
    pub fn new(kind: ScopeKind) -> Self {
        Self {
            tags: HashMap::new(),
            members: HashMap::new(),
            labels: HashMap::new(),
            ordinaries: HashMap::new(),
            kind,
        }
    }
}

#[derive(Default, Debug)]
struct SymbolResolver {
    scopes: Vec<Scope>,
    diagnosis: Vec<DiagnosisNode>,
    symbol_arena: SymbolArena,
    types: ResolvedTypeArena,
}

impl DiagCollector for SymbolResolver {
    fn diagnosis(&mut self) -> &mut Vec<DiagnosisNode> {
        &mut self.diagnosis
    }
}

impl SymbolResolver {
    fn scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    fn scope(&self) -> &Scope {
        self.scopes.last().unwrap()
    }

    fn get_map_mut(&mut self, kind: SymbolKind) -> &mut HashMap<StringId, SymbolId> {
        match kind {
            SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union => &mut self.scope_mut().tags,
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
            SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union => &self.scope().tags,
            SymbolKind::Label => &self.scope().labels,
            SymbolKind::Typedef
            | SymbolKind::Variable
            | SymbolKind::Parameter
            | SymbolKind::Function
            | SymbolKind::Variant => &self.scope().ordinaries,
            _ => unimplemented!(),
        }
    }

    fn process_specifiers(
        &mut self,
        ctx: &Context,
        specifiers: &[DeclarationSpecifier],
        span: &Span,
    ) -> Option<QualifiedType> {
        let (is_const, is_volatile) = constrain::declaration::get_qualifier(specifiers).collect(self, span);
        let ty = constrain::declaration::resolve_type(specifiers).collect(self, span)?;
        match ty {
            ResolvedType::Enum(id) => self.add_enum(ctx, id),
            ResolvedType::Struct(id) => self.add_struct_or_union(ctx, TagId::StructId(id)),
            ResolvedType::Union(id) => self.add_struct_or_union(ctx, TagId::UnionId(id)),
            _ => (),
        }
        let id = self.types.alloc(ty);
        let inner_most = QualifiedType::new(id, is_const, is_volatile);
        Some(inner_most)
    }

    fn make_qualified_type(
        &mut self,
        ctx: &Context,
        inner_most: Option<QualifiedType>,
        decl: &DeclaratorNode,
    ) -> Option<(QualifiedType, DeclaratorNode)> {
        Some(self.extract_pointer(ctx, decl, inner_most?))
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

    fn add_struct_or_union(&mut self, ctx: &Context, id: TagId) {
        let (name, fields) = match id {
            TagId::StructId(id) => (id.resolve(ctx).name, &id.resolve(ctx).fields),
            TagId::UnionId(id) => (id.resolve(ctx).name, &id.resolve(ctx).fields),
        };
        if let Some(name) = name {
            let ty = match id {
                TagId::StructId(id) => self.types.new_struct(id),
                TagId::UnionId(id) => self.types.new_union(id),
            };
            self.add_symbol(
                name,
                QualifiedType::new(ty, false, false),
                None,
                SymbolKind::Struct,
                &Span::default(),
                false,
            );
        }
        for field in fields {
            let qual = self.process_specifiers(ctx, &field.specifiers, &field.span);
            for declarator in &field.struct_declarators {
                let decl = &declarator.declarator;
                let Some((ty, node)) = self.make_qualified_type(ctx, qual, decl) else { continue };
                let Some(name) = node.ident(ctx) else { continue };
                self.add_member(name, ty, &decl.span, declarator.bit_width.clone(), id);
            }
        }
    }

    fn add_enum(&mut self, ctx: &Context, id: EnumId) {
        let enum_node = id.resolve(ctx);
        if let Some(name) = enum_node.name {
            let ty = self.types.new_enum(id);
            self.add_symbol(
                name,
                QualifiedType::new(ty, false, false),
                None,
                SymbolKind::Enum,
                &enum_node.span,
                !enum_node.variants.is_empty(),
            );
        }
        for variant_id in &enum_node.variants {
            let variant = variant_id.resolve(ctx);
            let ty = QualifiedType::new(self.types.int(), false, false);
            self.add_symbol(variant.name, ty, None, SymbolKind::Variant, &enum_node.span, true);
        }
    }

    fn dedup(&mut self, sym: &Symbol, span: &Span) -> Option<SymbolId> {
        let old_id = *self.get_map(sym.kind).get(&sym.name.id)?;
        let old_symbol = self.symbol_arena.get(old_id);
        if self.scope().kind == ScopeKind::File && old_symbol.is_compatible(sym) && !(sym.is_init && old_symbol.is_init)
        {
            return Some(old_id);
        }
        self.diagnosis.push(DiagnosisNode::new(
            Diagnosis::DuplicateDeclaration(sym.kind, sym.name),
            *span,
        ));
        Some(old_id)
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
            ty,
            storage,
            kind,
            bit_width: None,
            is_complete: true,
            is_init,
        };
        let sym_id = self
            .dedup(&sym, span)
            .unwrap_or(self.symbol_arena.add(name, ty, storage, kind, is_init));
        self.get_map_mut(kind).insert(name.id, sym_id);
        sym_id
    }

    fn add_member(
        &mut self,
        name: Name,
        ty: QualifiedType,
        span: &Span,
        bit_width: Option<ExpressionNode>,
        tag_id: TagId,
    ) {
        let sym = Symbol {
            name,
            ty,
            storage: None,
            kind: SymbolKind::Member,
            bit_width,
            is_complete: true,
            is_init: true,
        };
        if self.scope_mut().members.contains_key(&(tag_id, name.id)) {
            return self.diagnosis.push(DiagnosisNode::new(
                Diagnosis::DuplicateDeclaration(sym.kind, sym.name),
                *span,
            ));
        }
        let sym_id = self.symbol_arena.add(name, ty, None, SymbolKind::Member, true);
        self.scope_mut().members.insert((tag_id, name.id), sym_id);
    }

    fn add_label_symbol(&mut self, name: Name, span: &Span, is_init: bool) {
        if let Some(&old) = self.scopes.iter().rev().find_map(|s| s.labels.get(&name.id)) {
            let old_init = self.symbol_arena.get(old).is_init;
            if !old_init && is_init {
                self.symbol_arena.get_mut(old).is_init = true;
                return;
            }
            if !(is_init && old_init) {
                return;
            }
            return self.diagnosis.push(DiagnosisNode::new(
                Diagnosis::DuplicateDeclaration(SymbolKind::Label, name),
                *span,
            ));
        };
        let ty = self.types.label();
        let ty = QualifiedType::new(ty, false, false);
        let sym_id = self.symbol_arena.add(name, ty, None, SymbolKind::Label, is_init);
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

        let qualif = self.process_specifiers(ctx, &node.specifiers, span);
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
                                .map(|sym_id| self.symbol_arena.get(sym_id).name.id)
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
        let qualif = self.process_specifiers(ctx, specifiers, span);
        let (ty, decl) = self.make_qualified_type(ctx, qualif, decl)?;
        let storage = constrain::declaration::get_storage(specifiers)
            .collect(self, span)
            .unwrap_or(Storage::Register);
        constrain::external::param_storage_only_register(storage).collect(self, span)?;
        let name = decl.ident(ctx)?;
        Some(self.add_symbol(name, ty, Some(storage), SymbolKind::Parameter, &decl.span, false))
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
        let qualif = self.process_specifiers(ctx, specifiers, span);
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

    // fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
    //     match node.id.resolve(ctx) {
    //         Expression::Identifier(name) => self.check_ident(name),
    //         Expression::PtrAcces(tag_expr, member) | Expression::DotAcces(tag_expr, member) => {
    //             // check tag_expr type is tag
    //             // check if member is in tag_expr type members
    //         }
    //         Expression::Cast(ty, _) | Expression::SizeofType(ty) => {
    //             // check is ty is complete
    //         }
    //         // for every binary check type compat
    //         _ => (),
    //     }
    //     walk_expression(self, ctx, node);
    // }
}

// impl SymbolResolver {
//     fn check_ident(&mut self, name: &Name) {
//         if !self
//             .scopes
//             .iter()
//             .rev()
//             .find_map(|s| s.ordinaries.get(&name.id))
//             .is_some()
//         {
//             self.diagnosis
//                 .push(DiagnosisNode::new(Diagnosis::UndeclaredIdentifier(*name), name.span));
//         }
//     }
// }

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(ctx: Context) -> Context {
        Self::resolve_names(&ctx);
        ctx
    }

    fn resolve_names(ctx: &Context) -> Vec<DeclarationNode> {
        let mut collector = SymbolResolver::default();
        collector.scopes.push(Scope::new(ScopeKind::File));
        walk_translation_unit(&mut collector, ctx, &ctx.ast);
        collector.scopes.pop();
        assert!(collector.scopes.is_empty());
        println!("Symbols:");
        for symbol in &collector.symbol_arena.data {
            println!("{}", symbol.name.id.resolve(ctx));
        }
        println!("Diagnosis:");
        for diag in &collector.diagnosis {
            let _ = diag.print(ctx);
        }
        Vec::new()
    }
}
