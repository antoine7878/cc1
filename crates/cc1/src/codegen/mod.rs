pub mod alloca_collector;
pub mod expression;
pub mod generator;
pub mod types;

pub use alloca_collector::AllocaCollector;
pub use generator::{Generator, generate};

#[macro_export]
macro_rules! emit { ($self:ident, $($arg:tt)*) => {{
        let _ = write!($self.w, $($arg)*);
    }};
}

#[macro_export]
macro_rules! emitln {
    ($self:ident $(, $($arg:tt)*)?) => {{
        let _ = writeln!($self.w $(, $($arg)*)?);
    }};
}
