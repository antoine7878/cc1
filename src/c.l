%no_main

%{
use crate::symbol::NameId;
use crate::parser::YYToken;
use crate::context::{Context, ContextAccess};
%}

D			[0-9]
L			[a-zA-Z_]
H			[a-fA-F0-9]
E			[Ee][+-]?{D}+
FS			(f|F|l|L)
IS			(u|U|l|L)*

%%

#.*                         {}
"//".*                      {}
"/*"			            { comment(self); }
"auto"			            { return YYToken::AUTO; }
"break"			            { return YYToken::BREAK; }
"case"			            { return YYToken::CASE; }
"char"			            { return YYToken::CHAR; }
"const"			            { return YYToken::CONST; }
"continue"		            { return YYToken::CONTINUE; }
"default"		            { return YYToken::DEFAULT; }
"do"			            { return YYToken::DO; }
"double"		            { return YYToken::DOUBLE; }
"else"			            { return YYToken::ELSE; }
"enum"			            { return YYToken::ENUM; }
"extern"		            { return YYToken::EXTERN; }
"float"			            { return YYToken::FLOAT; }
"for"			            { return YYToken::FOR; }
"goto"			            { return YYToken::GOTO; }
"if"			            { return YYToken::IF; }
"int"			            { return YYToken::INT; }
"long"			            { return YYToken::LONG; }
"register"		            { return YYToken::REGISTER; }
"return"		            { return YYToken::RETURN; }
"short"			            { return YYToken::SHORT; }
"signed"		            { return YYToken::SIGNED; }
"sizeof"		            { return YYToken::SIZEOF; }
"static"		            { return YYToken::STATIC; }
"struct"		            { return YYToken::STRUCT; }
"switch"		            { return YYToken::SWITCH; }
"typedef"		            { return YYToken::TYPEDEF; }
"union"			            { return YYToken::UNION; }
"unsigned"		            { return YYToken::UNSIGNED; }
"void"			            { return YYToken::VOID; }
"volatile"		            { return YYToken::VOLATILE; }
"while"			            { return YYToken::WHILE; }
{L}({L}|{D})*		        { return check_type(alloc_name(self)); }
0[xX]{H}+{IS}?		        { return YYToken::CONSTANT(alloc_name(self)); }
0{D}+{IS}?		            { return YYToken::CONSTANT(alloc_name(self)); }
{D}+{IS}?		            { return YYToken::CONSTANT(alloc_name(self)); }
L?'(\\.|[^\\'])+'	        { return YYToken::CONSTANT(alloc_name(self)); }
{D}+{E}{FS}?		        { return YYToken::CONSTANT(alloc_name(self)); }
{D}*"."{D}+({E})?{FS}?	    { return YYToken::CONSTANT(alloc_name(self)); }
{D}+"."{D}*({E})?{FS}?	    { return YYToken::CONSTANT(alloc_name(self)); }
L?\"(\\.|[^\\"])*\"	        { return YYToken::STRING_LITERAL(alloc_name(self)); }
"..."			            { return YYToken::ELLIPSIS; }
">>="			            { return YYToken::RIGHT_ASSIGN; }
"<<="			            { return YYToken::LEFT_ASSIGN; }
"+="			            { return YYToken::ADD_ASSIGN; }
"-="			            { return YYToken::SUB_ASSIGN; }
"*="			            { return YYToken::MUL_ASSIGN; }
"/="			            { return YYToken::DIV_ASSIGN; }
"%="			            { return YYToken::MOD_ASSIGN; }
"&="			            { return YYToken::AND_ASSIGN; }
"^="			            { return YYToken::XOR_ASSIGN; }
"|="			            { return YYToken::OR_ASSIGN; }
">>"			            { return YYToken::RIGHT_OP; }
"<<"			            { return YYToken::LEFT_OP; }
"++"			            { return YYToken::INC_OP; }
"--"			            { return YYToken::DEC_OP; }
"->"			            { return YYToken::PTR_OP; }
"&&"			            { return YYToken::AND_OP; }
"||"			            { return YYToken::OR_OP; }
"<="			            { return YYToken::LE_OP; }
">="			            { return YYToken::GE_OP; }
"=="			            { return YYToken::EQ_OP; }
"!="			            { return YYToken::NE_OP; }
";"			                { return YYToken::Char(';'); }
","			                { return YYToken::Char(','); }
":"			                { return YYToken::Char(':'); }
"="			                { return YYToken::Char('='); }
"("			                { return YYToken::Char('('); }
")"			                { return YYToken::Char(')'); }
("["|"<:"|"??(")            { return YYToken::Char('['); }
("]"|":>"|"??)")            { return YYToken::Char(']'); }
"."			                { return YYToken::Char('.'); }
"&"			                { return YYToken::Char('&'); }
"!"			                { return YYToken::Char('!'); }
("~"|"??-")	                { return YYToken::Char('~'); }
"-"			                { return YYToken::Char('-'); }
"+"			                { return YYToken::Char('+'); }
"*"			                { return YYToken::Char('*'); }
"/"			                { return YYToken::Char('/'); }
"%"			                { return YYToken::Char('%'); }
"<"			                { return YYToken::Char('<'); }
(">"|"??'")	                { return YYToken::Char('>'); }
"^"			                { return YYToken::Char('^'); }
("|"|"??!")	                { return YYToken::Char('|'); }
"?"			                { return YYToken::Char('?'); }
("{"|"<%"|"??<")            { return YYToken::Char('{'); }
("}"|"%>")|"??>"            { return YYToken::Char('}'); }
[ \t\v\n\f]		            { }
.			                { /* ignore bad characters */ }

%%

fn comment<T: Read>(lex: &mut YYLex<T>) {
    let mut c = lex.input();
    while c != b'*' && c != 0 {
        c = lex.input();
        if c == b'*' {
            while c == b'*' {
                c = lex.input();
            }
            if c == b'/' || c == 0 {
                return;
            }
        }
    }
}

fn check_type(name_id: NameId) -> YYToken {
// pseudo code --- this is what it should check
//
//	if (yytext == type_name)
//	     return YYToken::TYPE_NAME;
	YYToken::IDENTIFIER(name_id)
}

fn alloc_name<R: Read>(lex: &mut YYLex<R>) -> NameId {
    let name = lex.take_yytext();
    lex.names().add(name)
}
