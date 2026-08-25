use cc1::ast::print::AstPrinter;
use cc1::parser::{self, Context};
use cc1::pipeline::Pipeline;
use cc1::semantic::Analyzer;

fn main() {
    Pipeline::default()
        .then(parser::parse_args)
        .then(parser::parse_source)
        .peek(AstPrinter::print)
        .then(Analyzer::analyze)
        .peek(Context::dump_symbols)
        .finally(Context::dump_diagnostics);
}
