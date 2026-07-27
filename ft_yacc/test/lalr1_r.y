%start S
%token a b

%%

S
    : X X {println!("0");}
    ;

X
    : a {println!("mid");} X {println!("1");}
    | b {println!("2");}
    ;

%%
