%{

#include <stdio.h>
#include <stdlib.h>

int yylex(void);
void yyerror(const char *msg);

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
    : rvalue
        {
            $$ = $1;
        }
    ;

rvalue
    : rvalue '+' rvalue
        {
            printf("[line %d] add\n", $<i>-1);
            $$ = $1 + $3;
        }
    | rvalue '-' rvalue
        {
            printf("[line %d] sub\n", $<i>-1);
            $$ = $1 - $3;
        }
    | rvalue '*' rvalue
        {
            printf("[line %d] mul\n", $<i>-1);
            $$ = $1 * $3;
        }
    | rvalue '/' rvalue
        {
            printf("[line %d] div\n", $<i>-1);
            $$ = $1 / $3;
        }
    | '(' rvalue ')' { $$ = $2; }
    | NUMBER { $$ = $1; }
    ;

%%

void yyerror(const char *msg)
{
    fprintf(stderr, "%s\n", msg);
}

int main(void)
{
    return yyparse();
}
