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
    double d;
}
%start prog
%token<i> INT
%token<d> REAL
%left '\n'
%left '+'

%type<d> expr

%%

prog
    : expr {printf ("= %g\n", $1);}
    | expr '\n' prog {printf ("= %g\n", $1);}
    |
    ;

expr
    : expr '+' expr {$$ = $1 + $3;}
    | REAL
    | INT {$$ = $<i>1;}
    ;

%%

void yyerror(const char *msg) {
    fprintf(stderr, "%s\n", msg);
}

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    yyparse();
}
