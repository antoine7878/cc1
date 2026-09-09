pub mod generator;
pub use generator::*;

use crate::context::Context;

use std::io::Write;

pub trait LLVM {
    fn emit<W: Write>(w: W, ctx: &Context);
}
