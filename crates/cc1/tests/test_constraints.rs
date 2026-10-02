#![cfg_attr(rustfmt, rustfmt_skip)]

invalid!(two_storage_specifiers_are_rejected, "static extern int x;", &["error: Multiple storage class declaration"]);
invalid!(a_function_declared_in_a_block_must_be_extern, "void f(void) { auto int g(void); }", &["error: Function in block not declared as extern"]);
invalid!(duplicate_qualifiers_are_rejected, "const const int x;", &["error: Duplicate type qualifiers"]);
invalid!(incompatible_type_specifiers_are_rejected, "unsigned double x;", &["error: Invalid type specifier or combination thereof"]);
invalid!(register_address_is_rejected, "int main(void) { register int x; int *p = &x; return 0; }", &["error: address of register variable requested"]);
invalid!(void_object_is_rejected, "void x;", &["error: variable has incomplete type 'void'", "error: tentative definition has type 'void' that is never completed"], gcc_accepts = true);
invalid!(incomplete_object_is_rejected, "struct S; struct S x;", &["error: tentative definition has type 'struct S' that is never completed"]);
invalid!(array_of_functions_is_rejected, "int f[2](void);", &["error: array has incomplete or function element type 'int(void)'"]);
invalid!(function_returning_array_is_rejected, "int f(void)[2];", &["error: function cannot return array type 'int[2]'"]);
invalid!(variadic_function_needs_named_parameter, "int f(...);", &["error: syntax error, unexpected ELLIPSIS, expecting ')'"]);
invalid!(register_array_subscript, "int f(void) { register int a[3]; a[0] = 1; return a[0]; }", &["error: address of register variable requested", "error: address of register variable requested"]);
invalid!(register_array_decay, "int f(void) { register int a[3]; int *p = a; return p != 0; }", &["error: address of register variable requested"]);
invalid!(qualified_function_type, "typedef int F(void); const F f;", &["error: ISO C forbids qualified function types"]);
valid!(register_scalar_use_is_accepted, "int main(void) { register int x = 3; x = x + 4; return x; }", 7, "");
valid!(register_array_sizeof_is_accepted, "int main(void) { register int a[3]; return sizeof(a) == 3 * sizeof(int) ? 0 : 1; }", 0, "");
valid!(non_register_array_decay_is_accepted, "int main(void) { int a[3]; int *p = a; a[0] = 5; return p[0]; }", 5, "");
valid!(function_typedef_declaration_is_accepted, "typedef int F(void); F f;\nint main(void) { return 0; }", 0, "");
