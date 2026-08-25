use cc1::ast::print::AstPrinter;
use cc1::parser::{self, Context};
use cc1::pipeline::Pipeline;
use cc1::semantic::Analyzer;

fn main() {
    Pipeline::default()
        .then(parser::parse_args)
        .then(parser::parse_source)
        .tap(AstPrinter::print)
        .then(Analyzer::analyze)
        .tap(Context::dump_symbols)
        .finaly(Context::dump_diagnostics);
}
