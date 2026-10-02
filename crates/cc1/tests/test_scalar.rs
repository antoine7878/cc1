#![cfg_attr(rustfmt, rustfmt_skip)]

valid!(scalar_layouts, "int main(void) { return !(sizeof(char) == 1 && sizeof(short) == 2 && sizeof(int) == 4 && sizeof(long) == 4 && sizeof(float) == 4 && sizeof(double) == 8 && sizeof(long double) == 12 && sizeof(void *) == 4); }", 0, "");
valid!(plain_char_is_signed, "int main(void) { char c = -1; return !(c < 0); }", 0, "");
