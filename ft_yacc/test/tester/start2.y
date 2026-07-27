%union {
    int i;
}
%token<i> NUMBER
%start second
%type<i> first second

%%

first
    : 'a' NUMBER {$$ = $2 + 100;}
    ;

second
    : NUMBER {$$ = $1;}
    ;

%%
