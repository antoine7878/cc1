use cc1::args::parse_args;
use cc1::pipeline::Pipeline;
use cc1::semantic::Analyzer;
use cc1::{codegen, parser, report};
fn main() {
    Pipeline::default()
        .pass_group([parse_args])
        .pass_group([parser::parse_source])
        .then(Analyzer::begin)
        .pass_group([
            Analyzer::resolve_names,
            Analyzer::check_constants,
            Analyzer::mark_uses,
            Analyzer::finish_externals,
            Analyzer::finalize_layouts,
        ])
        .then(Analyzer::end)
        .run(codegen::generate)
        .finally(report::dump_diagnostics);
}
