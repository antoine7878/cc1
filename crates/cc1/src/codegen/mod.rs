pub mod generator;
pub use generator::*;

use crate::context::Context;

use std::io::Write;

pub trait LLVM {
    fn emit<W: Write>(&self, w: &mut Generator<W>, ctx: &Context);
}

#[macro_export]
macro_rules! emit {
    ($self:ident, $($arg:tt)*) => {{
        $self.lead();
        let _ = write!($self, $($arg)*);
        $self.start_line = false;
    }};
}

#[macro_export]
macro_rules! emitln {
    ($self:ident $(, $($arg:tt)*)?) => {{
        $self.lead();
        let _ = writeln!($self $(, $($arg)*)?);
        $self.start_line = true;
    }};
}
