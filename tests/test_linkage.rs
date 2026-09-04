use crate::common::accepted;
use cc1::semantic::Diagnosis;

placements!(
    placement_tentative_external,
    "int x;",
    &[("x", "external", "static", "definition")]
);

placements!(
    placement_tentative_internal,
    "static int x;",
    &[("x", "internal", "static", "definition")]
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

accept!(
    linkage_accept_static_then_extern_inherits,
    "static int b; extern int b;"
);
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
accept!(
    linkage_accept_composite_array_length,
    "extern int a[]; extern int a[10];"
);
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

reject!(tentative_never_completed_external, "struct S; struct S x;");
reject!(tentative_never_completed_internal, "struct S; static struct S x;");
reject!(
    tentative_never_completed_used,
    "struct S; static struct S x; struct S *p(void) { return &x; }"
);

accept!(
    tentative_completed_later,
    "struct S; static struct S x; struct S { int i; };"
);
accept!(
    tentative_array_completed,
    "static int a[]; int f(void) { return a[0]; }"
);
accept!(tentative_object_used, "static int sv; int f(void) { return sv; }");
accept!(
    internal_function_defined_then_used,
    "static int sf(void) { return 1; } int use(void) { return sf(); }"
);
accept!(
    internal_function_never_used,
    "static int sf(void); int use(void) { return 0; }"
);
accept!(
    internal_function_used_under_sizeof,
    "static int sf(void); int use(void) { return sizeof(&sf); }"
);
accept!(
    external_function_never_defined,
    "int ef(void); int use(void) { return ef(); }"
);
accept!(
    external_incomplete_array_declaration,
    "extern int a[]; int *p(void) { return a; }"
);

reject!(
    tentative_array_sizeof_before_completion,
    "int a[]; enum e { P = sizeof(a) };"
);

recover!(
    internal_function_never_defined,
    "static int sf(void); int use(void) { return sf(); }",
    [Diagnosis::InternalNeverDefined(_)],
    &[]
);
recover!(
    end_of_unit_diagnosis_order,
    "static int sf(void); struct S; static struct S x; int use(void) { return sf(); }",
    [
        Diagnosis::InternalNeverDefined(_),
        Diagnosis::TentativeNeverCompleted(_)
    ],
    &[]
);
