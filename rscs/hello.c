struct A {
	int x;
};

struct S {
	int a;
	int b[2];
	int c;
	struct A s;
};

struct S s = {1, 2, 3, 4, {1}};
