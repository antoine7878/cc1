%{
#include <stdio.h>
#include <stdlib.h>

#include "b.tab.h"
void error(const char *fmt, ...);

static int input ( void );

char escape(char ec) {
    switch(ec) {
        case '0':
            return 0;
        case 'e':
            return EOF;
        case '(':
            return '{';
        case ')':
            return '}';
        case 't':
            return '\t';
        case '\'':
            return '\'';
        case '"':
            return '"';
        case 'n':
            return '\n';
    }
    return ec;
}

int make_wchar(char *s) {
    int ret = 0;

    ++s;
    ret |= *s == '\\' ? escape(*(++s)) : *s;
    ++s;
    if (*s == '\'')
        return ret;
    ret <<= 8;
    ret |= *s == '\\' ? escape(*(++s)) : *s;
    return ret;
}

static void comment(void)
{
    int c;

    while ((c = input()) != 0) {
		if (c == '*') {
            while ((c = input()) == '*')
                ;
            if (c == '/')
                return;
            if (c == 0)
                break;
        }
    }
    error("unterminated comment");
}

%}

O   [0-7]
H   [0-9a-fA-F]
D   [0-9]
B   [0-1]
NZ  [1-9]
L   [a-zA-Z_]
A   [a-zA-Z0-9_]
WS  [ \t\v\n\f]
ES  (\\[0e()t*'n\"])
C   (.|{ES})
S   [-+]

%%

"auto"                          return AUTO;
"extrn"                         return EXTRN;
"case"                          return CASE;
"if"                            return IF;
"else"                          return ELSE;
"while"                         return WHILE;
"switch"                        return SWITCH;
"goto"                          return GOTO;
"return"                        return RETURN;

"--"                            return DEC;
"++"                            return INC;

"=="                            return EQ;
"!="                            return NEQ;
"<="                            return LOWER_EQ;
">="                            return GREATER_EQ;
"<<"                            return LEFT;
">>"                            return RIGHT;

"=|"                            return BITOR_ASSIGN;
"=&"                            return BITAND_ASSIGN;
"==="                           return EQ_ASSIGN;
"=!="                           return NEQ_ASSIGN;
"=<"                            return LOWER_ASSIGN;
"=<="                           return LOWER_EQ_ASSIGN;
"=>"                            return GREATER_ASSIGN;
"=>="                           return GREATER_EQ_ASSIGN;
"=<<"                           return LEFT_ASSIGN;
"=>>"                           return RIGHT_ASSIGN;
"=+"                            return ADD_ASSIGN;
"=-"                            return SUB_ASSIGN;
"=*"                            return MUL_ASSIGN;
"=/"                            return DIV_ASSIGN;
"=%"                            return MOD_ASSIGN;

"#+"                            return F_ADD;
"#-"                            return F_SUB;
"#*"                            return F_MUL;
"#/"                            return F_DIV;
"#"                             return TO_FLOAT;
"##"                            return TO_INT;

[!~|&+%*\/=<>()\[\]{},;\"'?:-]  return *yytext;

{L}{A}{0,7}                     { yylval.s = strdup(yytext); return NAME; }
{L}{A}*                         { error("%s name too long", yytext); }

0[bB]{B}*                       { yylval.i = strtol(yytext + 2, NULL, 2); return CST; }
0{O}*                           { yylval.i = strtol(yytext + 1, NULL, 8); return CST; }
{NZ}{D}*                        { yylval.i = strtol(yytext, NULL, 10); return CST; }
0[xX]{H}*                       { yylval.i = strtol(yytext + 2, NULL, 16); return CST; }
{D}+\.{D}+|\.{D}+|{D}+\.        { yylval.f = strtof(yytext, NULL); return CST; }


'{C}{1,2}'                      { yylval.i = make_wchar(yytext); return CST; }

\"([^"]|{ES})*\"                { yylval.s = strdup(yytext); return LITERAL; }

{WS}+                           ;

"/*"                            comment();
"*/"                            error("unopened comment");
.                               error("Unknown token: %s", yytext);

%%

int yywrap(void) {
    return 1;
}
