pub mod c_generator;
pub mod dumper;
pub mod lang;
pub mod rs_generator;

use crate::models::{TokenData, Yacc};
use crate::parser::LALRParser;
use crate::utils::{Args, YaccError};
use c_generator::CGenerator;
use dumper::Dumper;
use rs_generator::RSGenerator;

#[rustfmt::skip]
pub trait Generator {
    fn dump_code_before(&self, w: &mut Dumper, yacc: &Yacc, line: bool) -> Result<(), YaccError>;
    fn dump_tables(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError>;
    fn dump_debug(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError>;
    fn dump_actions(&self, w: &mut Dumper, parser: &LALRParser, line: bool) -> Result<(), YaccError>;
    fn dump_defines(&self, w: &mut Dumper, parser: &LALRParser, args: &Args) -> Result<(), YaccError>;
    fn dump_tokens_src(&self, w: &mut Dumper, tokens: &[TokenData]) -> Result<(), YaccError>;
    fn dump_tokens_hdr(&self, w: &mut Dumper, tokens: &[TokenData]) -> Result<(), YaccError>;
    fn dump_union(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError>;
}
