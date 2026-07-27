%start S
%token A

%%

S
    : X
    | Y
    ;

X
    : A
    ;

Y
    : A
    ;

%%
