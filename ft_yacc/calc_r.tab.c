#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>


int yychar = -2;


#define STACK_CAP 100
#define yyerrok (yy_recovering = 0)
#define yyclearin (yychar = YYEMPTY)

#define YYACCEPT goto yyacceptlabel
#define YYABORT goto yyabortlabel
#define YYERROR goto yyerrorlabel

#define YYRECOVERING() (!!yy_recovering)

enum yysymbol_kind_t {
	YYSYMBOL_YYEMPTY = -2,
YYSYMBOL_YYEOF = 0,
YYSYMBOL_ERROR = 1,
YYSYMBOL_ACCEPT = 2,
YYSYMBOL_RVALUE = 3,
YYSYMBOL_NUMBER = 4,
YYSYMBOL_CHAR10 = 5,
YYSYMBOL_CHAR43 = 6,
YYSYMBOL_CHAR45 = 7,
YYSYMBOL_CHAR42 = 8,
YYSYMBOL_CHAR47 = 9,
YYSYMBOL_CHAR40 = 10,
YYSYMBOL_PROG = 11,
YYSYMBOL_EXPR = 12,
YYSYMBOL_CHAR41 = 13,
};
typedef enum yysymbol_kind_t yysymbol_kind_t;

static const ssize_t yy_gotos[21][14] ={
{0,-2,0,-3,-4,0,0,0,0,0,-5,-6,-7,0},
{0,0,0,0,0,-8,0,0,0,0,0,0,0,0},
{5,0,0,0,0,5,-9,-10,-11,-12,0,0,0,0},
{11,0,0,0,0,11,11,11,11,11,0,0,0,11},
{0,0,0,-13,-4,0,0,0,0,0,-5,0,0,0},
{1,0,0,0,0,0,0,0,0,0,0,0,0,0},
{2,0,0,0,0,-14,0,0,0,0,0,0,0,0},
{0,-2,0,-3,-4,0,0,0,0,0,-5,-15,-7,0},
{0,0,0,-16,-4,0,0,0,0,0,-5,0,0,0},
{0,0,0,-17,-4,0,0,0,0,0,-5,0,0,0},
{0,0,0,-18,-4,0,0,0,0,0,-5,0,0,0},
{0,0,0,-19,-4,0,0,0,0,0,-5,0,0,0},
{0,0,0,0,0,0,-9,-10,-11,-12,0,0,0,-20},
{0,-2,0,-3,-4,0,0,0,0,0,-5,-21,-7,0},
{4,0,0,0,0,0,0,0,0,0,0,0,0,0},
{6,0,0,0,0,6,6,6,-11,-12,0,0,0,6},
{7,0,0,0,0,7,7,7,-11,-12,0,0,0,7},
{8,0,0,0,0,8,8,8,8,8,0,0,0,8},
{9,0,0,0,0,9,9,9,9,9,0,0,0,9},
{10,0,0,0,0,10,10,10,10,10,0,0,0,10},
{3,0,0,0,0,0,0,0,0,0,0,0,0,0},
};
static const ssize_t yy_indexes[262] = {0,2,2,2,2,2,2,2,2,2,5,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,10,13,8,6,2,7,2,9,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,1,2,3,4,11,12};
static const size_t yy_rlens[11] = {1,1,3,3,1,3,3,3,3,3,1};
static const size_t yy_products[11] = {2,11,11,11,12,3,3,3,3,3,3};
static const ssize_t yy_actions[11] = {-1,-1,-1,-1,0,1,2,3,4,5,6};
static const ssize_t yy_default_act[21] = {0,0,0,11,0,0,0,0,0,0,0,0,0,0,4,0,0,8,9,10,3};
static const ssize_t yy_default_reduce_act[21] = {0,0,5,11,0,0,2,0,0,0,0,0,0,0,4,6,7,8,9,10,3};


const static int yy_lines [] = {0,14,15,16,20,24,25,26,27,28,29};
const static int yy_terminal [] = {1,1,1,0,1,1,1,1,1,1,1,0,0,1};
const static char* const yy_names[] = {"yyeof","error","invalid token","rvalue","Number","'\\n'","'+'","'-'","'*'","'/'","'('","prog","expr","')'"};

typedef struct {
	size_t state;
	YYSTYPE value;
#if YYDEBUG
	ssize_t id;
#endif
} yyparse_elt;

YYSTYPE yylval;
static yyparse_elt *yyparse_stack = NULL;
static yyparse_elt *yyparse_stack_top = NULL;
static size_t yyparse_stack_cap = STACK_CAP;
static YYSTYPE yychar_value;
static ssize_t yychar_id;
static int yyact;
static size_t yylen;
static YYSTYPE yyval;
static int yy_recovering = 0;
static int yytoken_since_error = 0;

#ifndef yydebug
int yydebug;
#endif

#ifndef yyerror
void yyerror(const char *);
#endif

int yyparse(void);

#ifndef yylex
int yylex(void);
#endif

// ----- error --------------------

void yyexiterror(const char *msg) {
	fprintf(stderr, "%s\n", msg);
	free(yyparse_stack);
	yyparse_stack = NULL;
	exit(1);
}

// ----- debug --------------------

#if YYDEBUG
void yylog(const char *fmt, ...) {
	if (!yydebug) {
		return;
	}
	va_list ap;
	va_start(ap, fmt);
	vfprintf(stderr, fmt, ap);
	va_end(ap);
}

const char *token_class(size_t token_id) {
	return yy_terminal[token_id] ? "token" : "nterm";
}

void print_stack() {
	if (!yydebug) {
		return;
	}
	fprintf(stderr, "Stack now ");
	yyparse_elt *it = yyparse_stack;
	while (it < yyparse_stack_top) {
		fprintf(stderr, "%zu", it->state);
		it++;
		if (it < yyparse_stack_top + 1)
			fprintf(stderr, ", ");
	}
	fprintf(stderr, "\n");
}
#endif

// ----- parse stack management --------------------

size_t yyparse_stack_len() {
	return yyparse_stack_top - yyparse_stack + 1;
}

void yyparse_stack_drain(size_t len) {
	if (yyparse_stack_len() < len) {
		yyexiterror("parse stack underflow");
	}
	yyparse_stack_top -= len;
}

void yyparse_stack_pop() {
	yyparse_stack_drain(1);
}

void yyparse_stack_push_eps(int state, ssize_t token_id) {
	if (yyparse_stack_len() >= yyparse_stack_cap) {
		yyparse_stack_cap += STACK_CAP;
		size_t len = yyparse_stack_len() - 1;
		yyparse_stack = realloc(yyparse_stack, yyparse_stack_cap * sizeof(yyparse_elt));
		yyparse_stack_top = yyparse_stack + len;
		if (!yyparse_stack) {
			yyexiterror("alloc failed");
		}
	}
	yyparse_stack_top++;
	yyparse_stack_top->state = state;
#if YYDEBUG
	yyparse_stack_top->id = token_id;
	yylog("Entering state %zu\n", state);
	print_stack();
#endif
}

void yyparse_stack_push(int state, YYSTYPE value, ssize_t token_id) {
	yyparse_stack_push_eps(state, token_id);
	yyparse_stack_top->value = value;
}

// ----- parser actions --------------------

void yyread_token() {
	yychar = yylex();
	if (yychar <= 0) {
		yychar = YYEOF;
	}
	yychar_id = yy_indexes[yychar];
	yychar_value = yylval;

#if YYDEBUG
	yylog("Reading a token\n");
	if (yychar == 0) {
		yylog("Now at end of input.\n");
	} else {
		yylog("Next token is token %s\n", yy_names[yychar_id]);
	}
#endif
}

void yyshift() {
	yyact += 1;

	if (yy_recovering && yytoken_since_error >= 2) {
		yy_recovering = 0;
#if YYDEBUG
		yylog("Error recovered\n");
#endif
	} else if (yy_recovering) {
		yytoken_since_error += 1;
	}

	if (yychar == YYEMPTY)
		yyread_token();
#if YYDEBUG
	yylog("Shifting token %s\n", yy_names[yychar_id]);
#endif
	yyparse_stack_push(-yyact, yychar_value, yychar_id);
	yychar = YYEMPTY;
	yychar_id = YYSYMBOL_YYEMPTY;
}

int yynext_act() {
	size_t state_id = yyparse_stack_top->state;
	ssize_t dflt = yy_default_act[state_id];
	if (dflt)
		return dflt;
	if (yychar == YYEMPTY)
		yyread_token();
	dflt = yy_gotos[state_id][yychar_id];
	if (dflt)
		return dflt;
	return yy_default_reduce_act[state_id];
}

// ----- error --------------------

int yyunwind() {
	while (yyparse_stack_len() > 0 && yy_gotos[yyparse_stack_top->state][YYSYMBOL_ERROR] >= 0) {
#if YYDEBUG
		yylog(
			"Error: popping %s %s\n",
			token_class(yyparse_stack_top->id),
			yy_names[yyparse_stack_top->id]);
		print_stack();
#endif
		yyparse_stack_pop();
	}
	if (yyparse_stack_len() == 0) {
		return 1;
	}
	yy_recovering = 1;
	yyact = yy_gotos[yyparse_stack_top->state][YYSYMBOL_ERROR] + 1;
#if YYDEBUG
	yylog("Shifting token error\n");
#endif
	yyparse_stack_push_eps(-yyact, YYSYMBOL_ERROR);
	yytoken_since_error = 0;
	yyact = yy_gotos[yyparse_stack_top->state][yychar_id];
	while (yyact == 0 && yychar != YYEOF) {
		yyclearin;
		yyread_token();
		yyact = yy_gotos[yyparse_stack_top->state][yychar_id];
	}
	return 0;
}

// ----- yyparse --------------------

int yyparse() {
#if YYDEBUG
	yylog("Start parsing\n");
#endif

	yyparse_stack = malloc(yyparse_stack_cap * sizeof(yyparse_elt));
	yyparse_stack_top = yyparse_stack;
	if (!yyparse_stack) {
		yyexiterror("alloc failed");
	}

	yyparse_stack_push_eps(0, 0);

	yyread_token();

	while (1) {
		yyact = yynext_act();
		if (yyact == 1) {
			break;
		}
		if (yyact > 0) {

			yyact -= 1;
#if YYDEBUG
			yylog("Reducing stack by rule %d (line %d)\n", yyact, yy_lines[yyact]);
#endif
			size_t len = yy_rlens[yyact];
			size_t product_id = yy_products[yyact];
			yylen = yy_rlens[yyact];
			yyval = (yyparse_stack_top)[1 - yylen].value;
#if YYDEBUG
			int i = 0;
			while (i < len) {
				yyparse_elt *token = yyparse_stack_top - i;
				yylog("   $%d = %s %s\n", i + 1, token_class(token->id), yy_names[token->id]);
				i++;
			}
			yylog("-> $$ = %s %s\n", token_class(product_id), yy_names[product_id]);
#endif

			switch (yy_actions[yyact]) {
case 0:
#line 20 "test/tester/calc_r.y"
{println!("= {}", (yyparse_stack_top[0].value