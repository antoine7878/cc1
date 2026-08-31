mod args;
mod error;
mod func;
mod graph;

pub use args::Args;
pub use error::YaccError;
pub use func::{escape_of_char, str_of_escape};
pub use graph::Graph;
pub use libft::BitSet;

#[macro_export]
macro_rules! iformat {
    ($indent:expr, $($arg:tt)*) => {{
        format!("{}{}", " ".repeat($indent), format!($($arg)*))
    }};
}
