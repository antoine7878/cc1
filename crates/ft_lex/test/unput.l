%%

salut  printf("MATCH"); input(); input(); unput('1'); unput('2');

%%

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    while (yylex()) ;
}
