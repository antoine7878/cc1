use std::io::{Write, stdout};

use crate::ast::{
    DeclarationNode, DeclarationSpecifier, Declarator, DeclaratorNode, Expression, ExpressionNode, FunctionParameters,
    FunctionParametersNode, InitDeclaratorNode, Initializer, InitializerNode, ParameterDeclaration, Type, TypeSpecifier,
};
use crate::context::Context;

const MID: &str = "├── ";
const LAST: &str = "└── ";
const VERT: &str = "│   ";
const PAD: &str = "    ";

fn branch(is_last: bool) -> (&'static str, &'static str) {
    if is_last { (LAST, PAD) } else { (MID, VERT) }
}

impl Context {
    pub fn print_ast(&mut self, node: &DeclarationNode) -> std::io::Result<()> {
        let w = &mut stdout();
        writeln!(w, "Declaration {}", node.span)?;
        let total = node.specifiers.len() + node.init_declarators.len();
        let mut i = 0;
        for spec in &node.specifiers {
            i += 1;
            self.print_specifier(w, "", spec, i == total)?;
        }
        for init_decl in &node.init_declarators {
            i += 1;
            self.print_init_declarator(w, "", init_decl, i == total)?;
        }
        Ok(())
    }

    fn print_specifier<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        spec: &DeclarationSpecifier,
        is_last: bool,
    ) -> std::io::Result<()> {
        let (branch, _) = branch(is_last);
        writeln!(w, "{prefix}{branch}{}", self.format_specifier(spec))
    }

    fn format_specifier(&self, spec: &DeclarationSpecifier) -> String {
        match spec {
            DeclarationSpecifier::Type(ty) => format!("Type {}", self.format_type_specifier(ty)),
            DeclarationSpecifier::Qualifier(qualifier) => format!("Qualifier {qualifier}"),
            DeclarationSpecifier::Storage(storage) => format!("Storage {storage}"),
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

    fn print_init_declarator<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        node: &InitDeclaratorNode,
        is_last: bool,
    ) -> std::io::Result<()> {
        let (branch, extend) = branch(is_last);
        writeln!(w, "{prefix}{branch}InitDeclarator {}", node.span)?;
        let prefix = &format!("{prefix}{extend}");
        match &node.initializer {
            Some(init) => {
                self.print_declarator(w, prefix, &node.declarator, false)?;
                self.print_initializer(w, prefix, init, true)
            }
            None => self.print_declarator(w, prefix, &node.declarator, true),
        }
    }

    fn print_declarator<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        node: &DeclaratorNode,
        is_last: bool,
    ) -> std::io::Result<()> {
        let declarator = self.arenas.declarators.get(node.id);
        let (branch, extend) = branch(is_last);
        writeln!(w, "{prefix}{branch}{declarator} {}{}", node.span, self.inline_declarator(declarator))?;
        self.print_declarator_children(w, &format!("{prefix}{extend}"), node)
    }

    fn inline_declarator(&self, declarator: &Declarator) -> String {
        match declarator {
            Declarator::Ident(name) => format!(" {}", self.arenas.names.get(name.id)),
            Declarator::Pointer { qualifiers, .. } => qualifiers.iter().map(|q| format!(" {q}")).collect(),
            _ => String::new(),
        }
    }

    fn print_declarator_children<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        node: &DeclaratorNode,
    ) -> std::io::Result<()> {
        let declarator = self.arenas.declarators.get(node.id);
        match declarator {
            Declarator::Ident(_) | Declarator::Abstract => {}
            Declarator::Pointer { inner, .. } => {
                if let Some(inner) = inner {
                    self.print_declarator(w, prefix, inner, true)?;
                }
            }
            Declarator::Array { declarator, size } => match size {
                Some(size) => {
                    self.print_declarator(w, prefix, declarator, false)?;
                    self.print_expression(w, prefix, size, true)?;
                }
                None => self.print_declarator(w, prefix, declarator, true)?,
            },
            Declarator::Function { declarator, params } => {
                self.print_declarator(w, prefix, declarator, false)?;
                self.print_parameters(w, prefix, params, true)?;
            }
        }
        Ok(())
    }

    fn print_initializer<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        node: &InitializerNode,
        is_last: bool,
    ) -> std::io::Result<()> {
        let (branch, extend) = branch(is_last);
        writeln!(w, "{prefix}{branch}{} {}", node.init, node.span)?;
        let prefix = &format!("{prefix}{extend}");
        match &node.init {
            Initializer::Single(expr) => self.print_expression(w, prefix, expr, true)?,
            Initializer::List(list) => {
                for (i, item) in list.iter().enumerate() {
                    self.print_initializer(w, prefix, item, i + 1 == list.len())?;
                }
            }
        }
        Ok(())
    }

    fn print_parameters<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        node: &FunctionParametersNode,
        is_last: bool,
    ) -> std::io::Result<()> {
        let (branch, extend) = branch(is_last);
        writeln!(w, "{prefix}{branch}{} {}", node.param, node.span)?;
        let prefix = &format!("{prefix}{extend}");
        match &node.param {
            FunctionParameters::Empty => {}
            FunctionParameters::OldStyle(names) => {
                for (i, name) in names.iter().enumerate() {
                    let b = if i + 1 == names.len() { LAST } else { MID };
                    writeln!(w, "{prefix}{b}{} {}", name.span, self.arenas.names.get(name.id))?;
                }
            }
            FunctionParameters::ParameterTypeList(params) | FunctionParameters::Variadic(params) => {
                for (i, param) in params.iter().enumerate() {
                    self.print_parameter(w, prefix, param, i + 1 == params.len())?;
                }
            }
        }
        Ok(())
    }

    fn print_parameter<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        node: &ParameterDeclaration,
        is_last: bool,
    ) -> std::io::Result<()> {
        let (branch, extend) = branch(is_last);
        writeln!(w, "{prefix}{branch}Parameter {}", node.span)?;
        let prefix = &format!("{prefix}{extend}");
        for spec in &node.specifiers {
            self.print_specifier(w, prefix, spec, false)?;
        }
        self.print_declarator(w, prefix, &node.declarator, true)
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
        let (branch, extend) = branch(is_last);
        writeln!(w, "{prefix}{branch}Type")?;
        let prefix = &format!("{prefix}{extend}");
        for spec in &ty.specifiers {
            writeln!(w, "{prefix}{MID}{}", self.format_type_specifier(spec))?;
        }
        self.print_declarator(w, prefix, &ty.declarator, true)
    }
}
