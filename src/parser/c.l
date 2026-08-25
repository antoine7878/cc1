%no_main

%{
use std::fmt;

use crate::ast::{Name, ValueNode, Value};
use crate::parser::{YYToken, YYFeedback, Context};
use crate::semantic::{SymbolKind};

%}

D			[0-9]
L			[a-zA-Z_]
H			[a-fA-F0-9]
E			[Ee][+-]?{D}+
FS			(f|F|l|L)
IS			(u|U|l|L|ul|UL|Ul|uL|lu|LU|lU|Lu)

%%

^#[ \t]*[0-9]+.*\n         { line_marker(self); }
#.*                         {}
"//".*                      {}
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

{L}({L}|{D})*		        { let name = alloc_name(self); return self.ctx.check_type(name); }

L?'(\\.|[^\\'])+'	        { return YYToken::CONSTANT(ValueNode::new(Value::from(self.yytext.as_str()), self.span)); }
0[xX]{H}+{IS}?		        { return YYToken::CONSTANT(ValueNode::new(Value::from(self.yytext.as_str()), self.span)); }
0{D}+{IS}?		            { return YYToken::CONSTANT(ValueNode::new(Value::from(self.yytext.as_str()), self.span)); }
{D}+{IS}?		            { return YYToken::CONSTANT(ValueNode::new(Value::from(self.yytext.as_str()), self.span)); }
{D}+{E}{FS}?		        { return YYToken::CONSTANT(ValueNode::new(Value::from(self.yytext.as_str()), self.span)); }
{D}*"."{D}+({E})?{FS}?	    { return YYToken::CONSTANT(ValueNode::new(Value::from(self.yytext.as_str()), self.span)); }
{D}+"."{D}*({E})?{FS}?	    { return YYToken::CONSTANT(ValueNode::new(Value::from(self.yytext.as_str()), self.span)); }

\"(\\.|[^\\"])*\"	        { let len = self.yytext.len(); return YYToken::STRING_LITERAL(self.yytext[1..len-1].to_string()); }
L\"(\\.|[^\\"])*\"	        { let len = self.yytext.len(); return YYToken::STRING_LITERAL(self.yytext[2..len-1].to_string()); }

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

impl<R: Read> YYFeedback for YYLex<R> {
    fn yy_feedback(&mut self, accepts: &dyn Fn(usize) -> bool) {
        self.ctx.feedback(accepts);
    }
}

fn line_marker<R: Read>(lex: &mut YYLex<R>) {
    let text = lex.yytext.clone();
    let rest = text.trim_start().trim_start_matches('#').trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    let Ok(line) = rest[..end].parse::<usize>() else { return };

    if let Some(file) = rest[end..].trim_start().strip_prefix('"').and_then(|s| s.split('"').next()) {
        lex.pos.file = lex.ctx.arenas.names.alloc(file.to_string()).into();
    }
    lex.pos.line = line;
    lex.pos.col = 1;
}

fn alloc_name<R: Read>(lex: &mut YYLex<R>) -> Name {
    let span = lex.span;
    let name = lex.yytext.clone();
    lex.ctx.arenas.names.add(name, span)
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.start.line == self.end.line && self.start.col == self.end.col) {
            false => write!(f, "<{}:{}, {}:{}>", self.start.line, self.start.col, self.end.line, self.end.col),
            true => write!(f, "<{}:{}>", self.start.line, self.start.col),
        }
    }
}
