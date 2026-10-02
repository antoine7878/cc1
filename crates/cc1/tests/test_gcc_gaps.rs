#![cfg_attr(rustfmt, rustfmt_skip)]

accept!(gap_extern_void_object, "extern void v;");

emits!(not gap_string_concat_emits_no_intermediate_literals, "char *p = \"ab\" \"cd\" \"ef\";", "@.str.1");

exits!(gap_extern_void_object_emits_valid_ir, "extern void v; int main(void) { return 0; }", 0);
exits!(gap_extern_const_void_object_emits_valid_ir, "extern const void cv; int main(void) { return 0; }", 0);
exits!(gap_extern_void_object_expression_statement, "extern void v; int main(void) { v; return 0; }", 0);
exits!(gap_incomplete_struct_parameter_declaration, "struct S; void f(struct S); int main(void) { return 0; }", 0);
exits!(gap_incomplete_union_parameter_declaration, "union U; void f(union U); int main(void) { return 0; }", 0);
exits!(gap_incomplete_struct_parameter_function_pointer, "struct S; void f(struct S); void (*fp)(struct S) = f; int main(void) { return fp != 0; }", 1);
exits!(gap_incomplete_struct_parameter_after_complete_param, "struct S; int f(int, struct S); int main(void) { return 0; }", 0);
emits!(not gap_incomplete_struct_parameter_no_byval, "struct S; void f(struct S); int main(void) { return 0; }", "byval");
exits!(gap_static_init_address_to_int, "int g; int i = (int)&g; int main(void) { return i != 0 ? 0 : 1; }", 0);
exits!(gap_static_init_address_to_unsigned, "int g; unsigned u = (unsigned)&g; int main(void) { return u != 0 ? 0 : 1; }", 0);
exits!(gap_static_init_address_to_long, "int g; long l = (long)&g; int main(void) { return l != 0 ? 0 : 1; }", 0);
reject!(gap_static_init_address_to_short, "int g; short s = (short)&g;");
reject!(gap_static_init_address_to_char_in_struct, "int g; struct { char c; } x = { (char)&g };");
reject!(gap_static_init_address_to_char_in_array, "int g; int a[2] = { (int)&g, (char)&g };");
reject!(gap_static_init_address_via_int_to_char, "int g; char c = (char)(int)&g;");
reject!(gap_static_init_address_via_char_to_int, "int g; int i = (int)(char)&g;");
reject!(gap_auto_init_address_to_char_in_struct, "int g; int main(void) { struct { char c; } x = { (char)&g }; return 0; }");
reject!(gap_auto_init_address_to_short_in_struct, "int g; int main(void) { struct { short c; } x = { (short)&g }; return 0; }");
accept!(gap_auto_init_address_to_int_in_array, "int g; int main(void) { int a[2] = { (int)&g, 1 }; return 0; }");
reject!(gap_static_init_int_cast_address_to_short, "int g; short s = (int)&g;");
reject!(gap_static_init_int_cast_address_to_char_in_struct, "int g; struct { char c; } x = { (int)&g };");
reject!(gap_static_init_long_cast_address_to_unsigned_char, "int g; unsigned char uc = (unsigned long)&g;");
accept!(gap_static_init_address_to_int_in_struct, "int g; struct { int c; } x = { (int)&g };");

exits!(gap_arrow_on_array, "struct S { int a; } arr[2]; int main(void) { arr->a = 42; return arr[0].a; }", 42);
exits!(gap_incomplete_extern_object, "struct S; extern struct S s; int main(void) { return 0; }", 0);
exits!(gap_incomplete_union_extern_object, "union U; extern union U u; int main(void) { return 0; }", 0);
exits!(gap_incomplete_return_declaration, "struct S; struct S f(void); int main(void) { return 0; }", 0);
exits!(gap_paren_comma_first_argument, "int f(int a, int b) { return a * 10 + b; } int main(void) { return f((1, 2), 3); }", 23);
exits!(gap_paren_comma_single_argument_unprototyped, "int g(); int main(void) { return g((1, 2)); } int g(a) int a; { return a; }", 2);
exits!(gap_long_divided_by_unsigned_bit_field, "struct B { unsigned x : 4; } b; int main(void) { long l = -6; b.x = 2; return l / b.x == -3; }", 1);
exits!(gap_long_compared_to_unsigned_bit_field, "struct B { unsigned x : 21; } b; int main(void) { long l = -5; b.x = 3; return l <= b.x; }", 1);
exits!(gap_conditional_long_and_unsigned_bit_field, "struct B { unsigned x : 4; } b; int main(void) { long l = -1; b.x = 1; return (0 ? b.x : l) < 0; }", 1);
exits!(gap_pragma_pack, "#pragma pack(1)\nstruct S { char c; int i; };\n#pragma pack()\nint main(void) { return sizeof(struct S); }", 5);
exits!(gap_enum_constant_in_parameter_list, "int f(enum E { A, B } x) { return x + B; } int main(void) { return f(0); }", 1);
exits!(warns gap_deref_void_pointer_discarded, "int main(void) { int x = 1; void *p = &x; *p; (void)*p; return 0; }", 0);
exits!(gap_typedef_void_parameter_list, "typedef void V; int f(V); int f(V) { return 3; } int main(void) { return f(); }", 3);

reject!(gap_nested_tag_redefinition, "struct S { struct S { int a; } x; };");
reject!(gap_paren_comma_whole_argument_list, "int printf(const char *, ...); int main(void) { printf((\"%d\", 1)); return 0; }");
reject!(gap_hex_pp_number_with_sign, "int x = 0x1E-1;");

reject!(gap_cpp_comment, "int x = 4 // 2\n;");
reject!(gap_digraphs, "int a<:2:>;");
reject!(gap_newline_in_string_literal, "char *s = \"a\nb\";");
reject!(gap_sizeof_type_name_then_operand, "int f(void) { return sizeof (int) 1; }");
reject!(gap_identifier_list_in_declaration, "int f(a, b);");
reject!(gap_qualified_function_type, "typedef int F(void); const F f;");
reject!(gap_static_incomplete_array, "static int a[];");
reject!(gap_main_returning_void, "void main(void) { }");
reject!(gap_main_char_parameter, "int main(char c) { return 0; }");
reject!(gap_main_static, "static int main(void) { return 0; }");
reject!(gap_subscript_non_lvalue_array, "struct S { int a[3]; }; struct S f(void); int g(void) { return f().a[1]; }");
reject!(gap_decay_non_lvalue_array, "struct S { int a[3]; } x, y; int g(void) { int *p = (x = y).a; return p != 0; }");
reject!(gap_register_array_subscript, "int f(void) { register int a[3]; a[0] = 1; return a[0]; }");
reject!(gap_register_array_decay, "int f(void) { register int a[3]; int *p = a; return p != 0; }");
reject!(gap_register_struct_member_address, "struct S { int x; }; int f(void) { register struct S s; int *p = &s.x; return p != 0; }");
reject!(gap_function_to_object_pointer_cast, "int f(void); void g(void) { void *v = (void *)f; }");
reject!(gap_object_to_function_pointer_cast, "void g(void) { int x; int (*p)(void) = (int (*)(void))&x; }");
reject!(gap_conditional_merges_qualifiers_into_void_pointer, "void g(int c) { int x = 1; const int *p = &x; void *q = &x; void *r = c ? p : q; }");
reject!(gap_deref_incomplete_struct, "struct S; struct S *p; void f(void) { *p; }");
reject!(gap_call_qualified_void_function, "void volatile f(); void g(void) { f(); }");
reject!(gap_static_init_pointer_to_char, "int g; char c = (char)&g;");
reject!(gap_constant_shift_too_wide, "enum { A = 1 << 32 };");
reject!(gap_constant_shift_negative, "enum { B = 1 << -1 };");
reject!(gap_array_from_parenthesized_string, "char s[] = (\"abc\");");
reject!(gap_struct_too_large, "struct S { char a[0x40000000]; char b[0x40000000]; };");
