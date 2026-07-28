%no_main
    use crate::{YYToken, YYLexer};

%%

[0-9]+              return YYToken::Number(self.yytext.parse::<f32>().unwrap());
[ \t]               ;
.|\n                return YYToken::Char(self.yytext.chars().next().unwrap());

%%
