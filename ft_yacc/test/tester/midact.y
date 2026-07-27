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
%start s
%token<i> NUMBER

%%

s
    : 'x' NUMBER {printf ("mid %d\n", $2); $<i>$ = $2 + 1;} 'y' {printf ("end %d %d\n", $2, $<i>3);}
    ;

%%

void yyerror(const char *msg) {
    fprintf(stderr, "%s\n", msg);
}

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    yyparse();
}
