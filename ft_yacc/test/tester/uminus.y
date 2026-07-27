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
%right UMINUS

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
    | rvalue '*' rvalue {$$ = $1 * $3;}
    | rvalue '/' rvalue {$$ = $1 / $3;}
    | '-' rvalue %prec UMINUS {$$ = -$2;}
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
