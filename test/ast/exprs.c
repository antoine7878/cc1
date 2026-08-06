int f(int a, int b) {
    int r;
    r = a + b * 2;
    r += a;
    r <<= 1;
    r = -a + +b;
    r = !a && b || c;
    r = a < b ? a : b;
    r = (a > b) & (c < d);
    r = a ^ b | c & d;
    r = *p;
    r = *(p + 1);
    r = p->x;
    r = s.x;
    r = arr[i];
    r = f(1, 2);
    r = f();
    r = sizeof(int);
    r = sizeof(a);
    r = (long)a;
    r = a++, b--;
    r = ++a + --b;
    r = (int *)p;
    return r;
}
