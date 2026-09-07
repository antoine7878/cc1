#include <stdio.h>

enum E { A = (int)3.3L };

int main(void) {
	printf("coucou:%i\n", A);
	return 0;
}
