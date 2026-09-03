struct S;
struct S *p;

struct S *f(void) {
	return &*p;
}
