%no_main
    use crate::{YYToken, YYLexer};

%%

[0-9]+\.[0-9]+      return YYToken::Real(self.yytext.parse::<f64>().unwrap());
[0-9]+              return YYToken::Int(self.yytext.parse::<i32>().unwrap());
[ \t]               ;
.|\n                return YYToken::Char(self.yytext.chars().next().unwrap());

%%
