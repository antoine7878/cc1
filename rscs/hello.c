int main(void) {
	int i;
	int s;
	s = 0;
	for (i = 0; i < 10; i++) {
		if (i % 2)
			continue;
		s += i;
	}
	return s + 22;
}
