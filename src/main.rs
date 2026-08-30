use cc1::ast::print::AstPrinter;
use cc1::context::Context;
use cc1::parser;
use cc1::pipeline::{Pipeline, parse_args};
use cc1::semantic::Analyzer;

fn main() {
    Pipeline::default()
        .pass(parse_args)
        .pass(parser::parse_source)
        .pass(Analyzer::analyze)
        .report(AstPrinter::print)
        .report(Context::dump_symbols)
        .finally(Context::dump_diagnostics);
}
