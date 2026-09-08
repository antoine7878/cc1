%{
#include <stdio.h>
#include <stdlib.h>
int yylex();
void yyerror(const char *msg);
int yydebug = 1;
%}

%start prog
%token NUMBER
%left '\n'
%left '+' '-'
%left '*' '/'
%nonassoc '(' ')'
%type expr rvalue

%%

prog
    :
    | expr
    | expr '\n' prog
    | error '\n' prog
    ;

expr
    : rvalue {printf ("= %d\n", $$);}
    ;

rvalue
    : rvalue '+' rvalue {$$ = $1 + $3;}
    | rvalue '-' rvalue {$$ = $1 - $3;}
    | rvalue '*' rvalue {$$ = $1 * $3;}
    | rvalue '/' rvalue {$$ = $1 / $3;}
    | '(' rvalue ')' {$$ = $2;}
    | NUMBER {$$ = $1;}
    ;

%%

void yyerror(const char *msg) {
    fprintf(stderr, "%s\n", msg);
}

int main() {
    yyparse();
}
