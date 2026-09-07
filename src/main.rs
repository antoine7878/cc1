use cc1::ast::print::AstPrinter;
use cc1::context::Context;
use cc1::parser;
use cc1::pipeline::{Pipeline, parse_args};
use cc1::semantic::Analyzer;

fn main() {
    Pipeline::default()
        .pass_group([parse_args])
        .pass_group([parser::parse_source])
        .pass_group([
            Analyzer::init,
            Analyzer::resolve_names,
            Analyzer::check_constants,
            Analyzer::mark_uses,
            Analyzer::finish_externals,
            Analyzer::finalize_layouts,
        ])
        .report(AstPrinter::print)
        .report(Context::dump_symbols)
        .finally(Context::dump_diagnostics);
}
