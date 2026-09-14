struct S {
	int a;
};
struct S x = {1};
int a;
int arr[2];
short b = 1;

int *h(void) {
	int arr[2];
	return arr;
}

int g(void) {
	return a;
}

struct S f(void) {
	return x;
}
