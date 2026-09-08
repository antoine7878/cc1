%%

"%%"   printf("COUCOU");

%%


int main(int argc, char *argv[]) {
	(void)argc;
	(void)argv;

	yyin = fopen("./test/empty.l", "r");
	while (yylex() != 0)
		;

	return 0;
}

int yywrap(void) {
    static int file_count = 0;

    if (file_count < 3){
        file_count++;
	    yyin = fopen("./test/empty.l", "r");
        return 0;
    }
    return 1;
}
