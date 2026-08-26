#include <stdio.h>

struct a {
	int a : 1;
	: 0;
	int : 3;
	int b : 1;
};

int main(void) {
	printf("%i\n", sizeof(struct a));
	return 0;
}
