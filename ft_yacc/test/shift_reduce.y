%start stmt
%token IF ELSE ID

%%

stmt
    : IF expr stmt
    | IF expr stmt ELSE stmt
    | ID
    ;

expr
    : ID
    ;

%%
