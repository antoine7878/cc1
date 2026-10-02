#![cfg_attr(rustfmt, rustfmt_skip)]

valid!(literal_char_utf8_narrow_is_multichar, "int main(void) { return !(('é' == (50089)) && (sizeof('é') == sizeof(int)) && ((('é') - ('é') - 1 < 0) == 1)); }", 0, "");
valid!(literal_wide_char_utf8_is_code_point, "int main(void) { return !((L'é' == (233)) && (sizeof(L'é') == sizeof(int)) && (((L'é') - (L'é') - 1 < 0) == 1)); }", 0, "");
valid!(literal_wide_char_utf8_euro_sign, "int main(void) { return !((L'€' == (8364)) && (sizeof(L'€') == sizeof(int)) && (((L'€') - (L'€') - 1 < 0) == 1)); }", 0, "");
valid!(string_utf8_char_is_its_bytes, "int main(void) { return !(((unsigned char)(\"é\")[0] == 195) && ((unsigned char)(\"é\")[1] == 169) && ((unsigned char)(\"é\")[2] == 0)); }", 0, "");
valid!(string_wide_utf8_char_is_one_unit, "int main(void) { return !(((L\"€\")[0] == 8364) && ((L\"€\")[1] == 0)); }", 0, "");
valid!(utf8_wide_string_values, "int main(void) { return sizeof L\"é€\" == 12 && L\"é€\"[0] == 233 && L\"é€\"[1] == 8364 && L\"é€\"[2] == 0; }", 1, "");
valid!(utf8_wide_char_value, "int main(void) { return L'é' == 233 && L'€' == 8364; }", 1, "");
valid!(utf8_narrow_string_length, "int main(void) { return sizeof \"é€\" == 6 && (unsigned char)\"é\"[0] == 195 && (unsigned char)\"é\"[1] == 169; }", 1, "");
valid!(utf8_narrow_char_is_multichar, "int main(void) { return 'é' == 50089; }", 1, "");
valid!(mixed_plain_and_escaped_character_units, "int main(void) { return 'a\\377' == 25087 && '\\377' == -1 && L'\\101' == 65; }", 1, "");
valid!(raw_byte_in_string_literal_is_kept, b"char s[] = \"\xe9t\"; int main(void) { return !((unsigned char)s[0] == 233 && s[1] == 116 && sizeof s == 3); }", 0, "", preprocessed = true);
valid!(raw_byte_in_char_constant_is_kept, b"int main(void) { return '\xe9'; }", 233, "", preprocessed = true);
invalid!(raw_byte_in_wide_string_is_an_error, b"int main(void) { return L\"\xe9\"[0]; }", &["error: converting to execution character set: Invalid argument"], preprocessed = true);
invalid!(raw_byte_in_wide_char_is_an_error, b"int main(void) { return L'\xe9'; }", &["error: converting to execution character set: Invalid argument"], preprocessed = true);
invalid!(raw_byte_outside_literals_is_reported_in_octal, b"int main(void) { return 1 \xe9; }", &["error: stray '\\351' in program"], preprocessed = true);
valid!(utf8_wide_string_decodes_code_points, "int main(void) { return !(sizeof L\"é€\" == 12 && L\"é€\"[0] == 233 && L\"é€\"[1] == 8364); }", 0, "");
valid!(utf8_narrow_string_keeps_its_bytes, "int main(void) { return !(sizeof \"é€\" == 6 && (unsigned char)\"é€\"[0] == 195 && (unsigned char)\"é€\"[1] == 169); }", 0, "");
