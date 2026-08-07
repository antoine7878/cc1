int f(int a, int b, int *p) {
    int r;
    r = a / b;
    r = a % b;
    r = a << b;
    r = a >> b;
    r = a >= b;
    r = a <= b;
    r = a == b;
    r = a != b;
    r = ~a;
    p = &a;
    r *= 2;
    r /= 2;
    r %= 2;
    r -= 2;
    r >>= 1;
    r &= a;
    r ^= a;
    r |= a;
    return r;
}