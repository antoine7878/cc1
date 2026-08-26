
struct a {
	int : 1;
};

enum s { OUI = sizeof(struct a) };

int main(void) {
	return 0;
}
