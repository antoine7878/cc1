uses!(use_plain_read, "int x; int f(void) { return x; }", &[("x", true), ("f", false)]);

uses!(use_sizeof_ident_not_used, "int x; int f(void) { return sizeof x; }", &[("x", false), ("f", false)]);

uses!(use_sizeof_expr_not_used, "int x; int f(void) { return sizeof(x + 1); }", &[("x", false), ("f", false)]);

uses!(
    use_sizeof_operand_precedence,
    "int x; int y; int f(void) { return sizeof x + y; }",
    &[("x", false), ("y", true), ("f", false)]
);

uses!(use_function_call, "int f(void); int g(void) { return f(); }", &[("f", true), ("g", false)]);

uses!(use_address_of, "int x; int *p; void f(void) { p = &x; }", &[("x", true), ("p", true), ("f", false)]);

uses!(
    use_member_access_marks_struct_var,
    "struct S { int m; }; struct S s; int f(void) { return s.m; }",
    &[("s", true), ("f", false)]
);

uses!(use_never_referenced, "int x;", &[("x", false)]);

uses!(
    use_shadowed_by_inner_scope,
    "int x; int f(void) { int x; return x; }",
    &[("x", false), ("f", false), ("x", true)]
);
