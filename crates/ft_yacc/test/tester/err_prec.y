%token A

%%

s
    : A %prec B
    ;

%%
