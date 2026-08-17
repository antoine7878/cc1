struct test {
	int a;
	double b;
};

int main(int argc, char **argv) {
	struct test s;
	s.b;
	(&s)->a;
	return 0;
}
