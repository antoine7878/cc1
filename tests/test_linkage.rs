use crate::common::accepted;

placements!(placement_tentative_external, "int x;", &[("x", "external", "static", "tentative")]);

placements!(
    placement_tentative_internal,
    "static int x;",
    &[("x", "internal", "static", "tentative")]
);

placements!(
    placement_declaration_external,
    "extern int x;",
    &[("x", "external", "static", "declaration")]
);

placements!(
    placement_definition_external,
    "int x = 1;",
    &[("x", "external", "static", "definition")]
);

placements!(
    placement_block_scope_mix,
    "void f(void) { int a; static int b; extern int c; }",
    &[
        ("f", "external", "none", "definition"),
        ("a", "none", "automatic", "definition"),
        ("b", "none", "static", "definition"),
        ("c", "external", "static", "declaration"),
    ]
);

placements!(
    placement_static_then_definition_one_entry,
    "static int sf(void); int sf(void) { return 0; }",
    &[("sf", "internal", "none", "definition")]
);

placements!(
    placement_function_declaration_no_duration,
    "int fn(void);",
    &[("fn", "external", "none", "declaration")]
);

placements!(
    placement_function_definition_no_duration,
    "int fn(void) { return 0; }",
    &[("fn", "external", "none", "definition")]
);

placements!(
    placement_static_function_definition_no_duration,
    "static int sfn(void) { return 0; }",
    &[("sfn", "internal", "none", "definition")]
);

reject!(linkage_reject_extern_then_static, "extern int a; static int a;");
reject!(linkage_reject_external_then_static, "int c; static int c;");
reject!(linkage_reject_static_then_external, "static int d; int d;");
reject!(
    linkage_reject_block_extern_then_file_static,
    "void f(void) { extern int x; } static int x;"
);
reject!(
    linkage_reject_function_then_static_definition,
    "int g(void); static int g(void) { return 0; }"
);

accept!(linkage_accept_static_then_extern_inherits, "static int b; extern int b;");
accept!(
    linkage_accept_implicit_extern_function_inherits,
    "static int sf(void); int sf(void) { return 0; }"
);
accept!(
    linkage_accept_repeated_block_extern,
    "void f(void) { extern int x; extern int x; } int x;"
);
accept!(
    linkage_accept_no_linkage_local_never_meets_external,
    "void f(void) { int y; { extern double y; } }"
);
accept!(linkage_accept_composite_array_length, "extern int a[]; extern int a[10];");
accept!(linkage_accept_redeclare_tentative, "int g; int g;");
accept!(linkage_accept_tentative_then_definition, "int h; int h = 1;");
accept!(linkage_accept_definition_then_tentative, "int i = 1; int i;");
accept!(
    linkage_accept_internal_tentative_then_definition,
    "static int j; static int j = 1;"
);

#[test]
fn linkage_composite_array_length_describes() {
    let src = "extern int a[]; extern int a[10];";
    let unit = accepted(src);
    assert_eq!(unit.describe("a").as_deref(), Some("int[10]"), "{src}");
}
