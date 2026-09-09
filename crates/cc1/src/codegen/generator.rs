use std::io::{Write, stdout};

use crate::ast::visit::{walk_compound_statement, walk_jump_statement, walk_translation_unit};
use crate::ast::{
    CompoundStatementNode, Expression, ExpressionNode, FunctionDefinitionNode, JumpStatement, JumpStatementNode,
    TranslationUnitNode, Visitor,
};
use crate::codegen::LLVM;
use crate::context::Context;
use crate::semantic::{QualifiedType, ResolvedType};
use crate::{emit, emitln};

pub fn generate(ctx: Context) -> Context {
    let mut generator = Generator::new(stdout());
    generator.visit_translation_unit(&ctx, &ctx.ast);
    ctx
}

#[derive(Debug)]
pub struct Generator<W: Write> {
    w: W,
    start_line: bool,
    indent: usize,
}

impl<W: Write> Generator<W> {
    const I: usize = 4;

    fn new(w: W) -> Self {
        Self {
            w,
            start_line: true,
            indent: 0,
        }
    }

    fn add_indent(&mut self) {
        self.indent += Self::I;
    }

    fn sub_indent(&mut self) {
        self.indent -= Self::I;
    }

    pub fn lead(&mut self) {
        if self.start_line {
            let _ = write!(self.w, "{:1$}", "", self.indent.saturating_sub(1));
        } else {
            let _ = write!(self.w, " ");
        }
    }
}

impl<W: Write> Visitor for Generator<W> {
    fn visit_translation_unit(&mut self, ctx: &Context, node: &TranslationUnitNode) {
        emitln!(self, r#"target datalayout = "<{} layout>""#, ctx.target.name);
        emitln!(self, r#"target triple = "{}-pc-linux-gnu""#, ctx.target.name);
        emitln!(self);
        walk_translation_unit(self, ctx, node);
    }

    fn visit_function_definition(&mut self, ctx: &Context, node: &FunctionDefinitionNode) {
        let sym = ctx.sema.declarations[&node.declarator.id].resolve(ctx);
        let ty = sym.ty.expect("pas sym type").id.resolve(ctx);
        let ResolvedType::Function { ret, .. } = ty else { panic!("pas function") };
        let ret_ty = ret.id.resolve(ctx);
        emit!(self, "define");
        ret_ty.emit(self, ctx);
        emitln!(self, "@{}() {{", sym.name.id.resolve(ctx));
        self.visit_compound_statement(ctx, &node.body);
        emitln!(self, "}}");
    }

    fn visit_jump_statement(&mut self, ctx: &Context, node: &JumpStatementNode) {
        match &node.stmt {
            JumpStatement::Return(Some(_)) => emit!(self, "ret"),
            JumpStatement::Return(None) => emit!(self, "ret void"),
            _ => todo!(),
        }
        walk_jump_statement(self, ctx, node);
        emitln!(self);
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        let re = &ctx.sema.expr_types[node.id];
        re.casted_ty().emit(self, ctx);
        match node.id.resolve(ctx) {
            Expression::Constant(value) => emit!(self, "{}", value.value),
            _ => todo!(),
        }
    }

    fn visit_compound_statement(&mut self, ctx: &Context, node: &CompoundStatementNode) {
        self.add_indent();
        walk_compound_statement(self, ctx, node);
        self.sub_indent();
    }
}

impl<W: Write> Write for Generator<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.w.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.w.flush()
    }
}

impl LLVM for ResolvedType {
    fn emit<W: Write>(&self, w: &mut Generator<W>, ctx: &Context) {
        match self {
            ResolvedType::Void => emit!(w, "void"),
            ResolvedType::Char
            | ResolvedType::SignedChar
            | ResolvedType::UnsignedChar
            | ResolvedType::Short
            | ResolvedType::UnsignedShort
            | ResolvedType::Long
            | ResolvedType::UnsignedInt
            | ResolvedType::UnsignedLong
            | ResolvedType::Int => emit!(w, "i{}", ctx.target.layout(self).unwrap().size * 8),
            ResolvedType::Float => emit!(w, "float"),
            ResolvedType::Double => emit!(w, "double"),
            ResolvedType::LongDouble => emit!(w, "x86_fp80"),
            ResolvedType::Function { .. } => todo!(),
            ResolvedType::Tag { .. } => todo!(),
            ResolvedType::Array { .. } => todo!(),
            ResolvedType::Pointer(_) => todo!(),
        }
    }
}

impl LLVM for QualifiedType {
    fn emit<W: Write>(&self, w: &mut Generator<W>, ctx: &Context) {
        self.id.resolve(ctx).emit(w, ctx);
    }
}
