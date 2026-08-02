
int main(void) {
	void *f(void);
	int (*k)[];
	void *(*a)(void) = f;
	return 0;
}

void *f(void) {
	return 0;
}
