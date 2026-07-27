%pointer
%{
#include <unistd.h>
#include <stdlib.h>
%}

%%

[0-9]+\.[0-9]+      { yylval.d = atof(yytext); return REAL; }
[0-9]+              { yylval.i = atoi(yytext); return INT; }
[[:blank:]]         ; // skip blanks
.|\n                return *yytext; // literals

%%

int yywrap() {
    return 1;
}
