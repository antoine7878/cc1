%%

^salut  printf("SALUT");

%%

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    while (yylex()) ;
}
