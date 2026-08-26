#include <stdio.h>

struct a {
	int a;
	int : 1;
	int n;
};

enum s { OUI = sizeof(struct a) };

int main(void) {
	printf("%i\n", OUI);
	return 0;
}
