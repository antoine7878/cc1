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
