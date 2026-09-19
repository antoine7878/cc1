use std::thread;

use cc1::context::{Context, ctx, install_context};
use cc1::semantic::{Sema, install_sema, sema};
use cc1::target::I386;

fn named(name: &str) -> Context {
    let mut ctx = Context::default();
    ctx.set_file_name(name.to_string());
    ctx
}

#[test]
fn install_makes_the_context_readable_through_ctx() {
    install_context(named("main.c"));
    assert_eq!(ctx().file_name, "main.c");
}

#[test]
fn a_later_install_replaces_the_context_on_the_same_thread() {
    let first = install_context(named("first.c"));
    install_context(named("second.c"));
    assert_eq!(ctx().file_name, "second.c");
    assert_eq!(first.file_name, "first.c");
}

#[test]
fn installs_are_isolated_per_thread() {
    install_context(named("outer.c"));
    let inner = thread::spawn(|| {
        install_context(named("inner.c"));
        ctx().file_name.clone()
    })
    .join()
    .unwrap();
    assert_eq!(inner, "inner.c");
    assert_eq!(ctx().file_name, "outer.c");
}

#[test]
fn install_makes_the_sema_readable_through_sema() {
    let mut s = Sema::new(I386);
    s.diagnostics.clear();
    let installed = install_sema(s);
    assert!(std::ptr::eq(installed, sema()));
}

#[test]
fn string_ids_resolve_through_the_installed_context() {
    let mut context = named("ids.c");
    let id = context.arenas.names.intern("hello".to_string());
    install_context(context);
    assert_eq!(id.resolve(), "hello");
}
