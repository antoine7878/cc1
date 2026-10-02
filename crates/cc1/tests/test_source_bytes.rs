use crate::common::compile_raw as compile_bytes;

test_case!(raw_byte_in_string_literal_is_kept, {
    let run = compile_bytes("raw_string", b"char s[] = \"\xe9t\xe9\";\n");
    assert_eq!(run.status, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("[4 x i8]"), "stdout: {}", run.stdout);
    assert!(run.stdout.contains("[i8 233, i8 116, i8 233, i8 0]"), "stdout: {}", run.stdout);
});

test_case!(raw_byte_in_char_constant_is_kept, {
    let run = compile_bytes("raw_char", b"int main(void) { return '\xe9'; }\n");
    assert_eq!(run.status, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("ret i32 -23"), "stdout: {}", run.stdout);
});

test_case!(raw_byte_in_wide_string_is_an_error, {
    let run = compile_bytes("raw_wide_string", b"int main(void) { return L\"\xe9\"[0]; }\n");
    assert_eq!(run.status, 1, "stderr: {}", run.stderr);
    assert!(run.stderr.contains("converting to execution character set"), "stderr: {}", run.stderr);
});

test_case!(raw_byte_in_wide_char_is_an_error, {
    let run = compile_bytes("raw_wide_char", b"int main(void) { return L'\xe9'; }\n");
    assert_eq!(run.status, 1, "stderr: {}", run.stderr);
    assert!(run.stderr.contains("converting to execution character set"), "stderr: {}", run.stderr);
});

test_case!(raw_byte_outside_literals_is_reported_in_octal, {
    let run = compile_bytes("raw_stray", b"int main(void) { return 1 \xe9; }\n");
    assert_eq!(run.status, 1, "stderr: {}", run.stderr);
    assert!(run.stderr.contains("stray '\\351' in program"), "stderr: {}", run.stderr);
});

test_case!(utf8_wide_string_decodes_code_points, {
    let run = compile_bytes("utf8_wide", "int main(void) { return sizeof L\"é€\"; }\n".as_bytes());
    assert_eq!(run.status, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("ret i32 12"), "stdout: {}", run.stdout);
});

test_case!(utf8_narrow_string_keeps_its_bytes, {
    let run = compile_bytes("utf8_narrow", "int main(void) { return sizeof \"é€\"; }\n".as_bytes());
    assert_eq!(run.status, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("ret i32 6"), "stdout: {}", run.stdout);
});

constant!(literal_char_utf8_narrow_is_multichar, "'\u{e9}'", "Int(50089)");
constant!(literal_wide_char_utf8_is_code_point, "L'\u{e9}'", "Int(233)");
constant!(literal_wide_char_utf8_euro_sign, "L'\u{20ac}'", "Int(8364)");
constant!(literal_char_mapped_invalid_byte_is_one_byte, "'\u{10ffe9}'", "Int(-23)");
constant!(literal_char_mapped_invalid_byte_after_ascii, "'a\u{10ffe9}'", "Int(25065)");
escape_invalid!(
    literal_wide_char_mapped_invalid_byte_is_an_error,
    "L'\u{10ffe9}'",
    "Int(1114089)",
    cc1::semantic::Diagnostic::InvalidWideCharacter
);

literal!(string_mapped_invalid_byte_is_one_unit, "char *s = \"\u{10ffe9}\";", "\\xe9");
literal!(string_utf8_char_is_its_bytes, "char *s = \"\u{e9}\";", "\\xc3\\xa9");
literal!(string_wide_utf8_char_is_one_unit, "int *s = L\"\u{20ac}\";", "L\\x20ac");

exits!(
    utf8_wide_string_values,
    "int main(void) { return sizeof L\"é€\" == 12 && L\"é€\"[0] == 233 && L\"é€\"[1] == 8364 && L\"é€\"[2] == 0; }",
    1
);
exits!(utf8_wide_char_value, "int main(void) { return L'é' == 233 && L'€' == 8364; }", 1);
exits!(
    utf8_narrow_string_length,
    "int main(void) { return sizeof \"é€\" == 6 && (unsigned char)\"é\"[0] == 195 && (unsigned char)\"é\"[1] == 169; }",
    1
);
exits!(warns utf8_narrow_char_is_multichar, "int main(void) { return 'é' == 50089; }", 1);
exits!(
    warns mixed_plain_and_escaped_character_units,
    "int main(void) { return 'a\\377' == 25087 && '\\377' == -1 && L'\\101' == 65; }",
    1
);
