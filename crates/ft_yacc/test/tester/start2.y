%no_main
%token<i32> NUMBER
%start second
%type<i32> first second

%%

first
    : 'a' NUMBER {$2 + 100}
    ;

second
    : NUMBER {$1}
    ;

%%
