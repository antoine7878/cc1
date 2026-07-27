%start S
%token IF ELSE A

%%

S
    : stmt
    | X
    | Y
    ;

stmt
    : IF stmt
    | IF stmt ELSE stmt
    | A
    ;

X
    : A
    ;

Y
    : A
    ;

%%
