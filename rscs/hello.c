
int main(void) {
	int a = 1;
	a = 42;
	{
		int b = a;
		return b;
	}
}
