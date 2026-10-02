#![cfg_attr(rustfmt, rustfmt_skip)]

valid!(stdout_text, "int puts(const char *); int main(void) { puts(\"hello\"); return 7; }", 7, "hello\n", max_size = 32768);
valid!(stdout_binary, "int putchar(int); int main(void) { putchar(0); putchar(255); putchar(65); return 0; }", 0, b"\0\xffA");
valid!(stdout_more_than_pipe_buffer, "int putchar(int); int main(void) { int i; for (i = 0; i < 70000; i++) putchar(120); return 0; }", 0, b"x".repeat(70000));
valid!(multiple_translation_units, "int shared; int increment(void); int main(void) { shared = 40; return increment(); }", 42, "", files = &["extern int shared; int increment(void) { return shared + 2; }"], max_size = 32768);
valid!(gcc_abi_helper, "int external(int); int main(void) { return external(40); }", 42, "", helper = "int external(int x) { return x + 2; }");
valid!(preprocessor_defines, "#define ANSWER 42\nint main(void) { return ANSWER; }", 42, "");
invalid!(invalid_source_stops_before_link, "int main(void) { return missing; }", &["error: Use of undeclared identifier 'missing'"]);
invalid!(semantic_passes_do_not_stop_at_the_first_error, "int main(void) { return x + y; }", &["error: Use of undeclared identifier 'x'", "error: Use of undeclared identifier 'y'"]);
invalid!(syntax_error_stops_before_semantic_and_codegen, "int main(void) { return }", &["error: syntax error, unexpected '}', expecting ';'"]);
valid!(large_zero_tail_size, "char data[100000] = { 7 }; int main(void) { return data[0] + data[99999]; }", 7, "", max_size = 120000, max_ir_size = 2000);
invalid!(diagnostic_words_in_source_are_not_diagnostics, "int main(void) { char *s = \"error: not a diagnostic\"; return missing; }", &["error: Use of undeclared identifier 'missing'"]);
