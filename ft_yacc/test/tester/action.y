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
    : NUMBER {
        /* } in block comment */
        printf ("\"%d\"\n", $1); // } in line comment
        if ($1 == '{') { printf ("brace\n"); }
    }
    ;

%%

void yyerror(const char *msg) {
    fprintf(stderr, "%s\n", msg);
}

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    yyparse();
}
