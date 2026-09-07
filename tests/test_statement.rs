// 6.6.1 Labeled statements

// ---- 6.1.2.1 a label has function scope: visible before and after its definition ----

accept!(label_backward_goto, "void f(void){ l: ; goto l; }");
accept!(label_forward_goto, "void f(void){ goto l; l: ; }");
accept!(label_goto_out_of_a_block, "void f(void){ { l: ; } goto l; }");
accept!(label_goto_into_a_block, "void f(void){ goto l; { l: ; } }");
accept!(label_inside_a_loop, "void f(void){ while (1) { l: goto l; } }");
accept!(
    label_out_of_a_switch,
    "void f(void){ switch (1) { case 1: goto l; } l: ; }"
);
accept!(
    label_several_in_one_function,
    "void f(void){ l: ; m: ; goto m; goto l; }"
);

// A label lives in its own namespace, so it never collides with an ordinary identifier.
accept!(
    label_shares_a_name_with_an_object,
    "void f(void){ int x; x: ; goto x; x = 1; }"
);

// Each function gets its own labels, so the same name may be reused.
accept!(
    label_reused_in_another_function,
    "void f(void){ l: ; } void g(void){ l: ; }"
);

// ---- 6.6.1 the identifier must name a label of the current function ----

reject!(label_goto_undefined, "void f(void){ goto nowhere; }");
reject!(label_defined_twice, "void f(void){ l: ; l: ; }");
reject!(
    label_goto_from_another_function,
    "void f(void){ l: ; } void g(void){ goto l; }"
);
reject!(label_goto_never_defined_anywhere, "void f(void){ goto l; }");

// ---- the labels recorded on each function definition ----

labels!(labels_are_empty_without_any, "void f(void){ ; }", &[("f", &[])]);
labels!(
    labels_are_collected_in_order,
    "void f(void){ a: ; b: ; }",
    &[("f", &["a", "b"])]
);
labels!(
    labels_are_collected_from_nested_statements,
    "void f(void){ while (1) { a: ; if (1) { b: ; } } }",
    &[("f", &["a", "b"])]
);
labels!(
    labels_belong_to_their_own_function,
    "void f(void){ a: ; } void g(void){ b: ; }",
    &[("f", &["a"]), ("g", &["b"])]
);
labels!(
    labels_record_a_forward_target,
    "void f(void){ goto l; l: ; }",
    &[("f", &["l"])]
);

// 6.6.4 Selection and iteration statements: the controlling expression is resolved too

reject!(control_if_undeclared, "void f(void){ if (undeclared_cond) ; }");
reject!(
    control_if_else_undeclared,
    "void f(void){ if (1) ; else undeclared_else; }"
);
reject!(
    control_switch_undeclared,
    "void f(void){ switch (undeclared_ctl) { case 1: ; } }"
);
reject!(control_while_undeclared, "void f(void){ while (undeclared_while) ; }");
reject!(control_do_undeclared, "void f(void){ do ; while (undeclared_do); }");
reject!(
    control_for_init_undeclared,
    "void f(void){ for (undeclared_init; 1; 1) ; }"
);
reject!(
    control_for_condition_undeclared,
    "void f(void){ for (1; undeclared_cond; 1) ; }"
);
reject!(
    control_for_step_undeclared,
    "void f(void){ for (1; 1; undeclared_step) ; }"
);
reject!(
    control_if_requires_a_scalar,
    "struct S { int x; }; void f(struct S s){ if (s) ; }"
);
reject!(
    control_switch_requires_an_integer,
    "void f(double d){ switch (d) { case 1: ; } }"
);
