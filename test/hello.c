
struct I {
	char a;
	int b;
};

struct S {
	char x;
	struct I i;
};

enum { PROBE = sizeof(struct S) };

// #include <stdio.h>
//
// int main(void) {
// 	printf("size: %zu\n", sizeof(struct S));
// 	return 0;
// 	;
// }
