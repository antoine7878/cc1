#![cfg_attr(rustfmt, rustfmt_skip)]

valid!(long_double_dominates, "int main(void) { return !(sizeof(1 + 2.0L) == sizeof(long double)); }", 0, "");
valid!(integer_and_double_meet_at_double, "int main(void) { return !(sizeof(1 + 2.0) == sizeof(double)); }", 0, "");
valid!(integer_and_float_meet_at_float, "int main(void) { return !(sizeof(1 + 2.0f) == sizeof(float)); }", 0, "");
valid!(float_and_double_meet_at_double, "int main(void) { return !(sizeof(1.0f + 2.0) == sizeof(double)); }", 0, "");
valid!(narrow_operands_of_the_same_type_are_still_promoted, "int main(void) { return !(sizeof((char)1 + (char)1) == sizeof(int)); }", 0, "");
valid!(promotion_alone_reaches_the_common_type, "int main(void) { return !(sizeof((char)1 + (short)1) == sizeof(int)); }", 0, "");
valid!(unsigned_int_and_long_meet_at_unsigned_long, "int main(void) { return !((-1L < 1u) == 0); }", 0, "");
valid!(float_conversion_rounds, "int main(void) { return !((float)16777217.0 == 16777216.0f); }", 0, "");
valid!(unsigned_conversion_wraps, "int main(void) { return !((unsigned int)-1 == 4294967295u); }", 0, "");
valid!(char_conversion_narrows, "int main(void) { return !((char)300 == 44); }", 0, "");
valid!(integer_conversion_truncates, "int main(void) { return !((int)3.9 == 3 && (int)-3.9 == -3); }", 0, "");
valid!(array_decay_pointer_arithmetic, "int main(void) { return !(sizeof(\"abc\" + 1) == sizeof(char *)); }", 0, "");
