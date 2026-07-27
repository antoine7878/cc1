%start S
%token a b c d e

%%

S
    : a B c
    | b C c
    | a C d
    | b B d
    ;

B
    : e
    ;

C
    : e
    ;

%%
