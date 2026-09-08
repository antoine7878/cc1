%%

salut   printf("1"); REJECT;
salu    printf("2"); REJECT;
sal     printf("3"); REJECT;
sa      printf("4"); REJECT;
s       printf("5"); REJECT

%%

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    while (yylex()) ;
}
