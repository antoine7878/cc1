%start S
%token a b c


%%

S : A b
  | A c
  ;

A : a
  | a b
  | a c
  ;
%%
