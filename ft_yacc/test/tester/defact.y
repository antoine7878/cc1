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

%type<i> a b c

%%

s
    : a {printf ("= %d\n", $1);}
    ;

a
    : b
    ;

b
    : c
    ;

c
    : NUMBER
    ;

%%

void yyerror(const char *msg) {
    fprintf(stderr, "%s\n", msg);
}

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    yyparse();
}
