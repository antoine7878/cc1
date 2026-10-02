#![cfg_attr(rustfmt, rustfmt_skip)]

accept!(string_escape_sizeof_newline, "char t[sizeof \"x\\n\" == 3 ? 1 : -1];");
accept!(string_escape_sizeof_nul, "char t[sizeof \"\\0\" == 2 ? 1 : -1];");
accept!(string_escape_sizeof_backslash, "char t[sizeof \"\\\\\" == 2 ? 1 : -1];");
accept!(string_escape_sizeof_quote, "char t[sizeof \"\\\"\" == 2 ? 1 : -1];");
accept!(string_escape_sizeof_octal, "char t[sizeof \"\\101\" == 2 ? 1 : -1];");
accept!(string_escape_sizeof_hex, "char t[sizeof \"\\x41\" == 2 ? 1 : -1];");
accept!(string_escape_question_mark, "char t[sizeof \"\\?\" == 2 ? 1 : -1];");
accept!(string_concat_after_hex_escape, "char t[sizeof \"\\x1\" \"2\" == 3 ? 1 : -1];");
accept!(string_concat_after_octal_escape, "char t[sizeof \"\\1\" \"2\" == 3 ? 1 : -1];");
accept!(string_wide_sizeof, "char t[sizeof L\"ab\" == 12 ? 1 : -1];");
accept!(string_wide_sizeof_escape, "char t[sizeof L\"\\n\" == 8 ? 1 : -1];");
accept!(string_init_exact_fit_with_escape, "char s[2] = \"a\\n\";");
accept!(string_init_exact_fit_without_nul, "char s[3] = \"abc\";");
accept!(string_init_infers_decoded_length, "char s[] = \"a\\n\"; char t[sizeof s == 3 ? 1 : -1];");

inits!(string_init_stores_decoded_units, "char s[4] = \"a\\nb\";", &[("s", "\"a\\x0ab\"")]);
inits!(string_init_offset_into_a_literal, "char *p = \"a\\nb\" + 1;", &[("p", "&\"a\\x0ab\"+1")]);

literal!(string_escape_decodes_newline, "char *s = \"x\\n\";", "x\\x0a");
literal!(string_escape_decodes_octal, "char *s = \"\\101\";", "A");
literal!(string_escape_decodes_hex, "char *s = \"\\x41\";", "A");
literal!(string_escape_decodes_nul, "char *s = \"a\\0b\";", "a\\x00b");
literal!(string_escape_decodes_backslash, "char *s = \"\\\\\";", "\\x5c");
literal!(string_concat_keeps_escapes_apart, "char *s = \"\\x1\" \"2\";", "\\x012");

pool!(string_pool_dedups_equal_literals, "char *a = \"x\"; char *b = \"x\";", &["\"x\""]);
pool!(string_pool_separates_wide_from_narrow, "char *a = \"ab\"; int *b = L\"ab\";", &["\"ab\"", "L\"ab\""]);
pool!(string_pool_keeps_concatenated_pieces, "char *s = \"x\" \"y\";", &["\"x\"", "\"y\"", "\"xy\""]);

reject!(string_escape_hex_out_of_range, "char *s = \"\\x1ff\";");
reject!(string_escape_octal_out_of_range, "char *s = \"\\777\";");
reject!(string_escape_hex_no_digits, "char *s = \"\\x\";");
reject!(string_escape_unknown, "char *s = \"a\\qb\";");
reject!(string_init_too_long, "char s[1] = \"ab\";");

literal!(string_concat_three_keeps_escapes_apart, "char *s = \"\\x1\" \"2\" \"\\x3\";", "\\x012\\x03");
literal!(string_concat_four_octal_escapes, "char *s = \"\\1\" \"2\" \"\\0\" \"3\";", "\\x012\\x003");
literal!(string_concat_three_wide, "int f(void) { return (L\"a\" L\"b\" L\"c\")[0]; }", "Labc");
literal!(string_concat_three_mixed_wide_first, "int f(void) { return (L\"a\" \"b\" \"c\")[0]; }", "Labc");

exits!(string_concat_three_escapes_value, "int main(void) { return (\"\\x1\" \"2\" \"\\x3\")[1]; }", 50);
exits!(string_concat_three_escapes_size, "int main(void) { return sizeof(\"\\x1\" \"2\" \"\\x3\"); }", 4);
exits!(string_concat_three_wide_value, "int main(void) { return (L\"a\" L\"b\" L\"c\")[2]; }", 99);
exits!(string_concat_four_pieces_last_byte, "int main(void) { char *s = \"a\" \"b\" \"c\" \"d\"; return s[3]; }", 100);

emits!(not string_concat_emits_no_intermediate_two_pieces, "char *p = \"ab\" \"cd\";", "@.str.1");
emits!(not string_concat_emits_no_unused_piece, "char *p = \"ab\" \"cd\" \"ef\";", "c\"ab\\00\"");
