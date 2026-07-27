%{

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
int yylex();
void yyerror(const char *msg);
extern FILE *yyin;

int ipow(int base, int exp) {
    int ret = 1;
    while (exp-- > 0)
        ret *= base;
    return ret;
}

%}

%union {
    int i;
}
%start prog
%token<i> NUMBER
%left '\n'
%left '+' '-'
%right '^'

%type<i> expr rvalue

%%

prog
    : expr
    | expr '\n' prog
    |
    ;

expr
    : rvalue {printf ("= %d\n", $1);}
    ;

rvalue
    : rvalue '+' rvalue {$$ = $1 + $3;}
    | rvalue '-' rvalue {$$ = $1 - $3;}
    | rvalue '^' rvalue {$$ = ipow($1, $3);}
    | '(' rvalue ')' {$$ = $2;}
    | NUMBER
    ;

%%

void yyerror(const char *msg) {
    fprintf(stderr, "%s\n", msg);
}

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    yyparse();
}
