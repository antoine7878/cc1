struct a {
	int a;
};

void fn(void) {
	struct a *st;
	struct a *pst;
	int i;

	(st = pst)->a = 1;
}
