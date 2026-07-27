%{
#include <stdlib.h>
#include "lr0.tab.h"

%}

%%

[0-9]+          { yylval.i = atoi(yytext); return id; }

"+"             { return '+'; }
"*"             { return '*'; }

.               { /* ignore unknown characters, or handle error */ }

%%

int yywrap(void) {
    return 1;
}
