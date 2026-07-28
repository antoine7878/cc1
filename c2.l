O   [0-7]
D   [0-9]
NZ  [1-9]
L   [a-zA-Z_]
A   [a-zA-Z_0-9]
H   [a-fA-F0-9]
HP  (0[xX])
E   ([Ee][+-]?{D}+)
P   ([Pp][+-]?{D}+)
FS  (f|F|l|L)
IS  (((u|U)(l|L|ll|LL)?)|((l|L|ll|LL)(u|U)?))
CP  (u|U|L)
SP  (u8|u|U|L)
ES  (\\(['"\?\\abfnrtv]|[0-7]{1,3}|x[a-fA-F0-9]+))
WS  [ \t\v\n\f]

%{
#include <stdio.h>

extern void yyerror(const char *);  /* prints grammar violation message */

extern int sym_type(const char *);  /* returns type from symbol table */

#define sym_type(identifier) IDENTIFIER /* with no symbol table, fake it */

static void comment(void);
static int check_type(void);
%}

%%

(([a][a-]?([a])+))		            { printf("token: F_CONSTANT, value: %s\n", yytext); }

("}"|"%>")				            { printf("token: '}', value: %s\n", yytext); }

#.*	/*ignore preproc*/;
"/*"                                { printf("comment: %s", yytext); comment(); putchar('\n'); }
"//".*                              { printf("comment: %s\n", yytext);}

"auto"					            { printf("token: AUTO, value: %s\n", yytext); }
"break"					            { printf("token: BREAK, value: %s\n", yytext); }
"case"					            { printf("token: CASE, value: %s\n", yytext); }
"char"					            { printf("token: CHAR, value: %s\n", yytext); }
"const"					            { printf("token: CONST, value: %s\n", yytext); }
"continue"				            { printf("token: CONTINUE, value: %s\n", yytext); }
"default"				            { printf("token: DEFAULT, value: %s\n", yytext); }
"do"					            { printf("token: DO, value: %s\n", yytext); }
"double"				            { printf("token: DOUBLE, value: %s\n", yytext); }
"else"					            { printf("token: ELSE, value: %s\n", yytext); }
"enum"					            { printf("token: ENUM, value: %s\n", yytext); }
"extern"				            { printf("token: EXTERN, value: %s\n", yytext); }
"float"					            { printf("token: FLOAT, value: %s\n", yytext); }
"for"					            { printf("token: FOR, value: %s\n", yytext); }
"goto"					            { printf("token: GOTO, value: %s\n", yytext); }
"if"					            { printf("token: IF, value: %s\n", yytext); }
"inline"				            { printf("token: INLINE, value: %s\n", yytext); }
"int"					            { printf("token: INT, value: %s\n", yytext); }
"long"					            { printf("token: LONG, value: %s\n", yytext); }
"register"				            { printf("token: REGISTER, value: %s\n", yytext); }
"restrict"				            { printf("token: RESTRICT, value: %s\n", yytext); }
"return"				            { printf("token: RETURN, value: %s\n", yytext); }
"short"					            { printf("token: SHORT, value: %s\n", yytext); }
"signed"				            { printf("token: SIGNED, value: %s\n", yytext); }
"sizeof"				            { printf("token: SIZEOF, value: %s\n", yytext); }
"static"				            { printf("token: STATIC, value: %s\n", yytext); }
"struct"				            { printf("token: STRUCT, value: %s\n", yytext); }
"switch"				            { printf("token: SWITCH, value: %s\n", yytext); }
"typedef"				            { printf("token: TYPEDEF, value: %s\n", yytext); }
"union"					            { printf("token: UNION, value: %s\n", yytext); }
"unsigned"				            { printf("token: UNSIGNED, value: %s\n", yytext); }
void					            { printf("token: VOID, value: %s\n", yytext); }
"volatile"				            { printf("token: VOLATILE, value: %s\n", yytext); }
"while"					            { printf("token: WHILE, value: %s\n", yytext); }
"_Alignas"                          { printf("token: ALIGNAS, value: %s\n", yytext); }
"_Alignof"                          { printf("token: ALIGNOF, value: %s\n", yytext); }
"_Atomic"                           { printf("token: ATOMIC, value: %s\n", yytext); }
"_Bool"                             { printf("token: BOOL, value: %s\n", yytext); }
"_Complex"                          { printf("token: COMPLEX, value: %s\n", yytext); }
"_Generic"                          { printf("token: GENERIC, value: %s\n", yytext); }
"_Imaginary"                        { printf("token: IMAGINARY, value: %s\n", yytext); }
"_Noreturn"                         { printf("token: NORETURN, value: %s\n", yytext); }
"_Static_assert"                    { printf("token: STATIC_ASSERT, value: %s\n", yytext); }
"_Thread_local"                     { printf("token: THREAD_LOCAL, value: %s\n", yytext); }
"__func__"                          { printf("token: FUNC_NAME, value: %s\n", yytext); }

{L}{A}*					            { printf("token: TYPE_OR_IDENTIFIER, value: %s\n", yytext); }

{HP}{H}+{IS}?				        { printf("token: I_CONSTANT, value: %s\n", yytext); }
{NZ}{D}*{IS}?				        { printf("token: I_CONSTANT, value: %s\n", yytext); }
"0"{O}*{IS}?				        { printf("token: I_CONSTANT, value: %s\n", yytext); }
{CP}?"'"([^'\\\n]|{ES})+"'"		    { printf("token: I_CONSTANT, value: %s\n", yytext); }

{D}*"."{D}+{E}?{FS}?			    { printf("token: F_CONSTANT, value: %s\n", yytext); }
{D}+"."{E}?{FS}?			        { printf("token: F_CONSTANT, value: %s\n", yytext); }
{HP}{H}+{P}{FS}?			        { printf("token: F_CONSTANT, value: %s\n", yytext); }
{HP}{H}*"."{H}+{P}{FS}?		        { printf("token: F_CONSTANT, value: %s\n", yytext); }
{HP}{H}+"."{P}{FS}?			        { printf("token: F_CONSTANT, value: %s\n", yytext); }

({SP}?\"([^"\\\n]|{ES})*\"{WS}*)+   { printf("token: STRING_LITERAL, value: %s\n", yytext); }

"..."					            { printf("token: ELLIPSIS, value: %s\n", yytext); }
">>="					            { printf("token: RIGHT_ASSIGN, value: %s\n", yytext); }
"<<="					            { printf("token: LEFT_ASSIGN, value: %s\n", yytext); }
"+="					            { printf("token: ADD_ASSIGN, value: %s\n", yytext); }
"-="					            { printf("token: SUB_ASSIGN, value: %s\n", yytext); }
"*="					            { printf("token: MUL_ASSIGN, value: %s\n", yytext); }
"/="					            { printf("token: DIV_ASSIGN, value: %s\n", yytext); }
"%="					            { printf("token: MOD_ASSIGN, value: %s\n", yytext); }
"&="					            { printf("token: AND_ASSIGN, value: %s\n", yytext); }
"^="					            { printf("token: XOR_ASSIGN, value: %s\n", yytext); }
"|="					            { printf("token: OR_ASSIGN, value: %s\n", yytext); }
">>"					            { printf("token: RIGHT_OP, value: %s\n", yytext); }
"<<"					            { printf("token: LEFT_OP, value: %s\n", yytext); }
"++"					            { printf("token: INC_OP, value: %s\n", yytext); }
"--"					            { printf("token: DEC_OP, value: %s\n", yytext); }
"->"					            { printf("token: PTR_OP, value: %s\n", yytext); }
"&&"					            { printf("token: AND_OP, value: %s\n", yytext); }
"||"					            { printf("token: OR_OP, value: %s\n", yytext); }
"<="					            { printf("token: LE_OP, value: %s\n", yytext); }
">="					            { printf("token: GE_OP, value: %s\n", yytext); }
"=="					            { printf("token: EQ_OP, value: %s\n", yytext); }
"!="					            { printf("token: NE_OP, value: %s\n", yytext); }
";"					                { printf("token: ';', value: %s\n", yytext); }
("{"|"<%")			                { printf("token: '{', value: %s\n", yytext); }
","					                { printf("token: ',', value: %s\n", yytext); }
":"					                { printf("token: ':', value: %s\n", yytext); }
"="					                { printf("token: '=', value: %s\n", yytext); }
"("					                { printf("token: '(', value: %s\n", yytext); }
")"					                { printf("token: ')', value: %s\n", yytext); }
("["|"<:")			                { printf("token: '[', value: %s\n", yytext); }
("]"|":>")			                { printf("token: ']', value: %s\n", yytext); }
"."					                { printf("token: '.', value: %s\n", yytext); }
"&"					                { printf("token: '&', value: %s\n", yytext); }
"!"					                { printf("token: '!', value: %s\n", yytext); }
"~"					                { printf("token: '~', value: %s\n", yytext); }
"-"					                { printf("token: '-', value: %s\n", yytext); }
"+"					                { printf("token: '+', value: %s\n", yytext); }
"*"					                { printf("token: '*', value: %s\n", yytext); }
"/"					                { printf("token: '/', value: %s\n", yytext); }
"%"					                { printf("token: '%%', value: %s\n", yytext); }
"<"					                { printf("token: '<', value: %s\n", yytext); }
">"					                { printf("token: '>', value: %s\n", yytext); }
"^"					                { printf("token: '^', value: %s\n", yytext); }
"|"					                { printf("token: '|', value: %s\n", yytext); }
"?"					                { printf("token: '?', value: %s\n", yytext); }

{WS}+				                { /* whitespace separates tokens */ }

.					                { printf("Unmatched character: '%c'\n", *yytext); }

%%

static void comment(void)
{
    int c;

    while ((c = input()) != 0) {
		putchar(c);
		if (c == '*') {
            while ((c = input()) == '*')
                ;

            if (c == '/')
                return;

            if (c == 0)
                break;
        }
    }
    fprintf(stderr, "unterminated comment");
}
