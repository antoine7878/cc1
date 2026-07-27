extern int yylex();

int main() {
	while (yylex() != 0)
		;
	return 0;
}
