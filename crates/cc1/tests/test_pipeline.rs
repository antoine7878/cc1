use std::sync::atomic::{AtomicUsize, Ordering};

use cc1::context::Context;
use cc1::pipeline::Pipeline;
use cc1::semantic::{Diagnosis, DiagnosisNode, ExpectedTokens, Sema};
use libft::Span;

static TAPPED: AtomicUsize = AtomicUsize::new(0);
static CHECKED: AtomicUsize = AtomicUsize::new(0);

fn mark(mut ctx: Context) -> Context {
    ctx.file_name.push('m');
    ctx
}

fn fail(mut ctx: Context) -> Context {
    ctx.diagnosis.push(DiagnosisNode::new(
        Diagnosis::SyntaxError { found: "';'", expected: ExpectedTokens::new("';'", &[]) },
        Span::default(),
    ));
    ctx
}

fn tap(ctx: &Context) {
    TAPPED.fetch_add(ctx.file_name.len() + 1, Ordering::SeqCst);
}

fn tap_checked(ctx: &Context) {
    CHECKED.fetch_add(ctx.file_name.len() + 1, Ordering::SeqCst);
}

#[test]
fn a_clean_pipeline_runs_every_pass_in_order() {
    let (ctx, stopped) = Pipeline::default().pass(mark).pass(mark).pass(mark).finish();
    assert_eq!(ctx.file_name, "mmm");
    assert!(!stopped);
}

#[test]
fn a_pass_that_reports_an_error_does_not_stop_the_pipeline() {
    let pipeline = Pipeline::default().pass(mark).pass(fail);
    assert!(pipeline.failed());
    assert!(!pipeline.stopped());
    let (ctx, stopped) = pipeline.pass(mark).pass(mark).finish();
    assert_eq!(ctx.file_name, "mmm");
    assert_eq!(ctx.diagnosis.len(), 1);
    assert!(!stopped);
}

#[test]
fn a_check_after_an_error_stops_the_pipeline() {
    let pipeline = Pipeline::default().pass(mark).pass(fail).checkpoint();
    assert!(pipeline.stopped());
    let (ctx, stopped) = pipeline.pass(mark).pass(mark).finish();
    assert_eq!(ctx.file_name, "m");
    assert_eq!(ctx.diagnosis.len(), 1);
    assert!(stopped);
}

#[test]
fn a_check_stays_stopped_once_an_earlier_pass_failed() {
    let pipeline = Pipeline::default().pass(fail).checkpoint().pass(mark).checkpoint();
    assert!(pipeline.stopped());
    let (ctx, _) = pipeline.finish();
    assert_eq!(ctx.file_name, "");
}

#[test]
fn a_check_on_a_clean_pipeline_leaves_it_running() {
    let pipeline = Pipeline::default().pass(mark).checkpoint();
    assert!(!pipeline.failed());
    assert!(!pipeline.stopped());
    let (ctx, stopped) = pipeline.pass(mark).finish();
    assert_eq!(ctx.file_name, "mm");
    assert!(!stopped);
}

#[test]
fn a_pass_that_reports_nothing_leaves_the_pipeline_running() {
    let pipeline = Pipeline::default().pass(mark);
    assert!(!pipeline.stopped());
    let (_, stopped) = pipeline.finish();
    assert!(!stopped);
}

#[test]
fn a_report_runs_after_an_error_until_a_check_stops_the_pipeline() {
    CHECKED.store(0, Ordering::SeqCst);
    let (_, stopped) =
        Pipeline::default().pass(mark).pass(fail).report(tap_checked).checkpoint().report(tap_checked).finish();
    assert_eq!(CHECKED.load(Ordering::SeqCst), 2);
    assert!(stopped);
}

#[test]
fn tap_observes_the_context_without_stopping_it() {
    TAPPED.store(0, Ordering::SeqCst);
    let (ctx, stopped) = Pipeline::default().pass(mark).report(tap).pass(mark).finish();
    assert_eq!(TAPPED.load(Ordering::SeqCst), 2);
    assert_eq!(ctx.file_name, "mm");
    assert!(!stopped);
}

fn to_sema(ctx: Context) -> Sema {
    Sema::new(ctx.target.clone())
}

fn to_unit(_: Sema) {}

fn mark_sema(mut sema: Sema) -> Sema {
    sema.diagnosis.clear();
    sema
}

#[test]
fn a_phase_change_keeps_the_failure_and_stop_state() {
    let pipeline = Pipeline::default().pass(fail).checkpoint().then(to_sema);
    assert!(pipeline.failed());
    assert!(pipeline.stopped());
    let pipeline = pipeline.pass(mark_sema).then(to_unit);
    assert!(pipeline.failed());
    assert!(pipeline.stopped());
}

#[test]
fn a_phase_change_on_a_clean_pipeline_leaves_it_running() {
    let pipeline = Pipeline::default().pass(mark).then(to_sema).pass(mark_sema).then(to_unit);
    assert!(!pipeline.failed());
    assert!(!pipeline.stopped());
}

#[test]
fn a_run_is_skipped_once_the_pipeline_is_stopped() {
    TAPPED.store(0, Ordering::SeqCst);
    let pipeline = Pipeline::default().pass(fail).checkpoint().then(to_sema).then(to_unit).run(|| {
        TAPPED.fetch_add(1, Ordering::SeqCst);
    });
    assert!(pipeline.stopped());
    assert_eq!(TAPPED.load(Ordering::SeqCst), 0);
}

#[test]
fn a_run_on_a_clean_pipeline_executes() {
    CHECKED.store(0, Ordering::SeqCst);
    let pipeline = Pipeline::default().then(to_sema).then(to_unit).run(|| {
        CHECKED.fetch_add(1, Ordering::SeqCst);
    });
    assert!(!pipeline.stopped());
    assert_eq!(CHECKED.load(Ordering::SeqCst), 1);
}
