use std::io::{Write, stdout};

use crate::ast::visit::{walk_external_declaration, walk_translation_unit};
use crate::ast::{
    ExternalDeclaration, ExternalDeclarationNode, JumpStatement, JumpStatementNode, TranslationUnitNode, Visitor,
};
use crate::context::Context;
use crate::semantic::ResolvedType;

pub fn generate(ctx: Context) -> Context {
    let mut generator = Generator::new(stdout());
    walk_translation_unit(&mut generator, &ctx, &ctx.ast);
    ctx
}

#[derive(Debug)]
pub struct Generator<W: Write> {
    w: W,
}

impl<W: Write> Generator<W> {
    fn new(w: W) -> Self {
        Self { w }
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

// macro_rules! emit {
//     ($self:ident, $($arg:tt)*) => {{
//         let _ = write!($self.w, $($arg)*);
//     }};
// }

macro_rules! emitln {
    ($self:ident) => {{
        let _ = write!($self, "\n");
    }};
    ($self:ident, $($arg:tt)*) => {{
        let _ = write!($self, $($arg)*);
        let _ = write!($self, "\n");
    }};
}

impl<W: Write> Visitor for Generator<W> {
    fn visit_translation_unit(&mut self, ctx: &Context, node: &TranslationUnitNode) {
        emitln!(self, r#"target datalayout = "<{} layout>""#, ctx.target.name);
        emitln!(self, r#"target triple = "{}-pc-linux-gnu""#, ctx.target.name);
        emitln!(self);
        walk_translation_unit(self, ctx, node);
    }

    fn visit_external_declaration(&mut self, ctx: &Context, node: &ExternalDeclarationNode) {
        match &node.decl {
            ExternalDeclaration::Function(fn_decl) => {
                let sym = ctx.sema.declarations[&fn_decl.declarator.id].resolve(ctx);
                let ty = sym.ty.expect("pas sym type").id.resolve(ctx);
                let ResolvedType::Function { ret, params: _ } = ty else { panic!("pas function") };
                let ret_ty = ret.id.resolve(ctx);
                emitln!(
                    self,
                    "define i{} @{}({}) {{\n",
                    ctx.target.layout(ret_ty).expect("pas layout").size * 8,
                    sym.name.id.resolve(ctx),
                    ""
                );
                emitln!(self, "ret i32 42");
                emitln!(self, "}}");
            }
            ExternalDeclaration::Declaration(_decl) => todo!(),
        }
        walk_external_declaration(self, ctx, node);
    }

    fn visit_jump_statement(&mut self, ctx: &Context, node: &JumpStatementNode) {
        match node.stmt {
            JumpStatement::Return(Some(e)) => {
                let ty = ctx.sema.expr_types.get(node.id);
                emitln!(self, "ret")
            }
            _ => todo!(),
        }
    }
}
