%start s
%token Id '*' '='

%%

s
    : l '=' r
    | r
    ;

l
    : '*' r
    | Id
    ;
r
    : l
    ;

%%
