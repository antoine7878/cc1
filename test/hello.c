// double extrn;

// struct tag {
// 	int member;
// };

// int fn(a, b, c)
// int a, b;
// int c;
// { return 1; }

#include <stdlib.h>

int fn(int (*f)(int[]), int a[12]) {
	return f(a);
}

int const *const *const fn2() {
	int volatile const *const volatile *volatile const a;
	return malloc(12);
}
