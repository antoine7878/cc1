use std::sync::atomic::{AtomicUsize, Ordering};

use cc1::parser::{Context, Span};
use cc1::pipeline::Pipeline;
use cc1::semantic::{Diagnosis, DiagnosisNode, ExpectedTokens};

static TAPPED: AtomicUsize = AtomicUsize::new(0);

fn mark(mut ctx: Context) -> Context {
    ctx.file_name.push('m');
    ctx
}

fn fail(mut ctx: Context) -> Context {
    ctx.diagnosis
        .push(DiagnosisNode::new(
            Diagnosis::SyntaxError {
                found: "';'",
                expected: ExpectedTokens::new("';'", &[]),
            },
            Span::default(),
        ));
    ctx
}

fn tap(ctx: &Context) {
    TAPPED.fetch_add(ctx.file_name.len() + 1, Ordering::SeqCst);
}

#[test]
fn a_clean_pipeline_runs_every_pass_in_order() {
    let (ctx, stopped) = Pipeline::default().pass(mark).pass(mark).pass(mark).finish();
    assert_eq!(ctx.file_name, "mmm");
    assert!(!stopped);
}

#[test]
fn a_pass_that_reports_an_error_stops_the_pipeline() {
    let pipeline = Pipeline::default().pass(mark).pass(fail);
    assert!(pipeline.stopped());
    let (ctx, stopped) = pipeline.pass(mark).pass(mark).finish();
    assert_eq!(ctx.file_name, "m");
    assert_eq!(ctx.diagnosis.len(), 1);
    assert!(stopped);
}

#[test]
fn a_pass_that_reports_nothing_leaves_the_pipeline_running() {
    let pipeline = Pipeline::default().pass(mark);
    assert!(!pipeline.stopped());
    let (_, stopped) = pipeline.finish();
    assert!(!stopped);
}

#[test]
fn tap_observes_the_context_without_stopping_it() {
    TAPPED.store(0, Ordering::SeqCst);
    let (ctx, stopped) = Pipeline::default().pass(mark).report(tap).pass(mark).finish();
    assert_eq!(TAPPED.load(Ordering::SeqCst), 2);
    assert_eq!(ctx.file_name, "mm");
    assert!(!stopped);
}
