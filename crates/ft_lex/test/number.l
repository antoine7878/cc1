    #define OPERATOR    1
    #define NUMBER      2
    #define DEFAULT     3

%%

[+/*%]        return OPERATOR;
[0-9]*        return NUMBER;
[ \t\n]*      return DEFAULT;

%%


int main(int argc, char *argv[]) {
    int tok;

    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    while ((tok = yylex()) != 0) {
        switch (tok) {
            case OPERATOR:
                printf("Operator: %s\n", yytext);
            break;
            case NUMBER:
                printf("Number: %s\n", yytext);
            break;
            case DEFAULT:
                printf("OTHER\n");
            break;
        }
        fflush(stdout);
    }
    return 0;
}

int yywrap(void) { return 1; }
