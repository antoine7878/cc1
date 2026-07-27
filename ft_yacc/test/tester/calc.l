%pointer
%{
#include <unistd.h>
%}

%%

[0-9]+              { yylval.i = atoi(yytext); return NUMBER; }
.|\n return         *yytext; // literals

%%

int yywrap() {
    return 1;
}
