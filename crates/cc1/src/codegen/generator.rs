use std::io::{self, Write, stdout};

use crate::ast::{ExternalDeclaration, ExternalDeclarationNode, Visitor};
use crate::context::Context;
use crate::semantic::{QualifiedType, ResolvedType};

pub fn generate(ctx: Context) -> Context {
    if let Err(e) = try_generate(&ctx) {
        eprint!("llvm err: {}", e);
    }
    ctx
}

fn print_target<W: Write>(w: &mut W, ctx: &Context) -> io::Result<()> {
    writeln!(w, r#"target datalayout = "<{} layout>""#, ctx.target.name)?;
    writeln!(w, r#"target triple = "{}-pc-linux-gnu""#, ctx.target.name)?;
    writeln!(w)
}

fn try_generate(ctx: &Context) -> io::Result<()> {
    let w = &mut stdout();
    print_target(w, ctx)?;
    let mut emitter = Emitter::default();
    for ext_decl in &ctx.ast.declarations {
        emitter.visit_external_declaration(ctx, ext_decl);
        w.write_all(emitter.entry_allocas.as_bytes())?;
        w.write_all(emitter.body.as_bytes())?;
        emitter.clear();
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct Emitter {
    entry_allocas: String,
    body: String,
}

impl Emitter {
    fn clear(&mut self) {
        self.entry_allocas.clear();
        self.body.clear();
    }
}

macro_rules! emit {
    ($buf:expr, $($arg:tt)*) => {{
        let _ = ::std::fmt::Write::write_fmt(&mut $buf, ::std::format_args!($($arg)*));
    }};
}

macro_rules! emitln {
    ($buf:expr) => {{
        $buf.push('\n');
    }};
    ($buf:expr, $($arg:tt)*) => {{
        let _ = ::std::fmt::Write::write_fmt(&mut $buf, ::std::format_args!($($arg)*));
        $buf.push('\n');
    }};
}

impl Visitor for Emitter {
    fn visit_external_declaration(&mut self, ctx: &Context, node: &ExternalDeclarationNode) {
        match &node.decl {
            ExternalDeclaration::Function(fn_decl) => {
                let &sym_id = ctx.sema.declarations.get(&fn_decl.declarator.id).unwrap();
                let sym = sym_id.resolve(ctx);
                let qty = sym.ty.unwrap();
                let ty = qty.id.resolve(ctx);
                let ResolvedType::Function { ret, params: _ } = ty else { panic!() };
                let ret_ty = ret.id.resolve(ctx);
                emitln!(
                    self.body,
                    "define {}{} @{}({}) {{\n",
                    ret_ty.interge_prefix(),
                    ctx.target.layout(ty).unwrap().size * 8,
                    sym.name.id.resolve(ctx),
                    ""
                );
                emitln!(self.body, "}}");
            }
            ExternalDeclaration::Declaration(_decl) => todo!(),
        }
        // define i32 @main() {
        //   ret i32 42
        // }
    }
}
