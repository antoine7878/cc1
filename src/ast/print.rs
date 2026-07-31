use std::io::{Write, stdout};

use crate::ast::expression::ExpressionNode;
use crate::ast::{Expression, Name, TypeNode};
use crate::context::Context;

impl Context {
    pub fn print_ast(&mut self, node: &ExpressionNode) -> std::io::Result<()> {
        self.print_expression(&mut stdout(), &mut "".to_string(), node)
    }

    pub fn print_expression<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &ExpressionNode,
    ) -> std::io::Result<()> {
        let expr = self.arenas.expressions.get(node.id);
        writeln!(w, "{prefix}{expr} {}", node.span)?;
        self.print_expression2(w, prefix, node)?;
        Ok(())
    }
    pub fn print_expression2<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        node: &ExpressionNode,
    ) -> std::io::Result<()> {
        let expr = self.arenas.expressions.get(node.id);
        write!(w, "{}", prefix)?;
        prefix.push_str("  ");
        match expr {
            Expression::Constant(name) | Expression::Identifier(name) | Expression::StringLiteral(name) => {
                self.print_name(w, name)?;
            }
            Expression::PostInc(id)
            | Expression::PostDec(id)
            | Expression::PreInc(id)
            | Expression::PreDec(id)
            | Expression::Addr(id)
            | Expression::Deref(id)
            | Expression::Plus(id)
            | Expression::Minus(id)
            | Expression::BitNot(id)
            | Expression::Not(id) => self.print_expression(w, prefix, id)?,
            Expression::Add(rhs, lhs)
            | Expression::Sub(rhs, lhs)
            | Expression::Mul(rhs, lhs)
            | Expression::Div(rhs, lhs)
            | Expression::Mod(rhs, lhs)
            | Expression::Right(rhs, lhs)
            | Expression::Left(rhs, lhs)
            | Expression::Greater(rhs, lhs)
            | Expression::Lower(rhs, lhs)
            | Expression::GreaterEq(rhs, lhs)
            | Expression::LowerEq(rhs, lhs)
            | Expression::Eq(rhs, lhs)
            | Expression::Neq(rhs, lhs)
            | Expression::BitAnd(rhs, lhs)
            | Expression::BitOr(rhs, lhs)
            | Expression::BitXor(rhs, lhs)
            | Expression::And(rhs, lhs)
            | Expression::Or(rhs, lhs)
            | Expression::Assign(rhs, lhs)
            | Expression::MulAssign(rhs, lhs)
            | Expression::DivAssign(rhs, lhs)
            | Expression::ModAssign(rhs, lhs)
            | Expression::AddAssign(rhs, lhs)
            | Expression::SubAssign(rhs, lhs)
            | Expression::LeftAssign(rhs, lhs)
            | Expression::RightAssign(rhs, lhs)
            | Expression::AndAssign(rhs, lhs)
            | Expression::XorAssign(rhs, lhs)
            | Expression::OrAssign(rhs, lhs)
            | Expression::List(rhs, lhs)
            | Expression::ArrayAcces(rhs, lhs)
            | Expression::FunctionCall(rhs, Some(lhs)) => self.print_binop(w, prefix, rhs, lhs)?,
            Expression::FunctionCall(fun, None) => self.print_expression(w, prefix, fun)?,
            Expression::DotAcces(tag, name) => self.print_access(w, prefix, tag, name)?,
            Expression::PtrAcces(tag, name) => self.print_access(w, prefix, tag, name)?,
            Expression::SizeofExpr(node) => self.print_expression(w, prefix, node)?,
            Expression::SizeofType(type_id) => self.print_type(w, type_id)?,
            Expression::ConstantExpression(node) => self.print_expression(w, prefix, node)?,
            Expression::Ternary(cond, rhs, lhs) => {
                self.print_expression(w, prefix, cond)?;
                self.print_expression(w, prefix, rhs)?;
                self.print_expression(w, prefix, lhs)?;
            }
            Expression::Cast(type_id, node) => {
                self.print_type(w, type_id)?;
                self.print_expression(w, prefix, node)?;
            }
        }
        prefix.pop();
        prefix.pop();
        // writeln!(w)?;
        Ok(())
    }

    fn print_binop<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        lhs: &ExpressionNode,
        rhs: &ExpressionNode,
    ) -> std::io::Result<()> {
        self.print_expression(w, prefix, rhs)?;
        self.print_expression(w, prefix, lhs)
    }

    fn print_access<W: Write>(
        &self,
        w: &mut W,
        prefix: &mut String,
        tag: &ExpressionNode,
        name: &Name,
    ) -> std::io::Result<()> {
        self.print_expression(w, prefix, tag)?;
        self.print_name(w, name)
    }

    fn print_name<W: Write>(&self, w: &mut W, name: &Name) -> std::io::Result<()> {
        writeln!(w, "{} {}", name.span, self.arenas.names.get(name.id))
    }

    fn print_type<R: Write>(&self, w: &mut R, _: &TypeNode) -> std::io::Result<()> {
        write!(w, "Type")
    }
}
