use std::io::{Write, stdout};

use crate::ast::expression::ExpressionNode;
use crate::ast::{Expression, Type};
use crate::context::Context;

const MID: &str = "├── ";
const LAST: &str = "└── ";
const VERT: &str = "│   ";
const PAD: &str = "    ";

impl Context {
    pub fn print_ast(&mut self, node: &ExpressionNode) -> std::io::Result<()> {
        let w = &mut stdout();
        let expr = self.arenas.expressions.get(node.id);
        writeln!(w, "{expr} {}{}", node.span, self.inline_name(expr))?;
        self.print_children(w, "", node)
    }

    fn print_expression<W: Write>(
        &self,
        w: &mut W,
        prefix: &str,
        node: &ExpressionNode,
        is_last: bool,
    ) -> std::io::Result<()> {
        let expr = self.arenas.expressions.get(node.id);
        let (branch, extend) = if is_last { (LAST, PAD) } else { (MID, VERT) };
        writeln!(w, "{prefix}{branch}{expr} {}{}", node.span, self.inline_name(expr))?;
        self.print_children(w, &format!("{prefix}{extend}"), node)
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

    fn print_children<W: Write>(&self, w: &mut W, prefix: &str, node: &ExpressionNode) -> std::io::Result<()> {
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
            Expression::SizeofType(type_id) => self.print_type(w, prefix, type_id, true)?,
            Expression::ConstantExpression(node) => self.print_expression(w, prefix, node, true)?,
            Expression::Ternary(cond, then, otherwise) => {
                self.print_expression(w, prefix, cond, false)?;
                self.print_expression(w, prefix, then, false)?;
                self.print_expression(w, prefix, otherwise, true)?;
            }
            Expression::Cast(type_id, node) => {
                self.print_type(w, prefix, type_id, false)?;
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

    fn print_type<W: Write>(&self, w: &mut W, prefix: &str, _: &Type, is_last: bool) -> std::io::Result<()> {
        let branch = if is_last { LAST } else { MID };
        writeln!(w, "{prefix}{branch}Type")
    }
}
