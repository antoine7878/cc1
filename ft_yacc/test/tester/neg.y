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
%start prog
%token<i> NUMBER
%left '\n'
%left '+' '-'
%left '*' '/'
%nonassoc '(' ')'

%type<i> expr rvalue line

%%

prog
    :
    | prog line
    ;

line
    : NUMBER expr '\n' { printf("line %d => %d\n", $1, $2); }
    ;

expr
    : rvalue { $$ = $1; }
    ;

rvalue
    : rvalue '+' rvalue
        {
            printf("[line %d] add\n", $<i>0);
            $$ = $1 + $3;
        }
    | rvalue '*' rvalue
        {
            printf("[line %d] mul\n", $<i>-2);
            $$ = $1 * $3;
        }
    | NUMBER { $$ = $1; }
    ;

%%

void yyerror(const char *msg) {
    fprintf(stderr, "%s\n", msg);
}

int main(int argc, char **argv) {

    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    yyparse();
}
