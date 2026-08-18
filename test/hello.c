// double extrn;

// struct tag {
// 	int member;
// };

// int fn(a, b, c)
// int a, b;
// int c;
// { return 1; }

#include <stdlib.h>

int const *const *const fn() {
	int volatile const *const volatile *volatile const a;
	return malloc(12);
}
