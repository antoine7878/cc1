%pointer
%{
#include <unistd.h>
#include <stdlib.h>
%}

%%

[0-9]+              { yylval.i = atoi(yytext); return NUMBER; }
[[:blank:]]         ; // skip blanks
.|\n return         *yytext; // literals

%%

int yywrap() {
    return 1;
}
