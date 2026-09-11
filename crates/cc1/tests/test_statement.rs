use cc1::semantic::Diagnosis;

// 6.6.1 Labeled statements

// ---- 6.1.2.1 a label has function scope: visible before and after its definition ----

accept!(label_backward_goto, "void f(void){ l: ; goto l; }");
accept!(label_forward_goto, "void f(void){ goto l; l: ; }");
accept!(label_goto_out_of_a_block, "void f(void){ { l: ; } goto l; }");
accept!(label_goto_into_a_block, "void f(void){ goto l; { l: ; } }");
accept!(label_inside_a_loop, "void f(void){ while (1) { l: goto l; } }");
accept!(label_out_of_a_switch, "void f(void){ switch (1) { case 1: goto l; } l: ; }");
accept!(label_several_in_one_function, "void f(void){ l: ; m: ; goto m; goto l; }");

// A label lives in its own namespace, so it never collides with an ordinary identifier.
accept!(label_shares_a_name_with_an_object, "void f(void){ int x; x: ; goto x; x = 1; }");

// Each function gets its own labels, so the same name may be reused.
accept!(label_reused_in_another_function, "void f(void){ l: ; } void g(void){ l: ; }");

// ---- 6.6.1 the identifier must name a label of the current function ----

reject!(label_goto_undefined, "void f(void){ goto nowhere; }");
reject!(label_defined_twice, "void f(void){ l: ; l: ; }");
reject!(label_goto_from_another_function, "void f(void){ l: ; } void g(void){ goto l; }");
reject!(label_goto_never_defined_anywhere, "void f(void){ goto l; }");

// ---- the labels recorded on each function definition ----

labels!(labels_are_empty_without_any, "void f(void){ ; }", &[("f", &[])]);
labels!(labels_are_collected_in_order, "void f(void){ a: ; b: ; }", &[("f", &["a", "b"])]);
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
labels!(labels_record_a_forward_target, "void f(void){ goto l; l: ; }", &[("f", &["l"])]);

// 6.6.4 Selection and iteration statements: the controlling expression is resolved too

reject!(control_if_undeclared, "void f(void){ if (undeclared_cond) ; }");
reject!(control_if_else_undeclared, "void f(void){ if (1) ; else undeclared_else; }");
reject!(control_switch_undeclared, "void f(void){ switch (undeclared_ctl) { case 1: ; } }");
reject!(control_while_undeclared, "void f(void){ while (undeclared_while) ; }");
reject!(control_do_undeclared, "void f(void){ do ; while (undeclared_do); }");
reject!(control_for_init_undeclared, "void f(void){ for (undeclared_init; 1; 1) ; }");
reject!(control_for_condition_undeclared, "void f(void){ for (1; undeclared_cond; 1) ; }");
reject!(control_for_step_undeclared, "void f(void){ for (1; 1; undeclared_step) ; }");
reject!(control_if_requires_a_scalar, "struct S { int x; }; void f(struct S s){ if (s) ; }");
reject!(control_switch_requires_an_integer, "void f(double d){ switch (d) { case 1: ; } }");

// ---- 6.6.4.1/6.6.6 the recorded jump targets and switch table ----

stmts!(
    stmt_loop_holds_its_jumps,
    "void f(void){ while (1) { break; continue; } }",
    &["#0 break->#3", "#1 continue->#3", "#3 loop"]
);
stmts!(
    stmt_switch_holds_its_cases,
    "void f(void){ switch (2) { case 1: break; default: ; } }",
    &["#0 break->#5", "#1 case Int(1)->#5", "#3 default->#5", "#5 switch int [Int(1)->#1] default=#3",]
);

// A break binds to the innermost loop or switch, a continue to the innermost loop,
// and a case to the innermost switch, each skipping enclosing scopes of the other kind.
stmts!(
    stmt_switch_inside_a_loop,
    "void f(void){ while (1) { switch (2) { case 1: break; continue; } } }",
    &["#0 break->#4", "#1 case Int(1)->#4", "#2 continue->#6", "#4 switch int [Int(1)->#1] default=none", "#6 loop",]
);
stmts!(
    stmt_loop_inside_a_switch,
    "void f(void){ switch (2) { while (1) { case 1: break; } } }",
    &["#0 break->#3", "#1 case Int(1)->#5", "#3 loop", "#5 switch int [Int(1)->#1] default=none",]
);

stmts!(stmt_goto_and_label, "void f(void){ goto l; l: ; }", &["#0 goto l", "#2 label l"]);

// 6.6.4.2 the controlling expression is promoted and each case is converted to that type
stmts!(
    stmt_case_values_take_the_promoted_control_type,
    "void f(char c){ switch (c) { case 1: ; case 'a': ; } }",
    &["#1 case Int(1)->#5", "#3 case Int(97)->#5", "#5 switch int [Int(1)->#1, Int(97)->#3] default=none",]
);

accept!(stmt_case_beyond_the_unpromoted_range, "void f(char c){ switch (c) { case 1: ; case 257: ; } }");
reject!(stmt_duplicate_case_after_conversion, "void f(void){ switch (1) { case 1: ; case 1L: ; } }");
accept!(stmt_case_inside_a_loop_inside_its_switch, "void f(void){ switch (1) { while (1) { case 1: ; } } }");
accept!(
    stmt_continue_inside_a_switch_inside_its_loop,
    "void f(void){ while (1) { switch (1) { case 1: continue; } } }"
);
accept!(stmt_default_inside_a_loop_inside_its_switch, "void f(void){ switch (1) { while (1) { default: ; } } }");
accept!(stmt_nested_switches_keep_their_own_cases, "void f(void){ switch (1) { switch (2) { case 1: ; } case 2: ; } }");
reject!(stmt_case_outside_any_switch, "void f(void){ while (1) { case 1: ; } }");
reject!(stmt_continue_outside_any_loop, "void f(void){ switch (1) { case 1: continue; } }");

// 6.6.4.2 The expression of each case label shall be an integral constant expression.
reject!(stmt_case_with_a_floating_value, "void f(void){ switch (1) { case 1.0: ; } }");
reject!(stmt_case_with_a_non_constant, "void f(void){ int x; switch (1) { case x: ; } }");

// 6.6.6.4 A return without a value in a non-void function is undefined behaviour, not a
// constraint violation, so it is diagnosed as a warning.
recover!(stmt_return_without_a_value, "int f(void){ return; }", [Diagnosis::ReturnWithoutValue], &[]);
reject!(stmt_return_with_a_value_from_void, "void f(void){ return 1; }");
accept!(stmt_return_without_a_value_from_void, "void f(void){ return; }");
