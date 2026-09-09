struct S {
	long double l;
};

long double fn(void) {
	struct S a = {1.};
	return 42;
}
