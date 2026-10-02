%no_main
%custom_span_types

%{
use crate::ast::{Name, StringLiteralNode, ConstValueNode, ConstValue};
use crate::context::Context;
use libft::{Position, Span};
use crate::parser::{YYToken, YYFeedback};
use crate::semantic::{Diag, Diagnostic, DiagnosticNode};

%}

D			[0-9]
L			[a-zA-Z_]
H			[a-fA-F0-9]
E			[Ee][+-]?{D}+
FS			(f|F|l|L)
IS			(u|U|l|L|ul|UL|Ul|uL|lu|LU|lU|Lu)
PP			"."?{D}({D}|{L}|[eE][+-]|".")*

%%

^#[ \t]*[0-9]+.*\n          { line_marker(self); }
"//".*                      { self.ctx.diagnostics.push(DiagnosticNode::new(Diagnostic::CPPStyleComment, self.span)); }
^[ \t]*#.*                  { directive(self); }
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

{L}({L}|{D})*		        { let name = alloc_ident(self); return self.ctx.parse.classify_identifier(name); }

L?'(\\.|[^\\'\n])+'	    { return YYToken::CONSTANT(alloc_constant(self)); }
L?'(\\.|[^\\'\n])*	        { unterminated(self); }
L?''                        { empty_constant(self); }
0[xX]{H}+{IS}?		        { return YYToken::CONSTANT(alloc_constant(self)); }
0{D}+{IS}?		            { return YYToken::CONSTANT(alloc_constant(self)); }
{D}+{IS}?		            { return YYToken::CONSTANT(alloc_constant(self)); }
{D}+{E}{FS}?		        { return YYToken::CONSTANT(alloc_constant(self)); }

{D}*"."{D}+({E})?{FS}?	    { return YYToken::CONSTANT(alloc_constant(self)); }
{D}+"."{D}*({E})?{FS}?	    { return YYToken::CONSTANT(alloc_constant(self)); }
{PP}			            { return YYToken::CONSTANT(invalid_number(self)); }

L?\"(\\.|[^\\"\n])*\"	    { return YYToken::STRING_LITERAL(alloc_string(self)); }
L?\"(\\.|[^\\"\n])*	        { unterminated(self); }

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
"["			                { return YYToken::Char('['); }
"]"			                { return YYToken::Char(']'); }
"."			                { return YYToken::Char('.'); }
"&"			                { return YYToken::Char('&'); }
"!"			                { return YYToken::Char('!'); }
"~"			                { return YYToken::Char('~'); }
"-"			                { return YYToken::Char('-'); }
"+"			                { return YYToken::Char('+'); }
"*"			                { return YYToken::Char('*'); }
"/"			                { return YYToken::Char('/'); }
"%"			                { return YYToken::Char('%'); }
"<"			                { return YYToken::Char('<'); }
">"			                { return YYToken::Char('>'); }
"^"			                { return YYToken::Char('^'); }
"|"			                { return YYToken::Char('|'); }
"?"			                { return YYToken::Char('?'); }
"{"			                { return YYToken::Char('{'); }
"}"			                { return YYToken::Char('}'); }
[ \t\v\n\f]		            { }
[\xc0-\xff][\x80-\xbf]*     { stray(self); }
.			                { stray(self); }

%%

impl<R: Read> YYFeedback for YYLex<R> {
    fn yy_feedback(&mut self, accepts: &dyn Fn(usize) -> bool) {
        self.ctx.parse.feedback(accepts);
    }
}

fn line_marker<R: Read>(lex: &mut YYLex<R>) {
    let text = lex.yytext.clone();
    let rest = text.trim_start().trim_start_matches('#').trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    let Ok(line) = rest[..end].parse::<usize>() else { return };
    lex.ctx.preprocessed = true;

    if let Some(file) = rest[end..].trim_start().strip_prefix('"').and_then(|s| s.split('"').next()) {
        lex.pos.file = lex.ctx.arenas.names.intern(file.to_string()).into();
    }
    lex.ctx.mark_line(lex.pos.file, line, lex.pos.line);
    lex.pos.line = line;
    lex.pos.col = 1;
}

fn directive<R: Read>(lex: &mut YYLex<R>) {
    let indent = lex.yytext.len() - lex.yytext.trim_start_matches([' ', '\t']).len();
    if indent > 0 && lex.ctx.preprocessed {
        let pushed = lex.yytext.len() - indent;
        lex.yyless(indent);
        lex.uncounted -= pushed;
        lex.pos.col -= pushed;
    }
}

fn alloc_ident<R: Read>(lex: &mut YYLex<R>) -> Name {
    let span = lex.span;
    let name = lex.yytext.clone();
    lex.ctx.arenas.names.add(name, span)
}

fn alloc_constant<R: Read>(lex: &mut YYLex<R>) -> ConstValueNode {
    let span = lex.span;
    let Diag { res: value, diagnostic } = ConstValue::parse(lex.yytext.as_str());
    if let Some(diagnostic) = diagnostic {
        lex.ctx.diagnostics.push(DiagnosticNode::new(diagnostic, span));
    }
    ConstValueNode::new(value, span)
}

fn invalid_number<R: Read>(lex: &mut YYLex<R>) -> ConstValueNode {
    let span = lex.span;
    let text = lex.yytext.clone();
    lex.ctx.diagnostics.push(DiagnosticNode::new(Diagnostic::InvalidNumber(text), span));
    ConstValueNode::new(ConstValue::Int(0), span)
}

fn alloc_string<R: Read>(lex: &mut YYLex<R>) -> StringLiteralNode {
    let span = lex.span;
    let text = lex.yytext.clone();
    let Diag { res: node, diagnostic } = lex.ctx.arenas.strings.literal(&text, span);
    if let Some(diagnostic) = diagnostic {
        lex.ctx.diagnostics.push(DiagnosticNode::new(diagnostic, span));
    }
    node
}

fn stray<R: Read>(lex: &mut YYLex<R>) {
    let span = lex.span;
    let text = lex.yytext.clone();
    lex.ctx.diagnostics.push(DiagnosticNode::new(Diagnostic::StrayCharacter(text), span));
}

fn empty_constant<R: Read>(lex: &mut YYLex<R>) {
    let span = lex.span;
    lex.ctx.diagnostics.push(DiagnosticNode::new(Diagnostic::EmptyCharacterConstant, span));
}

fn unterminated<R: Read>(lex: &mut YYLex<R>) {
    let span = lex.span;
    let quote = lex.yytext.trim_start_matches('L').chars().next().unwrap_or_default();
    lex.ctx.diagnostics.push(DiagnosticNode::new(Diagnostic::MissingTerminatingCharacter(quote), span));
}
