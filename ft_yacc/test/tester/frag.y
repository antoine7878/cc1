%{

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
int yylex();
void yyerror(const char *msg);
extern FILE *yyin;
/* %} and " inside block comment */
char *frag_msg = "esc \" %} ok";
// %} and " inside line comment
char quote = '"';
char pct = '%';

%}

%union {
    int i;
}
%start s
%token<i> NUMBER

%%

s
    : 'x' {printf ("%s\n", frag_msg);}
    ;

%%

void yyerror(const char *msg) {
    fprintf(stderr, "%s\n", msg);
}

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    yyparse();
}
