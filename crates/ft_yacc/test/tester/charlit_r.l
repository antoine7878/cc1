%no_main
%no_context
    use crate::YYToken;

%%

"a"                 return YYToken::Char('a');
"b"                 return YYToken::Char('b');
"e"                 return YYToken::Char('e');
"n"                 return YYToken::Char('n');
"t"                 return YYToken::Char('t');

%%
