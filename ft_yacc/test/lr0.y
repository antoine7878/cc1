%{
#include <stdio.h>

int yylex(void);
void yyerror(const char *s);
%}

%union {
    int i;
}

%type<i> e
%token<i> id
%left '+'
%left '*'

%%

s
    : e {printf("res = %i\n", $1);}
    ;

e
    : e '+' e {$$ = $1 + $3;}
    | e '*' e {$$ = $1 * $3;}
    | id {$$ = $1;}
    ;

%%

void yyerror(const char *s) {
    fprintf(stderr, "Error: %s\n", s);
}

int main(void) {
    yyparse();
    return 0;
}
