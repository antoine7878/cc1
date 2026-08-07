typedef int arr_t[10];
typedef int unary_t(int);
typedef int *p_int;

int g(int n) {
    int (*pf)(int);
    int * const volatile cv;
    int * const hp;
    int s = (const int)n;
    int t = sizeof(const char *);
    int u = sizeof(int [4]);
    return t;
}
