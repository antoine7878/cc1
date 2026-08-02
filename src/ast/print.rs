use std::io::{Write, stdout};

use crate::ast::{
    DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, Expression, ExpressionNode, FunctionParameters,
    FunctionParametersNode, InitDeclaratorNode, Initializer, InitializerNode, Name, ParameterDeclaration, Type,
    TypeSpecifier,
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

impl Context {
    pub fn print_ast(&mut self, node: &DeclarationNode) -> std::io::Result<()> {
        let w = &mut stdout();
        for (i, init_decl) in node.init_declarators.iter().enumerate() {
            let span = if i == 0 { &node.span } else { &init_decl.span };
            self.print_decl(w, node, init_decl, span)?;
        }
        Ok(())
    }

    fn print_decl<W: Write>(
        &self,
        w: &mut W,
        node: &DeclarationNode,
        init_decl: &InitDeclaratorNode,
        span: &Span,
    ) -> std::io::Result<()> {
        let (ty_specs, storage) = self.split_specifiers(&node.specifiers);
        let name = self.declarator_name(&init_decl.declarator);
        let name_loc = name.as_ref().map(|n| format!(" {}", n.span)).unwrap_or_default();
        let name_str = name
            .map(|n| format!(" {}", self.arenas.names.get(n.id)))
            .unwrap_or_default();
        let storage = if storage.is_empty() {
            storage
        } else {
            format!(" {storage}")
        };
        if let Declarator::Function { params, .. } = self.arenas.declarators.get(init_decl.declarator.id) {
            let ty = format!("{} ({})", ty_specs, self.clang_params(params));
            writeln!(w, "FunctionDecl {span}{name_loc}{name_str} '{ty}'{storage}")?;
            match &params.param {
                FunctionParameters::ParameterTypeList(params) | FunctionParameters::Variadic(params) => {
                    for (i, param) in params.iter().enumerate() {
                        self.print_parm(w, "", param, i + 1 == params.len())?;
                    }
                }
                FunctionParameters::OldStyle(names) => {
                    for (i, name) in names.iter().enumerate() {
                        let (b, _) = branch(i + 1 == names.len());
                        writeln!(
                            w,
                            "{b}ParmVarDecl {} {} {}",
                            name.span,
                            name.span,
                            self.arenas.names.get(name.id)
                        )?;
                    }
                }
                FunctionParameters::Empty => {}
            }
        } else {
            let ty = self.clang_type(&ty_specs, &init_decl.declarator);
            writeln!(w, "VarDecl {span}{name_loc}{name_str} '{ty}'{storage}")?;
            if let Some(init) = &init_decl.initializer {
                self.print_initializer(w, "", init, true)?;
            }
        }
        Ok(())
    }

    fn print_parm<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        param: &ParameterDeclaration,
        is_last: bool,
    ) -> std::io::Result<()> {
        let (b, _) = branch(is_last);
        let (ty_specs, _) = self.split_specifiers(&param.specifiers);
        let ty = self.clang_type(&ty_specs, &param.declarator);
        let name = self.declarator_name(&param.declarator);
        let name_loc = name.as_ref().map(|n| format!(" {}", n.span)).unwrap_or_default();
        let name_str = name
            .map(|n| format!(" {}", self.arenas.names.get(n.id)))
            .unwrap_or_default();
        writeln!(w, "{prefix}{b}ParmVarDecl {}{name_loc}{name_str} '{ty}'", param.span)
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

    fn split_specifiers(&self, specs: &[DeclarationSpecifier]) -> (String, String) {
        let mut ty = Vec::new();
        let mut storage = Vec::new();
        for spec in specs.iter().rev() {
            match spec {
                DeclarationSpecifier::Storage(s) => storage.push(format!("{s}")),
                _ => ty.push(self.format_specifier_atom(spec)),
            }
        }
        (ty.join(" "), storage.join(" "))
    }

    fn format_specifier_atom(&self, spec: &DeclarationSpecifier) -> String {
        match spec {
            DeclarationSpecifier::Type(ty) => self.format_type_specifier(ty),
            DeclarationSpecifier::Qualifier(qualifier) => format!("{qualifier}"),
            DeclarationSpecifier::Storage(storage) => format!("{storage}"),
        }
    }

    fn format_type_specifier(&self, spec: &TypeSpecifier) -> String {
        match spec {
            TypeSpecifier::TypedefName(name) => format!("{spec} {}", self.arenas.names.get(name.id)),
            TypeSpecifier::Struct(id) => format!("{spec} {id:?}"),
            TypeSpecifier::Union(id) => format!("{spec} {id:?}"),
            TypeSpecifier::Enum(id) => format!("{spec} {id:?}"),
            _ => format!("{spec}"),
        }
    }

    fn clang_type(&self, base: &str, node: &DeclaratorNode) -> String {
        match self.arenas.declarators.get(node.id) {
            Declarator::Ident(_) | Declarator::Abstract => base.to_string(),
            Declarator::Pointer { qualifiers, inner } => {
                let mut s = base.to_string();
                if !s.is_empty() && !s.ends_with('*') {
                    s.push(' ');
                }
                s.push('*');
                s.push_str(&qualifiers.iter().map(|q| format!("{q}")).collect::<Vec<_>>().join(" "));
                match inner {
                    Some(inner) => self.clang_type(&s, inner),
                    None => s,
                }
            }
            Declarator::Array { declarator, size } => {
                let size = size.as_ref().map(|s| self.format_expression(s)).unwrap_or_default();
                format!("{}[{size}]", self.clang_type(base, declarator))
            }
            Declarator::Function { declarator, params } => {
                format!("{} ({})", self.clang_type(base, declarator), self.clang_params(params))
            }
        }
    }

    fn clang_params(&self, node: &FunctionParametersNode) -> String {
        match &node.param {
            FunctionParameters::Empty | FunctionParameters::OldStyle(_) => String::new(),
            FunctionParameters::ParameterTypeList(params) => params
                .iter()
                .map(|p| {
                    let (ty_specs, _) = self.split_specifiers(&p.specifiers);
                    self.clang_type(&ty_specs, &p.declarator)
                })
                .collect::<Vec<_>>()
                .join(", "),
            FunctionParameters::Variadic(params) => {
                let mut s = params
                    .iter()
                    .map(|p| {
                        let (ty_specs, _) = self.split_specifiers(&p.specifiers);
                        self.clang_type(&ty_specs, &p.declarator)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                if !s.is_empty() {
                    s.push_str(", ");
                }
                s.push_str("...");
                s
            }
        }
    }

    fn print_initializer<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        node: &InitializerNode,
        is_last: bool,
    ) -> std::io::Result<()> {
        match &node.init {
            Initializer::Single(expr) => self.print_expression(w, prefix, expr, is_last),
            Initializer::List(list) => {
                let (branch, extend) = branch(is_last);
                writeln!(w, "{prefix}{branch}InitListExpr {}", node.span)?;
                let prefix = &format!("{prefix}{extend}");
                for (i, item) in list.iter().enumerate() {
                    self.print_initializer(w, prefix, item, i + 1 == list.len())?;
                }
                Ok(())
            }
        }
    }

    fn print_expression<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        node: &ExpressionNode,
        is_last: bool,
    ) -> std::io::Result<()> {
        let expr = self.arenas.expressions.get(node.id);
        let (branch, extend) = branch(is_last);
        writeln!(w, "{prefix}{branch}{expr} {}{}", node.span, self.inline_name(expr))?;
        self.print_expression_children(w, &format!("{prefix}{extend}"), node)
    }

    fn inline_name(&self, expr: &Expression) -> String {
        match expr {
            Expression::Constant(name)
            | Expression::Identifier(name)
            | Expression::StringLiteral(name)
            | Expression::DotAcces(_, name)
            | Expression::PtrAcces(_, name) => format!(" {}", self.arenas.names.get(name.id)),
            _ => String::new(),
        }
    }

    fn print_expression_children<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
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
            | Expression::Not(id) => self.print_expression(w, prefix, id, true)?,
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
            Expression::FunctionCall(fun, None) => self.print_expression(w, prefix, fun, true)?,
            Expression::DotAcces(tag, _) => self.print_expression(w, prefix, tag, true)?,
            Expression::PtrAcces(tag, _) => self.print_expression(w, prefix, tag, true)?,
            Expression::SizeofExpr(node) => self.print_expression(w, prefix, node, true)?,
            Expression::SizeofType(ty) => self.print_type(w, prefix, ty, true)?,
            Expression::ConstantExpression(node) => self.print_expression(w, prefix, node, true)?,
            Expression::Ternary(cond, then, otherwise) => {
                self.print_expression(w, prefix, cond, false)?;
                self.print_expression(w, prefix, then, false)?;
                self.print_expression(w, prefix, otherwise, true)?;
            }
            Expression::Cast(ty, node) => {
                self.print_type(w, prefix, ty, false)?;
                self.print_expression(w, prefix, node, true)?;
            }
        }
        Ok(())
    }

    fn print_binop<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> std::io::Result<()> {
        self.print_expression(w, prefix, lhs, false)?;
        self.print_expression(w, prefix, rhs, true)
    }

    fn print_type<W: Write>(&self, w: &mut W, prefix: &str, ty: &Type, is_last: bool) -> std::io::Result<()> {
        let (branch, _) = branch(is_last);
        writeln!(w, "{prefix}{branch}Type {}", self.format_type(ty))
    }

    fn format_type(&self, ty: &Type) -> String {
        let specs = ty
            .specifiers
            .iter()
            .rev()
            .map(|spec| self.format_type_specifier(spec))
            .collect::<Vec<_>>()
            .join(" ");
        let declarator = self.clang_type("", &ty.declarator);
        if declarator.is_empty() {
            specs
        } else {
            format!("{specs} {declarator}")
        }
    }

    fn format_expression(&self, node: &ExpressionNode) -> String {
        let expr = self.arenas.expressions.get(node.id);
        if let Some((lhs, op, rhs)) = binary_parts(expr) {
            return format!("{} {op} {}", self.format_expression(lhs), self.format_expression(rhs));
        }
        match expr {
            Expression::Constant(name) | Expression::Identifier(name) | Expression::StringLiteral(name) => {
                self.arenas.names.get(name.id).to_string()
            }
            Expression::ConstantExpression(inner) => self.format_expression(inner),
            Expression::PostInc(e) => format!("{}++", self.format_expression(e)),
            Expression::PostDec(e) => format!("{}--", self.format_expression(e)),
            Expression::PreInc(e) => format!("++{}", self.format_expression(e)),
            Expression::PreDec(e) => format!("--{}", self.format_expression(e)),
            Expression::Addr(e) => format!("&{}", self.format_expression(e)),
            Expression::Deref(e) => format!("*{}", self.format_expression(e)),
            Expression::Plus(e) => format!("+{}", self.format_expression(e)),
            Expression::Minus(e) => format!("-{}", self.format_expression(e)),
            Expression::BitNot(e) => format!("~{}", self.format_expression(e)),
            Expression::Not(e) => format!("!{}", self.format_expression(e)),
            Expression::Ternary(cond, then, otherwise) => format!(
                "{} ? {} : {}",
                self.format_expression(cond),
                self.format_expression(then),
                self.format_expression(otherwise)
            ),
            Expression::ArrayAcces(array, index) => {
                format!("{}[{}]", self.format_expression(array), self.format_expression(index))
            }
            Expression::FunctionCall(fun, args) => {
                let args = args.as_ref().map(|a| self.format_expression(a)).unwrap_or_default();
                format!("{}({args})", self.format_expression(fun))
            }
            Expression::DotAcces(tag, name) => {
                format!("{}.{}", self.format_expression(tag), self.arenas.names.get(name.id))
            }
            Expression::PtrAcces(tag, name) => {
                format!("{}->{}", self.format_expression(tag), self.arenas.names.get(name.id))
            }
            Expression::SizeofExpr(e) => format!("sizeof({})", self.format_expression(e)),
            Expression::SizeofType(ty) => format!("sizeof({})", self.format_type(ty)),
            Expression::Cast(ty, e) => format!("({}){}", self.format_type(ty), self.format_expression(e)),
            _ => format!("{expr}"),
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
