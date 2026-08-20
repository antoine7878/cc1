struct strct {
	int membr;
};

int fn(void) {
	const int val = 4;
	goto end;
end:
	return 1;
}
