use cc1::ast::print::AstPrinter;
use cc1::parser::{self, Context};
use cc1::pipeline::Pipeline;
use cc1::semantic::Analyzer;

fn main() {
    Pipeline::default()
        .pass(parser::parse_args)
        .pass(parser::parse_source)
        .report(AstPrinter::print)
        .pass(Analyzer::analyze)
        .report(Context::dump_symbols)
        .finally(Context::dump_diagnostics);
}
