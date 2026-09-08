%{

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
int yylex();
void yyerror(const char *msg);
extern FILE *yyin;

%}

%union {
    int i;
}
%start stmt
%token<i> NUMBER

%%

stmt
    : 'i' stmt           {printf ("if\n");}
    | 'i' stmt 'l' stmt  {printf ("ifelse\n");}
    | 's'
    ;

%%

void yyerror(const char *msg) {
    fprintf(stderr, "%s\n", msg);
}

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    yyparse();
}
