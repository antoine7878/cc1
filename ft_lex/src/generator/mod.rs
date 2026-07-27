pub mod c_generator;
pub mod lang;
pub mod rs_generator;

pub use c_generator::CGenerator;
pub use lang::Lang;
pub use rs_generator::RSGenerator;

use crate::front::Lex;

pub trait Generator {
    fn dump_tables(&self, dfa: &Lex) -> String;
    fn dump_actions(&self, code_fragments: &[String]) -> String;
    fn dump_defines(&self, lex: &Lex) -> String;
    fn dump_tokens(&self, tokens: &[String]) -> String;
}
