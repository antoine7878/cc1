use cc1::ast::print::AstPrinter;
use cc1::context::Context;
use cc1::parser;
use cc1::pipeline::{Pipeline, parse_args};
use cc1::semantic::Analyzer;

fn main() {
    Pipeline::default()
        .pass(parse_args)
        .checkpoint()
        .pass(parser::parse_source)
        .checkpoint()
        .pass(Analyzer::init)
        .pass(Analyzer::resolve_names)
        .pass(Analyzer::check_constants)
        .pass(Analyzer::mark_uses)
        .pass(Analyzer::finish)
        .checkpoint()
        .report(AstPrinter::print)
        .report(Context::dump_symbols)
        .finally(Context::dump_diagnostics);
}
