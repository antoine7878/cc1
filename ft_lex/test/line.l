%{
    /*  
     *   word count 
     */
int     nchar, nword, nline;
%}

%x LINE

%%

\n          { ++nchar, ++nline; }
[^ \t\n]+   { ++nword, nchar += yyleng; }
.           { ++nchar; }

%%

int main(int argc, char *argv[])
{
	yyin = fopen(argv[1], "r");
    yylex();
    printf("\t%d\t%d\t%d %s\n", nline, nword, nchar, argv[1]);
}
