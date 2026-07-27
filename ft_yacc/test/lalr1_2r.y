%start S
%token E F
%token '=' '+' '(' ')' id int

%%

S
    : V '=' E
    ;

E
    : F
    | E '+' F
    ;

F
    : V
    | int
    | '(' E ')'
    ;

V
    : id
    ;

%%
