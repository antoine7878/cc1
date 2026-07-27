%no_main
%{
use crate::YYToken;
%}

%%

[0-9]+              return YYToken::Number(self.yytext.parse::<f32>().unwrap());
"+"                 return YYToken::Char('+');
"-"                 return YYToken::Char('-');
"*"                 return YYToken::Char('*');
"/"                 return YYToken::Char('/');
"("                 return YYToken::Char('(');
")"                 return YYToken::Char(')');
"\n"                return YYToken::Char('\n');

%%
