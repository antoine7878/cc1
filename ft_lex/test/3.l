int main(int argc, char *argv[]) {
	(void)argc;
	(void)argv;

	yyin = fopen(argv[1], "r");
	while (yylex() != 0)
		;

	return 0;
}

int yywrap(void) { return 2; }
