#include <stdio.h>

struct a {
	char a1;  /* 1 1 */
	short a3; /* 4 8 */
	char a2;  /* 1 2 */
			  /* 8 */
};

struct b {
	int b2;	 /* 4 4 */
	char b1; /* 1 5 */
	char b3; /* 1 6 */
	/* 8 */
};

int x;

enum e { a = sizeof(x), b = sizeof(struct b) };

int main(void) {
	printf("a: %i\n", a);
	printf("b: %i\n", b);
	return 0;
}
