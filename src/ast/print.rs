use std::io::{Write, stdout};

use crate::ast::{
    CompoundStatementNode, DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, EnumId, Expression,
    ExpressionNode, ExpressionStatementNode, ExternalDeclaration, ExternalDeclarationNode, FunctionDefinitionNode,
    FunctionParameters, FunctionParametersNode, InitDeclaratorNode, Initializer, InitializerNode, IterationStatement,
    JumpStatement, Labeled, Name, ParameterDeclaration, SelectionStatement, Statement, StatementNode,
    StructDeclaration, StructDeclarator, Type, TypeSpecifier,
};
use crate::context::Context;
use crate::parser::Span;

const MID: &str = "|-";
const LAST: &str = "`-";
const VERT: &str = "| ";
const PAD: &str = "  ";

fn branch(is_last: bool) -> (&'static str, &'static str) {
    if is_last { (LAST, PAD) } else { (MID, VERT) }
}

fn has_type_specs(specs: &[DeclarationSpecifier]) -> bool {
    specs.iter().any(|s| !matches!(s, DeclarationSpecifier::Storage(_)))
}

impl Context {
    pub fn print_ast<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let node = &self.ast;
        writeln!(w, "TranslationUnitDecl {}", node.span)?;
        let prefix = &mut String::new();
        for (i, external) in node.declarations.iter().enumerate() {
            self.print_external(w, prefix, external, Some(i + 1 == node.declarations.len()))?;
        }
        Ok(())
    }

    pub fn print_statement(&self, node: &StatementNode) -> std::io::Result<()> {
        let w = &mut stdout();
        self.print_stmt(w, &mut String::new(), node, None)
    }

    fn print_external<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &ExternalDeclarationNode,
        is_last: Option<bool>,
    ) -> std::io::Result<()> {
        match &node.decl {
            ExternalDeclaration::Declaration(declaration) => self.print_decl_parts(w, prefix, declaration, is_last),
            ExternalDeclaration::Function(function) => self.print_function(w, prefix, function, is_last),
        }
    }

    fn print_function<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &FunctionDefinitionNode,
        is_last: Option<bool>,
    ) -> std::io::Result<()> {
        let (b, extend) = is_last.map_or(("", ""), branch);
        let name = self.declarator_name(&node.declarator);
        write!(w, "{prefix}{b}FunctionDecl {}", node.span)?;
        self.write_name(w, &name)?;
        write!(w, " '")?;
        self.write_decl_type(w, &node.specifiers, &node.declarator)?;
        write!(w, "'")?;
        self.write_storage(w, &node.specifiers)?;
        writeln!(w)?;
        let len = prefix.len();
        prefix.push_str(extend);
        if let Declarator::Function { params, .. } = self.arenas.declarators.get(node.declarator.id) {
            self.print_params_children(w, prefix, params, false)?;
        }
        for declaration in &node.declarations {
            self.print_decl_stmt(w, prefix, declaration, Some(false))?;
        }
        self.print_compound(w, prefix, &node.coumpound, Some(true))?;
        prefix.truncate(len);
        Ok(())
    }

    fn print_decl_parts<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &DeclarationNode,
        is_last: Option<bool>,
    ) -> std::io::Result<()> {
        let child = |i: usize, total: usize| is_last.map(|last| last && i == total);
        let tags = node
            .specifiers
            .iter()
            .filter(|s| {
                matches!(
                    s,
                    DeclarationSpecifier::Type(
                        TypeSpecifier::Struct(_) | TypeSpecifier::Union(_) | TypeSpecifier::Enum(_)
                    )
                )
            })
            .count();
        let total = tags + node.init_declarators.len();
        let mut i = 0;
        for spec in node.specifiers.iter().rev() {
            if let DeclarationSpecifier::Type(ty) = spec {
                match ty {
                    TypeSpecifier::Struct(id) => {
                        i += 1;
                        let record = self.arenas.structs.get(*id);
                        self.print_record(
                            w,
                            prefix,
                            "struct",
                            &record.span,
                            &record.name,
                            &record.fields,
                            child(i, total),
                        )?;
                    }
                    TypeSpecifier::Union(id) => {
                        i += 1;
                        let record = self.arenas.unions.get(*id);
                        self.print_record(
                            w,
                            prefix,
                            "union",
                            &record.span,
                            &record.name,
                            &record.fields,
                            child(i, total),
                        )?;
                    }
                    TypeSpecifier::Enum(id) => {
                        i += 1;
                        self.print_enum(w, prefix, id, child(i, total))?;
                    }
                    _ => {}
                }
            }
        }
        for (j, init_decl) in node.init_declarators.iter().enumerate() {
            i += 1;
            let span = if j == 0 { &node.span } else { &init_decl.span };
            self.print_decl(w, prefix, node, init_decl, span, child(i, total))?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn print_record<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        tag: &str,
        span: &Span,
        name: &Option<Name>,
        fields: &[StructDeclaration],
        is_last: Option<bool>,
    ) -> std::io::Result<()> {
        let (b, extend) = is_last.map_or(("", ""), branch);
        write!(w, "{prefix}{b}RecordDecl {span}")?;
        if let Some(name) = name {
            write!(w, " {}", name.span)?;
        }
        write!(w, " {tag}")?;
        if let Some(name) = name {
            write!(w, " {}", self.arenas.names.get(name.id))?;
        }
        if fields.is_empty() {
            return writeln!(w);
        }
        writeln!(w, " definition")?;
        let len = prefix.len();
        prefix.push_str(extend);
        let total = fields.iter().map(|field| field.struct_declarators.len()).sum::<usize>();
        let mut i = 0;
        for field in fields {
            for (j, declarator) in field.struct_declarators.iter().enumerate() {
                i += 1;
                let span = if j == 0 { &field.span } else { &declarator.span };
                self.print_field(w, prefix, field, declarator, span, i == total)?;
            }
        }
        prefix.truncate(len);
        Ok(())
    }

    fn print_field<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        field: &StructDeclaration,
        declarator: &StructDeclarator,
        span: &Span,
        is_last: bool,
    ) -> std::io::Result<()> {
        let (branch, extend) = branch(is_last);
        write!(w, "{prefix}{branch}FieldDecl {span}")?;
        let name = self.declarator_name(&declarator.declarator);
        self.write_name(w, &name)?;
        write!(w, " '")?;
        self.write_type_specifiers(w, &field.specifiers)?;
        self.write_clang_declarator(w, &declarator.declarator, has_type_specs(&field.specifiers))?;
        writeln!(w, "'")?;
        if let Some(bit_width) = &declarator.bit_width {
            let len = prefix.len();
            prefix.push_str(extend);
            self.print_expression(w, prefix, bit_width, Some(true))?;
            prefix.truncate(len);
        }
        Ok(())
    }

    fn print_enum<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        id: &EnumId,
        is_last: Option<bool>,
    ) -> std::io::Result<()> {
        let (b, extend) = is_last.map_or(("", ""), branch);
        let en = self.arenas.enums.get(*id);
        write!(w, "{prefix}{b}EnumDecl {}", en.span)?;
        if let Some(name) = &en.name {
            write!(w, " {} {}", name.span, self.arenas.names.get(name.id))?;
        }
        writeln!(w)?;
        let len = prefix.len();
        prefix.push_str(extend);
        for (i, variant_id) in en.variants.iter().enumerate() {
            let (branch, extend) = branch(i + 1 == en.variants.len());
            let variant = self.arenas.variants.get(*variant_id);
            writeln!(
                w,
                "{prefix}{branch}EnumConstantDecl {} {} {}",
                variant.span,
                variant.name.span,
                self.arenas.names.get(variant.name.id)
            )?;
            if let Some(value) = &variant.value {
                let len = prefix.len();
                prefix.push_str(extend);
                self.print_expression(w, prefix, value, Some(true))?;
                prefix.truncate(len);
            }
        }
        prefix.truncate(len);
        Ok(())
    }

    fn print_decl<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &DeclarationNode,
        init_decl: &InitDeclaratorNode,
        span: &Span,
        is_last: Option<bool>,
    ) -> std::io::Result<()> {
        let (b, extend) = is_last.map_or(("", ""), branch);
        let is_function = matches!(
            self.arenas.declarators.get(init_decl.declarator.id),
            Declarator::Function { .. }
        );
        let kind = if is_function { "FunctionDecl" } else { "VarDecl" };
        write!(w, "{prefix}{b}{kind} {span}")?;
        let name = self.declarator_name(&init_decl.declarator);
        self.write_name(w, &name)?;
        write!(w, " '")?;
        self.write_decl_type(w, &node.specifiers, &init_decl.declarator)?;
        write!(w, "'")?;
        self.write_storage(w, &node.specifiers)?;
        writeln!(w)?;
        let len = prefix.len();
        prefix.push_str(extend);
        if let Declarator::Function { params, .. } = self.arenas.declarators.get(init_decl.declarator.id) {
            self.print_params_children(w, prefix, params, true)?;
        } else if let Some(init) = &init_decl.initializer {
            self.print_initializer(w, prefix, init, true)?;
        }
        prefix.truncate(len);
        Ok(())
    }

    fn write_decl_type<W: Write>(
        &self,
        w: &mut W,
        specs: &[DeclarationSpecifier],
        declarator: &DeclaratorNode,
    ) -> std::io::Result<()> {
        self.write_type_specifiers(w, specs)?;
        if let Declarator::Function { params, .. } = self.arenas.declarators.get(declarator.id) {
            write!(w, " (")?;
            self.write_clang_params(w, params)?;
            write!(w, ")")
        } else {
            self.write_clang_declarator(w, declarator, has_type_specs(specs))
        }
    }

    fn print_params_children<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        params: &FunctionParametersNode,
        last_is_last: bool,
    ) -> std::io::Result<()> {
        match &params.param {
            FunctionParameters::ParameterTypeList(params) | FunctionParameters::Variadic(params) => {
                for (i, param) in params.iter().enumerate() {
                    self.print_parm(w, prefix, param, i + 1 == params.len() && last_is_last)?;
                }
            }
            FunctionParameters::OldStyle(names) => {
                for (i, name) in names.iter().enumerate() {
                    let (b, _) = branch(i + 1 == names.len() && last_is_last);
                    writeln!(
                        w,
                        "{prefix}{b}ParmVarDecl {} {} {}",
                        name.span,
                        name.span,
                        self.arenas.names.get(name.id)
                    )?;
                }
            }
            FunctionParameters::Empty => {}
        }
        Ok(())
    }

    fn print_stmt<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &StatementNode,
        is_last: Option<bool>,
    ) -> std::io::Result<()> {
        let (b, extend) = is_last.map_or(("", ""), branch);
        match self.arenas.statements.get(node.id) {
            Statement::Labeled(labeled) => match &labeled.inner {
                Labeled::Identifier(name, stmt) => {
                    writeln!(
                        w,
                        "{prefix}{b}LabelStmt {} {} '{}'",
                        labeled.span,
                        name.span,
                        self.arenas.names.get(name.id)
                    )?;
                    let len = prefix.len();
                    prefix.push_str(extend);
                    let result = self.print_stmt(w, prefix, stmt, Some(true));
                    prefix.truncate(len);
                    result
                }
                Labeled::Case(expr, stmt) => {
                    writeln!(w, "{prefix}{b}CaseStmt {}", labeled.span)?;
                    let len = prefix.len();
                    prefix.push_str(extend);
                    let result = self
                        .print_expression(w, prefix, expr, Some(false))
                        .and_then(|()| self.print_stmt(w, prefix, stmt, Some(true)));
                    prefix.truncate(len);
                    result
                }
                Labeled::Default(stmt) => {
                    writeln!(w, "{prefix}{b}DefaultStmt {}", labeled.span)?;
                    let len = prefix.len();
                    prefix.push_str(extend);
                    let result = self.print_stmt(w, prefix, stmt, Some(true));
                    prefix.truncate(len);
                    result
                }
            },
            Statement::Compound(compound) => self.print_compound(w, prefix, compound, is_last),
            Statement::Expression(expr_stmt) => match &expr_stmt.expr {
                Some(expr) => self.print_expression(w, prefix, expr, is_last),
                None => writeln!(w, "{prefix}{b}NullStmt {}", expr_stmt.span),
            },
            Statement::Selection(selection) => match &selection.stmt {
                SelectionStatement::If(cond, then, otherwise) => {
                    write!(w, "{prefix}{b}IfStmt {}", selection.span)?;
                    if otherwise.is_some() {
                        write!(w, " has_else")?;
                    }
                    writeln!(w)?;
                    let len = prefix.len();
                    prefix.push_str(extend);
                    self.print_expression(w, prefix, cond, Some(false))?;
                    self.print_stmt(w, prefix, then, Some(otherwise.is_some()))?;
                    if let Some(otherwise) = otherwise {
                        self.print_stmt(w, prefix, otherwise, Some(true))?;
                    }
                    prefix.truncate(len);
                    Ok(())
                }
                SelectionStatement::Switch(cond, stmt) => {
                    writeln!(w, "{prefix}{b}SwitchStmt {}", selection.span)?;
                    let len = prefix.len();
                    prefix.push_str(extend);
                    let result = self
                        .print_expression(w, prefix, cond, Some(false))
                        .and_then(|()| self.print_stmt(w, prefix, stmt, Some(true)));
                    prefix.truncate(len);
                    result
                }
            },
            Statement::Iteration(iteration) => match &iteration.stmt {
                IterationStatement::While(cond, body) => {
                    writeln!(w, "{prefix}{b}WhileStmt {}", iteration.span)?;
                    let len = prefix.len();
                    prefix.push_str(extend);
                    let result = self
                        .print_expression(w, prefix, cond, Some(false))
                        .and_then(|()| self.print_stmt(w, prefix, body, Some(true)));
                    prefix.truncate(len);
                    result
                }
                IterationStatement::Do(body, cond) => {
                    writeln!(w, "{prefix}{b}DoStmt {}", iteration.span)?;
                    let len = prefix.len();
                    prefix.push_str(extend);
                    let result = self
                        .print_stmt(w, prefix, body, Some(false))
                        .and_then(|()| self.print_expression(w, prefix, cond, Some(true)));
                    prefix.truncate(len);
                    result
                }
                IterationStatement::For(init, cond, inc, body) => {
                    writeln!(w, "{prefix}{b}ForStmt {}", iteration.span)?;
                    let len = prefix.len();
                    prefix.push_str(extend);
                    let total = 3 + inc.is_some() as usize;
                    let mut i = 0;
                    i += 1;
                    self.print_expr_stmt(w, prefix, init, i == total)?;
                    i += 1;
                    self.print_expr_stmt(w, prefix, cond, i == total)?;
                    if let Some(inc) = inc {
                        i += 1;
                        self.print_expression(w, prefix, inc, Some(i == total))?;
                    }
                    i += 1;
                    self.print_stmt(w, prefix, body, Some(i == total))?;
                    prefix.truncate(len);
                    Ok(())
                }
            },
            Statement::Jump(jump) => match &jump.stmt {
                JumpStatement::Goto => writeln!(w, "{prefix}{b}GotoStmt {}", jump.span),
                JumpStatement::Continue => writeln!(w, "{prefix}{b}ContinueStmt {}", jump.span),
                JumpStatement::Break => writeln!(w, "{prefix}{b}BreakStmt {}", jump.span),
                JumpStatement::Return(expr) => {
                    writeln!(w, "{prefix}{b}ReturnStmt {}", jump.span)?;
                    if let Some(expr) = expr {
                        let len = prefix.len();
                        prefix.push_str(extend);
                        self.print_expression(w, prefix, expr, Some(true))?;
                        prefix.truncate(len);
                    }
                    Ok(())
                }
            },
        }
    }

    fn print_compound<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        compound: &CompoundStatementNode,
        is_last: Option<bool>,
    ) -> std::io::Result<()> {
        let (b, extend) = is_last.map_or(("", ""), branch);
        writeln!(w, "{prefix}{b}CompoundStmt {}", compound.span)?;
        let len = prefix.len();
        prefix.push_str(extend);
        let total = compound.declarations.len() + compound.statements.len();
        let mut i = 0;
        for declaration in &compound.declarations {
            i += 1;
            self.print_decl_stmt(w, prefix, declaration, Some(i == total))?;
        }
        for stmt in &compound.statements {
            i += 1;
            self.print_stmt(w, prefix, stmt, Some(i == total))?;
        }
        prefix.truncate(len);
        Ok(())
    }

    fn print_expr_stmt<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &ExpressionStatementNode,
        is_last: bool,
    ) -> std::io::Result<()> {
        match &node.expr {
            Some(expr) => self.print_expression(w, prefix, expr, Some(is_last)),
            None => {
                let (b, _) = branch(is_last);
                writeln!(w, "{prefix}{b}NullStmt {}", node.span)
            }
        }
    }

    fn print_decl_stmt<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &DeclarationNode,
        is_last: Option<bool>,
    ) -> std::io::Result<()> {
        let (b, extend) = is_last.map_or(("", ""), branch);
        writeln!(w, "{prefix}{b}DeclStmt {}", node.span)?;
        let len = prefix.len();
        prefix.push_str(extend);
        let result = self.print_decl_parts(w, prefix, node, Some(true));
        prefix.truncate(len);
        result
    }

    fn write_name<W: Write>(&self, w: &mut W, name: &Option<Name>) -> std::io::Result<()> {
        if let Some(name) = name {
            write!(w, " {} {}", name.span, self.arenas.names.get(name.id))?;
        }
        Ok(())
    }

    fn print_parm<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        param: &ParameterDeclaration,
        is_last: bool,
    ) -> std::io::Result<()> {
        let (b, _) = branch(is_last);
        write!(w, "{prefix}{b}ParmVarDecl {}", param.span)?;
        let name = self.declarator_name(&param.declarator);
        self.write_name(w, &name)?;
        write!(w, " '")?;
        self.write_type_specifiers(w, &param.specifiers)?;
        self.write_clang_declarator(w, &param.declarator, has_type_specs(&param.specifiers))?;
        writeln!(w, "'")
    }

    fn declarator_name(&self, node: &DeclaratorNode) -> Option<Name> {
        match self.arenas.declarators.get(node.id) {
            Declarator::Ident(name) => Some(name.clone()),
            Declarator::Pointer { inner: Some(inner), .. } => self.declarator_name(inner),
            Declarator::Array { declarator, .. } | Declarator::Function { declarator, .. } => {
                self.declarator_name(declarator)
            }
            _ => None,
        }
    }

    fn write_type_specifiers<W: Write>(&self, w: &mut W, specs: &[DeclarationSpecifier]) -> std::io::Result<()> {
        let mut first = true;
        for spec in specs.iter().rev() {
            if matches!(spec, DeclarationSpecifier::Storage(_)) {
                continue;
            }
            if !first {
                write!(w, " ")?;
            }
            first = false;
            self.write_specifier_atom(w, spec)?;
        }
        Ok(())
    }

    fn write_storage<W: Write>(&self, w: &mut W, specs: &[DeclarationSpecifier]) -> std::io::Result<()> {
        for spec in specs.iter().rev() {
            if let DeclarationSpecifier::Storage(storage) = spec {
                write!(w, " {storage}")?;
            }
        }
        Ok(())
    }

    fn write_specifier_atom<W: Write>(&self, w: &mut W, spec: &DeclarationSpecifier) -> std::io::Result<()> {
        match spec {
            DeclarationSpecifier::Type(ty) => self.write_type_specifier(w, ty),
            DeclarationSpecifier::Qualifier(qualifier) => write!(w, "{qualifier}"),
            DeclarationSpecifier::Storage(storage) => write!(w, "{storage}"),
        }
    }

    fn write_type_specifier<W: Write>(&self, w: &mut W, spec: &TypeSpecifier) -> std::io::Result<()> {
        match spec {
            TypeSpecifier::TypedefName(name) => write!(w, "{spec} {}", self.arenas.names.get(name.id)),
            TypeSpecifier::Struct(id) => self.write_tag(w, "struct", &self.arenas.structs.get(*id).name),
            TypeSpecifier::Union(id) => self.write_tag(w, "union", &self.arenas.unions.get(*id).name),
            TypeSpecifier::Enum(id) => self.write_tag(w, "enum", &self.arenas.enums.get(*id).name),
            _ => write!(w, "{spec}"),
        }
    }

    fn write_tag<W: Write>(&self, w: &mut W, tag: &str, name: &Option<Name>) -> std::io::Result<()> {
        write!(w, "{tag}")?;
        if let Some(name) = name {
            write!(w, " {}", self.arenas.names.get(name.id))?;
        }
        Ok(())
    }

    fn write_clang_declarator<W: Write>(
        &self,
        w: &mut W,
        node: &DeclaratorNode,
        star_space: bool,
    ) -> std::io::Result<()> {
        match self.arenas.declarators.get(node.id) {
            Declarator::Ident(_) | Declarator::Abstract => Ok(()),
            Declarator::Pointer { qualifiers, inner } => {
                if star_space {
                    write!(w, " ")?;
                }
                write!(w, "*")?;
                for (i, qualifier) in qualifiers.iter().enumerate() {
                    if i > 0 {
                        write!(w, " ")?;
                    }
                    write!(w, "{qualifier}")?;
                }
                match inner {
                    Some(inner) => self.write_clang_declarator(w, inner, false),
                    None => Ok(()),
                }
            }
            Declarator::Array { declarator, size } => {
                self.write_clang_declarator(w, declarator, star_space)?;
                write!(w, "[")?;
                if let Some(size) = size {
                    self.write_expression(w, size)?;
                }
                write!(w, "]")
            }
            Declarator::Function { declarator, params } => {
                self.write_clang_declarator(w, declarator, star_space)?;
                write!(w, " (")?;
                self.write_clang_params(w, params)?;
                write!(w, ")")
            }
        }
    }

    fn write_clang_params<W: Write>(&self, w: &mut W, node: &FunctionParametersNode) -> std::io::Result<()> {
        match &node.param {
            FunctionParameters::Empty | FunctionParameters::OldStyle(_) => Ok(()),
            FunctionParameters::ParameterTypeList(params) => self.write_param_types(w, params, false),
            FunctionParameters::Variadic(params) => self.write_param_types(w, params, true),
        }
    }

    fn write_param_types<W: Write>(
        &self,
        w: &mut W,
        params: &[ParameterDeclaration],
        variadic: bool,
    ) -> std::io::Result<()> {
        let mut first = true;
        for param in params {
            if !first {
                write!(w, ", ")?;
            }
            first = false;
            self.write_type_specifiers(w, &param.specifiers)?;
            self.write_clang_declarator(w, &param.declarator, has_type_specs(&param.specifiers))?;
        }
        if variadic {
            if !first {
                write!(w, ", ")?;
            }
            write!(w, "...")?;
        }
        Ok(())
    }

    fn print_initializer<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &InitializerNode,
        is_last: bool,
    ) -> std::io::Result<()> {
        match &node.init {
            Initializer::Single(expr) => self.print_expression(w, prefix, expr, Some(is_last)),
            Initializer::List(list) => {
                let (branch, extend) = branch(is_last);
                writeln!(w, "{prefix}{branch}InitListExpr {}", node.span)?;
                let len = prefix.len();
                prefix.push_str(extend);
                for (i, item) in list.iter().enumerate() {
                    self.print_initializer(w, prefix, item, i + 1 == list.len())?;
                }
                prefix.truncate(len);
                Ok(())
            }
        }
    }

    fn print_expression<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &ExpressionNode,
        is_last: Option<bool>,
    ) -> std::io::Result<()> {
        let expr = self.arenas.expressions.get(node.id);
        let (b, extend) = is_last.map_or(("", ""), branch);
        write!(w, "{prefix}{b}{expr} {}", node.span)?;
        self.write_inline_name(w, expr)?;
        writeln!(w)?;
        let len = prefix.len();
        prefix.push_str(extend);
        let result = self.print_expression_children(w, prefix, node);
        prefix.truncate(len);
        result
    }

    fn write_inline_name<W: Write>(&self, w: &mut W, expr: &Expression) -> std::io::Result<()> {
        match expr {
            Expression::Constant(name)
            | Expression::Identifier(name)
            | Expression::StringLiteral(name)
            | Expression::DotAcces(_, name)
            | Expression::PtrAcces(_, name) => write!(w, " {}", self.arenas.names.get(name.id)),
            _ => Ok(()),
        }
    }

    fn print_expression_children<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &ExpressionNode,
    ) -> std::io::Result<()> {
        let expr = self.arenas.expressions.get(node.id);
        match expr {
            Expression::Constant(_) | Expression::Identifier(_) | Expression::StringLiteral(_) => {}
            Expression::PostInc(id)
            | Expression::PostDec(id)
            | Expression::PreInc(id)
            | Expression::PreDec(id)
            | Expression::Addr(id)
            | Expression::Deref(id)
            | Expression::Plus(id)
            | Expression::Minus(id)
            | Expression::BitNot(id)
            | Expression::Not(id) => self.print_expression(w, prefix, id, Some(true))?,
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
            | Expression::MulAssign(lhs, rhs)
            | Expression::DivAssign(lhs, rhs)
            | Expression::ModAssign(lhs, rhs)
            | Expression::AddAssign(lhs, rhs)
            | Expression::SubAssign(lhs, rhs)
            | Expression::LeftAssign(lhs, rhs)
            | Expression::RightAssign(lhs, rhs)
            | Expression::AndAssign(lhs, rhs)
            | Expression::XorAssign(lhs, rhs)
            | Expression::OrAssign(lhs, rhs)
            | Expression::List(lhs, rhs)
            | Expression::ArrayAcces(lhs, rhs) => self.print_binop(w, prefix, lhs, rhs)?,
            Expression::FunctionCall(fun, Some(args)) => self.print_binop(w, prefix, fun, args)?,
            Expression::FunctionCall(fun, None) => self.print_expression(w, prefix, fun, Some(true))?,
            Expression::DotAcces(tag, _) => self.print_expression(w, prefix, tag, Some(true))?,
            Expression::PtrAcces(tag, _) => self.print_expression(w, prefix, tag, Some(true))?,
            Expression::SizeofExpr(node) => self.print_expression(w, prefix, node, Some(true))?,
            Expression::SizeofType(ty) => self.print_type(w, prefix, ty, true)?,
            Expression::ConstantExpression(node) => self.print_expression(w, prefix, node, Some(true))?,
            Expression::Ternary(cond, then, otherwise) => {
                self.print_expression(w, prefix, cond, Some(false))?;
                self.print_expression(w, prefix, then, Some(false))?;
                self.print_expression(w, prefix, otherwise, Some(true))?;
            }
            Expression::Cast(ty, node) => {
                self.print_type(w, prefix, ty, false)?;
                self.print_expression(w, prefix, node, Some(true))?;
            }
        }
        Ok(())
    }

    fn print_binop<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> std::io::Result<()> {
        self.print_expression(w, prefix, lhs, Some(false))?;
        self.print_expression(w, prefix, rhs, Some(true))
    }

    fn print_type<W: Write>(&self, w: &mut W, prefix: &mut String, ty: &Type, is_last: bool) -> std::io::Result<()> {
        let (branch, _) = branch(is_last);
        write!(w, "{prefix}{branch}Type ")?;
        self.write_type(w, ty)?;
        writeln!(w)
    }

    fn write_type<W: Write>(&self, w: &mut W, ty: &Type) -> std::io::Result<()> {
        let mut first = true;
        for spec in ty.specifiers.iter().rev() {
            if !first {
                write!(w, " ")?;
            }
            first = false;
            self.write_specifier_atom(w, spec)?;
        }
        self.write_clang_declarator(w, &ty.declarator, !ty.specifiers.is_empty())
    }

    fn write_expression<W: Write>(&self, w: &mut W, node: &ExpressionNode) -> std::io::Result<()> {
        let expr = self.arenas.expressions.get(node.id);
        if let Some((lhs, op, rhs)) = binary_parts(expr) {
            self.write_expression(w, lhs)?;
            write!(w, " {op} ")?;
            return self.write_expression(w, rhs);
        }
        match expr {
            Expression::Constant(name) | Expression::Identifier(name) | Expression::StringLiteral(name) => {
                write!(w, "{}", self.arenas.names.get(name.id))
            }
            Expression::ConstantExpression(inner) => self.write_expression(w, inner),
            Expression::PostInc(e) => {
                self.write_expression(w, e)?;
                write!(w, "++")
            }
            Expression::PostDec(e) => {
                self.write_expression(w, e)?;
                write!(w, "--")
            }
            Expression::PreInc(e) => {
                write!(w, "++")?;
                self.write_expression(w, e)
            }
            Expression::PreDec(e) => {
                write!(w, "--")?;
                self.write_expression(w, e)
            }
            Expression::Addr(e) => {
                write!(w, "&")?;
                self.write_expression(w, e)
            }
            Expression::Deref(e) => {
                write!(w, "*")?;
                self.write_expression(w, e)
            }
            Expression::Plus(e) => {
                write!(w, "+")?;
                self.write_expression(w, e)
            }
            Expression::Minus(e) => {
                write!(w, "-")?;
                self.write_expression(w, e)
            }
            Expression::BitNot(e) => {
                write!(w, "~")?;
                self.write_expression(w, e)
            }
            Expression::Not(e) => {
                write!(w, "!")?;
                self.write_expression(w, e)
            }
            Expression::Ternary(cond, then, otherwise) => {
                self.write_expression(w, cond)?;
                write!(w, " ? ")?;
                self.write_expression(w, then)?;
                write!(w, " : ")?;
                self.write_expression(w, otherwise)
            }
            Expression::ArrayAcces(array, index) => {
                self.write_expression(w, array)?;
                write!(w, "[")?;
                self.write_expression(w, index)?;
                write!(w, "]")
            }
            Expression::FunctionCall(fun, args) => {
                self.write_expression(w, fun)?;
                write!(w, "(")?;
                if let Some(args) = args {
                    self.write_expression(w, args)?;
                }
                write!(w, ")")
            }
            Expression::DotAcces(tag, name) => {
                self.write_expression(w, tag)?;
                write!(w, ".{}", self.arenas.names.get(name.id))
            }
            Expression::PtrAcces(tag, name) => {
                self.write_expression(w, tag)?;
                write!(w, "->{}", self.arenas.names.get(name.id))
            }
            Expression::SizeofExpr(e) => {
                write!(w, "sizeof(")?;
                self.write_expression(w, e)?;
                write!(w, ")")
            }
            Expression::SizeofType(ty) => {
                write!(w, "sizeof(")?;
                self.write_type(w, ty)?;
                write!(w, ")")
            }
            Expression::Cast(ty, e) => {
                write!(w, "(")?;
                self.write_type(w, ty)?;
                write!(w, ")")?;
                self.write_expression(w, e)
            }
            _ => unreachable!("binary expression handled by binary_parts"),
        }
    }
}

fn binary_parts(expr: &Expression) -> Option<(&ExpressionNode, &'static str, &ExpressionNode)> {
    use Expression::*;
    let (lhs, op, rhs) = match expr {
        Add(l, r) => (l, "+", r),
        Sub(l, r) => (l, "-", r),
        Mul(l, r) => (l, "*", r),
        Div(l, r) => (l, "/", r),
        Mod(l, r) => (l, "%", r),
        Right(l, r) => (l, ">>", r),
        Left(l, r) => (l, "<<", r),
        Greater(l, r) => (l, ">", r),
        Lower(l, r) => (l, "<", r),
        GreaterEq(l, r) => (l, ">=", r),
        LowerEq(l, r) => (l, "<=", r),
        Eq(l, r) => (l, "==", r),
        Neq(l, r) => (l, "!=", r),
        BitAnd(l, r) => (l, "&", r),
        BitOr(l, r) => (l, "|", r),
        BitXor(l, r) => (l, "^", r),
        And(l, r) => (l, "&&", r),
        Or(l, r) => (l, "||", r),
        Assign(l, r) => (l, "=", r),
        MulAssign(l, r) => (l, "*=", r),
        DivAssign(l, r) => (l, "/=", r),
        ModAssign(l, r) => (l, "%=", r),
        AddAssign(l, r) => (l, "+=", r),
        SubAssign(l, r) => (l, "-=", r),
        LeftAssign(l, r) => (l, "<<=", r),
        RightAssign(l, r) => (l, ">>=", r),
        AndAssign(l, r) => (l, "&=", r),
        XorAssign(l, r) => (l, "^=", r),
        OrAssign(l, r) => (l, "|=", r),
        List(l, r) => (l, ",", r),
        _ => return None,
    };
    Some((lhs, op, rhs))
}
