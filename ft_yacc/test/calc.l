%pointer
%{
#include <unistd.h>
#include "calc.tab.h"
%}

%%

[0-9]+              { yylval = atoi(yytext); return NUMBER; }
[[:blank:]]         ; // skip blanks
.|\n return         *yytext; // literals

%%

int yywrap() {
    return 1;
}
