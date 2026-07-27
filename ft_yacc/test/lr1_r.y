%start S
%token a b

%%

S
    : X X
    ;

X
    : a X
    | b
    ;

%%
