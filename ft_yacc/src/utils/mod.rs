mod args;
mod bitset;
mod error;
mod func;
mod graph;

pub use args::Args;
pub use bitset::BitSet;
pub use error::YaccError;
pub use func::{escape_of_char, str_of_escape};
pub use graph::Graph;

#[macro_export]
macro_rules! iformat {
    ($indent:expr, $($arg:tt)*) => {{
        format!("{}{}", " ".repeat($indent), format!($($arg)*))
    }};
}
