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


%%

prog
    : item
    | item prog
    ;

item
    : 'o' '\n'   {printf ("ok\n");}
    | 'z' '\n'   {YYERROR;}
    | 'q'        {yyclearin; YYACCEPT;}
    | 'x'        {YYABORT;}
    | error '\n' {yyerrok; printf ("recovered %d\n", YYRECOVERING());}
    ;

%%

void yyerror(const char *msg) {
    fprintf(stderr, "%s\n", msg);
}

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    int ret = yyparse();
    printf("ret=%d\n", ret);
}
