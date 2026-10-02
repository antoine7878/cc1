#![cfg_attr(rustfmt, rustfmt_skip)]

valid!(f80_keeps_the_bits_a_double_would_lose, "int main(void) { return !((9007199254740992.0L + 1.0L) - 9007199254740992.0L == 1.0L); }", 0, "");
valid!(f80_arithmetic_rounds_once_to_nearest_even, "int main(void) { return !((18446744073709551616.0L + 1.0L) == 18446744073709551616.0L); }", 0, "");
valid!(f80_parses_decimal_constants, "int main(void) { return !(0.125L * 8.0L == 1.0L && 1e3L == 1000.0L); }", 0, "");
valid!(f80_converts_across_the_other_representations, "int main(void) { return !((int)3.9L == 3 && (double)1.5L == 1.5); }", 0, "");
valid!(f80_handles_zero_operands, "int main(void) { return !(0.0L + 2.0L == 2.0L && 2.0L * 0.0L == 0.0L); }", 0, "");
valid!(long_double_literal_precision, "int main(void) { volatile double d = 0.1; long double widened = d; return !(0.1L != widened); }", 0, "");
valid!(long_double_runtime_arithmetic, "int main(void) { volatile long double a = 9007199254740992.0L; long double b = a + 1.0L; return !(b - a == 1.0L); }", 0, "");
