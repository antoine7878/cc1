
    #define OPERATOR    1
    #define NUMBER      2
    #define DEFAULT     3


%%

salut   printf("1"); REJECT;
salu    printf("2"); REJECT;
sal     printf("3"); REJECT;
sa      printf("4"); REJECT;
s       printf("5"); REJECT;

%%


int main(int argc, char *argv[]) {
	(void)argc;
	(void)argv;
    int i = 0;

	yyin = fopen(argv[1], "r");
	while (yylex() != 0)
        ;
	return 0;
}

int yywrap(void) { return 1; }

