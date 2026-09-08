%%

a*   printf("1"); REJECT;
a*   printf("2"); REJECT;
a*   printf("3");

%%

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    while (yylex()) ;
}
