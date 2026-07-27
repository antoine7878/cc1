%{
#define _GNU_SOURCE
#include <search.h>
#include <stdbool.h>
#include <sys/types.h>
#include <stddef.h>
#include <string.h>
#include <stdarg.h>


#define MAX_CONTEXT_LEN 256
#define MAX_CALL_STACK_SIZE 256

#define ABS(a) (((a) < 0) ? -(a) : (a))

typedef enum
{
	STACK_S,
	EXTRN_S
} sym_type;

typedef struct
{
	sym_type type;
	char *name;
} t_symbol;

typedef struct
{
	sym_type type;
	char *name;
} t_symbol_global;

typedef struct
{
	sym_type type;
	char *name;
	int stack_offset;
} t_symbol_stack;

typedef struct
{
	void *symbols;
	char *name;
	int frame_size;
	int start_label;
	int end_label;
} t_context;

typedef union {
	float f;
	int u;
} float_conv;

void emit(const char *fmt, ...);
void emit_raw(const char *fmt, ...);
void error(const char *fmt, ...);
char *add_extrn_symbol(char *name);
char *add_arg_symbol(char *name, int pos);
char *add_auto_symbol(char *name);
t_symbol *get_symbol(char *name);
t_context *get_top_context();
t_context *get_fn_context();
size_t get_context_len();
t_context *get_context(size_t i);
char *get_fn_name();
void push_context();
void pop_context();
int get_jump_label();
int get_constant_label();
void reverse_stack(int len);
void push_call();
void add_arg();
int get_arg_count();
void pop_call();
void post(const char *instruction);
void pre(const char *instruction);
void compare(const char *set);
void op(const char *instruction);
void shift(const char *instruction);
void op_mod();
void op_div();
void op_mul();
void compare_assign(const char *set);
void op_assign(const char *instruction);
void shift_assign(const char *instruction);
void mul_assign();
void mod_assign();
void div_assign();
void fn_decl_enter(char *fn_name);
void argument(char *name);
void global_int(char *name, int value);
void global_string(char *name, const char *value);
void global_vector(char *name);
void global_vector_extra(int count, int start);
void switch_start();
void switch_end();
void case_start(int cst);
void case_end();
void to_float();
void to_int();
void opf(const char *op);
int yylex();
void yyerror(const char*);

static int call_stack[MAX_CALL_STACK_SIZE] = {0};
static int call_stack_len = 0;
static int arg_counter;
static int array_len;
static int switch_depth;
static int last_case;

%}

%union {
    int i;
    char *s;
    float f;
}

%token AUTO EXTRN IF CASE WHILE SWITCH GOTO RETURN
%token<i> CST
%token<s> NAME LITERAL
%nonassoc LOWER_THAN_ELSE
%nonassoc  ELSE
%right '=' EQ_ASSIGN LOWER_ASSIGN NEQ_ASSIGN GREATER_ASSIGN GREATER_EQ_ASSIGN LEFT_ASSIGN RIGHT_ASSIGN LOWER_EQ_ASSIGN ADD_ASSIGN SUB_ASSIGN MUL_ASSIGN DIV_ASSIGN MOD_ASSIGN BITOR_ASSIGN BITAND_ASSIGN
%right '?' ':'
%left '|'
%left '&'
%nonassoc EQ NEQ
%nonassoc '<' '>' LOWER_EQ GREATER_EQ
%left RIGHT LEFT
%left '+' '-' F_ADD F_SUB
%left '*' '/' '%' F_MUL F_DIV
%right '!' '~' INC DEC UADDR USUB UF_SUB TO_FLOAT TO_INT
%nonassoc '[' '('

%%

program
    :
    | program definition
    ;

definition
    : NAME '[' { global_vector($1); array_len = 0; } vector ';' { emit(""); }
    | NAME ';' { global_int($1, 0); }
    | NAME CST ';' { global_int($1, $2); }
    | NAME LITERAL ';' { global_string($1, $2); free($2); }
    | NAME '(' { fn_decl_enter($1); } arguments ')' statement { emit("leave"); emit("ret"); pop_context(); }
    | NAME '(' { fn_decl_enter($1); } ')' statement { emit("leave"); emit("ret"); pop_context(); }
    ;

vector
	: CST ']' ivals { global_vector_extra($1 - array_len, 0); }
	| CST ']' { global_vector_extra($1, 1); }
    | ']' ivals
    ;

ivals
    : ival { ++array_len; }
    | ivals ',' { ++array_len; emit_raw(", "); } ival 
    ;

ival
    : CST { emit_raw("%d", $1); }
	| NAME { emit_raw("\"%s\"", $1); free($1); }
	;

constant
    : CST { emit("mov eax, %d", $1); }
    | LITERAL {
        int id = get_constant_label();
        emit(".section .rodata");
        emit(".LC%d:", id);
        emit(".long .LC%d + 4", id);
        emit(".string %s", $1);
        emit(".text");
        emit("mov eax, .LC%d", id);
        free($1);
    }
    ;

arguments
    : NAME { argument($1); }
    | arguments ',' NAME { argument($3); }
    ;

statement
    : AUTO auto_names ';' statement
    | EXTRN extrn_names ';' statement
    | NAME ':' { emit("%s:", $1); free($1); } statement
    | SWITCH '(' rvalue ')' { switch_start(); } statement { switch_end(); }
    | CASE CST ':' { case_start($2); } statement { case_end(); }
    | '{' { push_context(); } statements '}' { pop_context(); }
    | '{' '}'
    | if_start statement %prec LOWER_THAN_ELSE {
        emit(".I%d:", get_top_context()->end_label);
        pop_context();
    }
    | if_start statement ELSE {
        get_top_context()->start_label = get_jump_label();
        emit("jmp .I%d", get_top_context()->start_label);
        emit(".I%d:", get_top_context()->end_label);
    } statement {
        emit(".I%d:", get_top_context()->start_label);
        pop_context();
    }
    | WHILE {
        push_context();
        get_top_context()->start_label = get_jump_label();
        emit(".L%d:", get_top_context()->start_label);
    } '(' rvalue ')' {
        get_top_context()->end_label = get_jump_label();
        emit("cmp eax, 0");
        emit("je .L%d", get_top_context()->end_label);
    } statement {
        emit("jmp .L%d", get_top_context()->start_label);
        emit(".L%d:", get_top_context()->end_label);
        pop_context();
    }
    | GOTO NAME ';' { emit("jmp %s", $2); free($2); }
    | RETURN ';' { emit("xor eax, eax"); emit("leave"); emit("ret"); }
    | RETURN '(' rvalue ')' ';' { emit("leave"); emit("ret"); }
    | rvalue ';'
    | ';'
    ;

if_start
    : IF '(' rvalue ')' {
        push_context();
        get_top_context()->end_label = get_jump_label();
        emit("cmp eax, 0");
        emit("je .I%d", get_top_context()->end_label);
    }
    ;

extrn_names
    : NAME { add_extrn_symbol($1); }
    | extrn_names ',' NAME { add_extrn_symbol($3); }
    ;

auto_name
    : NAME { add_auto_symbol($1); emit("push 0"); }
    | NAME constant { add_auto_symbol($1); emit("push eax"); }
    ;

auto_names
    : auto_name
    | auto_names ',' auto_name
    ;

statements
    : statement
    | statements statement
    ;

lvalue
    : '*' postfix
    | postfix '[' {
        emit("push eax");
    } rvalue ']' {
        emit("pop ebx");
        emit("shl eax, 2");
        emit("add eax, ebx");
    }
    | NAME {
        t_symbol *sym = get_symbol($1);
        if (!sym || sym->type == EXTRN_S)
            emit("lea eax, \"%s\"", sym->name);
        else if (((t_symbol_stack *)sym)->stack_offset > 0)
            emit("lea eax, [ebp - %d]",((t_symbol_stack *) sym)->stack_offset);
        else
            emit("lea eax, [ebp + %d]", -((t_symbol_stack *)sym)->stack_offset);
        free($1);
    }
    ;

rvalue
    : unary
    | binary
    | comparison
    | ternary
    | assignation
    ;

unary
    : '&' lvalue %prec UADDR
    | INC lvalue { pre("inc"); }
    | DEC lvalue { pre("dec"); }
    | '-' rvalue %prec USUB { emit("not eax"); emit("inc eax"); }
    | F_SUB rvalue %prec UF_SUB { emit("xor eax, 0x80000000"); }
    | '!' rvalue { emit("push 0"); compare("sete"); }
    | '~' rvalue { emit("NOT eax"); }
    | TO_FLOAT rvalue { to_float(); }
    | TO_INT   rvalue { to_int(); }
    | postfix
    ;

postfix
    : lvalue INC { post("inc"); }
    | lvalue DEC { post("dec"); }
    | lvalue { emit("mov eax, [eax]"); }
    | '(' rvalue ')'
    | constant
    | fncall
    ;

binary
    : rvalue '*'   { emit("push eax"); } rvalue { op_mul(); }
    | rvalue '/'   { emit("push eax"); } rvalue { op_div(); }
    | rvalue '%'   { emit("push eax"); } rvalue { op_mod(); }
    | rvalue '+'   { emit("push eax"); } rvalue { op("add"); }
    | rvalue '-'   { emit("push eax"); } rvalue { op("sub"); }
    | rvalue F_MUL { emit("push eax"); } rvalue { opf("mulss"); }
    | rvalue F_DIV { emit("push eax"); } rvalue { opf("divss"); }
    | rvalue F_ADD { emit("push eax"); } rvalue { opf("addss"); }
    | rvalue F_SUB { emit("push eax"); } rvalue { opf("subss"); }
    | rvalue RIGHT { emit("push eax"); } rvalue { shift("shr"); }
    | rvalue LEFT  { emit("push eax"); } rvalue { shift("shl"); }
    | rvalue '&'   { emit("push eax"); } rvalue { op("and"); }
    | rvalue '|'   { emit("push eax"); } rvalue { op("or"); }
    ;

ternary
    : rvalue '?' {
        push_context();
        get_top_context()->end_label = get_jump_label();
        emit("cmp eax, 0");
        emit("je .I%d", get_top_context()->end_label);
    }  rvalue ':' {
        get_top_context()->start_label = get_jump_label();
        emit("jmp .I%d", get_top_context()->start_label);
        emit(".I%d:", get_top_context()->end_label);
    } rvalue {
        emit(".I%d:", get_top_context()->start_label);
        pop_context();
    }
    ;

comparison
    : rvalue '>'                { emit("push eax"); } rvalue { compare("setg"); }
    | rvalue '<'                { emit("push eax"); } rvalue { compare("setl"); }
    | rvalue EQ                 { emit("push eax"); } rvalue { compare("sete"); }
    | rvalue NEQ                { emit("push eax"); } rvalue { compare("setne"); }
    | rvalue LOWER_EQ           { emit("push eax"); } rvalue { compare("setle"); }
    | rvalue GREATER_EQ         { emit("push eax"); } rvalue { compare("setge"); }
    ;

assignation
    : lvalue '='                { emit("push eax"); } rvalue { emit("pop ebx"); emit("mov [ebx], eax"); }
    | lvalue BITOR_ASSIGN       { emit("push eax"); } rvalue { op_assign("or"); }
    | lvalue BITAND_ASSIGN      { emit("push eax"); } rvalue { op_assign("and"); }
    | lvalue ADD_ASSIGN         { emit("push eax"); } rvalue { op_assign("add"); }
    | lvalue SUB_ASSIGN         { emit("push eax"); } rvalue { op_assign("sub"); }
    | lvalue MUL_ASSIGN         { emit("push eax"); } rvalue { mul_assign(); }
    | lvalue DIV_ASSIGN         { emit("push eax"); } rvalue { div_assign(); }
    | lvalue MOD_ASSIGN         { emit("push eax"); } rvalue { mod_assign(); }
    | lvalue RIGHT_ASSIGN       { emit("push eax"); } rvalue { shift_assign("shr"); }
    | lvalue LEFT_ASSIGN        { emit("push eax"); } rvalue { shift_assign("shl"); }
    | lvalue LOWER_ASSIGN       { emit("push eax"); } rvalue { compare_assign("setl"); }
    | lvalue LOWER_EQ_ASSIGN    { emit("push eax"); } rvalue { compare_assign("setle"); }
    | lvalue GREATER_ASSIGN     { emit("push eax"); } rvalue { compare_assign("setg"); }
    | lvalue GREATER_EQ_ASSIGN  { emit("push eax"); } rvalue { compare_assign("setge"); }
    | lvalue EQ_ASSIGN          { emit("push eax"); } rvalue { compare_assign("sete"); }
    | lvalue NEQ_ASSIGN         { emit("push eax"); } rvalue { compare_assign("setne"); }
    ;

fncall
    : postfix '(' { push_call(); } fnparams ')' { pop_call(); }
    | postfix '(' { push_call(); }          ')' { pop_call(); }
    ;

fnparams
    : rvalue { add_arg(); }
    | fnparams ',' rvalue { add_arg(); }
    ;

%%


int main() {

    emit(".intel_syntax noprefix");
    emit(".text");

        yyparse();
    pop_context();
    return 0;
}

void yyerror(const char* msg) {
    error("%s", msg);
}


static int symcmp(const void *t1, const void *t2) {
	return strcmp(((t_symbol *)t1)->name, ((t_symbol *)t2)->name);
}

static t_symbol_stack *new_auto_sym(char *name, void **syms, int pos) {
	t_symbol_stack *sym;
	sym = calloc(1, sizeof(t_symbol_stack));
	if (!sym)
		error("auto symbol '%s' calloc failed", name);
	sym->type = STACK_S;
	sym->stack_offset = pos;
	sym->name = name;
	if (tfind(sym, syms, symcmp) != NULL)
		error("auto redefinition of '%s'", sym->name);
	tsearch(sym, syms, symcmp);
	return sym;
}

static t_symbol_global *new_extrn_sym(char *name, void **syms) {
	t_symbol_global *sym;
	sym = calloc(1, sizeof(t_symbol_global));
	if (!sym)
		error("extrn symbol '%s' calloc failed", name);
	sym->type = EXTRN_S;
	sym->name = name;
	if (tfind(sym, syms, symcmp) != NULL)
		error("extrn redefinition of '%s'", sym->name);
	tsearch(sym, syms, symcmp);
	return sym;
}

char *add_extrn_symbol(char *name) {
	t_context *ctx = get_top_context();
	return new_extrn_sym(name, &ctx->symbols)->name;
}

char *add_arg_symbol(char *name, int pos) {
	t_context *ctx = get_fn_context();
	return new_auto_sym(name, &ctx->symbols, pos)->name;
}

char *add_auto_symbol(char *name) {
	t_context *ctx = get_top_context();
	return new_auto_sym(name, &ctx->symbols, ++(get_fn_context()->frame_size) * 4)->name;
}

static t_symbol *get_symbol_in(char *name, void **(*key)(t_context *)) {
	t_symbol **ret_sym;
	t_symbol sym = {.name = name};

	for (int i = get_context_len() - 1; i >= 0; --i) {
		ret_sym = (t_symbol **)tfind(&sym, key(get_context(i)), symcmp);
		if (ret_sym)
			return *ret_sym;
	}
	return NULL;
}

void **symbols(t_context *ctx) {
	return &ctx->symbols;
}

t_symbol *get_symbol(char *name) {
	t_symbol *sym;

	if ((sym = get_symbol_in(name, symbols)) != NULL)
		return sym;
	return NULL;
}

void emit(const char *fmt, ...) {
	va_list ap;
	va_start(ap, fmt);
	vprintf(fmt, ap);
	va_end(ap);
	printf("\n");
}

void emit_raw(const char *fmt, ...) {
	va_list ap;
	va_start(ap, fmt);
	vprintf(fmt, ap);
	va_end(ap);
}

void error(const char *fmt, ...) {
	va_list ap;
	va_start(ap, fmt);
	fflush(stderr);
	fflush(stdout);
	// fprintf(stderr, "line: %d: ", yylineno);
	vfprintf(stderr, fmt, ap);
	fprintf(stderr, "\n");
	va_end(ap);
	exit(4);
}

void to_float() {
	emit("cvtsi2ss xmm0, eax");
	emit("movd eax, xmm0");
}

void to_int() {
	emit("movd xmm0, eax");
	emit("cvttss2si eax, xmm0");
}

void opf(const char *op) {
	emit("movd xmm1, eax");
	emit("pop eax");
	emit("movd xmm0, eax");
	emit("%s xmm0, xmm1", op);
	emit("movd eax, xmm0");
}


int get_constant_label() {
	static int counter = 0;
	return counter++;
}

int get_jump_label() {
	static int counter = 1;
	return counter++;
}


void post(const char *instruction) {
	emit("mov ebx, [eax]");
	emit("mov ecx, ebx");
	emit("%s ebx", instruction);
	emit("mov [eax], ebx");
	emit("mov eax, ecx");
}

void pre(const char *instruction) {
	emit("%s DWORD PTR [eax]", instruction);
	emit("mov eax, [eax]");
}

void op_assign(const char *instruction) {
	emit("mov ebx, eax");
	emit("pop eax");
	emit("%s [eax], ebx", instruction);
	emit("mov eax, [eax]");
}

void compare_assign(const char *set) {
	emit("pop ebx");
	emit("cmp [ebx], eax");
	emit("%s al", set);
	emit("movzx eax, al");
	emit("mov [ebx], eax");
}

void shift_assign(const char *instruction) {
	emit("mov ecx, eax");
	emit("pop eax");
	emit("%s DWORD PTR [eax], cl", instruction);
	emit("mov eax, [eax]");
}

void mod_assign() {
	emit("mov ebx, eax");
	emit("pop ecx");
	emit("mov eax, [ecx]");
	emit("cdq");
	emit("idiv ebx");
	emit("mov [ecx], edx");
	emit("mov eax, edx");
}

void div_assign() {
	emit("mov ebx, eax");
	emit("pop ecx");
	emit("mov eax, [ecx]");
	emit("cdq");
	emit("idiv ebx");
	emit("mov [ecx], eax");
}

void mul_assign() {
	emit("mov ebx, eax");
	emit("pop ecx");
	emit("mov eax, [ecx]");
	emit("imul ebx");
	emit("mov [ecx], eax");
}
int get_switch_counter() {
	static int count = 3;
	return count++;
}

void switch_start() {
	push_context();
	switch_depth++;
	get_top_context()->end_label = get_switch_counter();
	emit("push eax\n");
}

void case_start(int cst) {
	int switch_label = (get_top_context() - 1)->end_label;
	int case_label = (get_top_context())->start_label;
	if (switch_depth == 0)
		error("case outside of switch");
	emit("case_%d_if_%d:", switch_label, case_label);
	emit("pop eax");
	emit("cmp eax, %d", cst);
	emit("push eax");
	emit("jne case_%d_if_%d", switch_label, case_label + 1);
	emit("case_%d_%d:", switch_label, case_label);
}

void case_end() {
	int switch_label = (get_top_context() - 1)->end_label;
	get_top_context()->start_label++;
	emit("jmp case_%d_%d", switch_label, get_top_context()->start_label);
	last_case = get_top_context()->start_label;
}

void switch_end() {
	int switch_label = get_top_context()->end_label;

	emit("case_%d_if_%d:", switch_label, last_case);
	emit("case_%d_%d:", switch_label, last_case);
	switch_depth--;
	pop_context();
}


void compare(const char *set) {
	emit("pop ebx");
	emit("cmp ebx, eax");
	emit("%s al", set);
	emit("movzx eax, al");
}


static t_context context_stack[MAX_CONTEXT_LEN] = {0};
static size_t stack_len = 1;

t_context *get_fn_context() {
	return context_stack + 1;
}

char *get_fn_name() {
	return context_stack[1].name;
}

t_context *get_top_context() {
	return context_stack + stack_len - 1;
}

size_t get_context_len() {
	return stack_len;
}

t_context *get_context(size_t i) {
	return context_stack + i;
}

void push_context() {
	if (stack_len >= MAX_CONTEXT_LEN)
		error("context stack overflow");
	memset(context_stack + stack_len, 0, sizeof(t_context));
	++stack_len;
}

static void free_sym(void *p) {
	t_symbol *sym = p;
	free((void *)sym->name);
	free(sym);
}

void pop_context() {
	if (stack_len <= 0)
		error("context stack underflow");
	--stack_len;
	free(context_stack[stack_len].name);
	tdestroy(context_stack[stack_len].symbols, free_sym);
	context_stack[stack_len].symbols = NULL;
}

void op(const char *instruction) {
	emit("mov ebx, eax");
	emit("pop eax");
	emit("%s eax, ebx", instruction);
}

void op_mul() {
	emit("mov ebx, eax");
	emit("pop eax");
	emit("imul ebx");
}

void op_div() {
	emit("mov ebx, eax");
	emit("pop eax");
	emit("cdq");
	emit("idiv ebx");
}

void op_mod() {
	op_div();
	emit("mov eax, edx");
}

void shift(const char *instruction) {
	emit("mov ecx, eax");
	emit("pop eax");
	emit("%s eax, cl", instruction);
}

void reverse_stack(int len) {
	for (int i = 0; i < len / 2; i++) {
		emit("mov ebx, [esp+%d]", i * 4);
		emit("mov ecx, [esp+%d]", (len - i - 1) * 4);
		emit("mov [esp+%d], ebx", (len - i - 1) * 4);
		emit("mov [esp+%d], ecx", i * 4);
	}
}

void push_call() {
	if (call_stack_len >= MAX_CALL_STACK_SIZE)
		error("call stack overflow");
	call_stack[call_stack_len] = 0;
	++call_stack_len;

	emit("push eax");
}

void add_arg() {
	emit("push eax");
	call_stack[call_stack_len - 1]++;
}

int get_arg_count() {
	return call_stack[call_stack_len - 1];
}

void pop_call() {
	int arg_count;

	if (call_stack_len == 0)
		error("call stack underflow");

	arg_count = get_arg_count();
	if (arg_count > 0)
		reverse_stack(arg_count + 1);
	emit("pop eax");
	emit("call eax");
	if (arg_count > 0)
		emit("add esp, %d", arg_count * 4);
	--call_stack_len;
}

void fn_decl_enter(char *fn_name)
{
	arg_counter = 2;
	get_fn_context()->name = strdup(fn_name);
	add_extrn_symbol(fn_name);
	push_context();
	emit(".text");
	emit(".globl %s", fn_name);
	emit("%s:", fn_name);
	emit(".long \"%s\" + 4", fn_name);
	emit("enter 0, 0");
}

void argument(char *name)
{
	add_arg_symbol(name, -arg_counter * 4);
	++arg_counter;
}

void global_int(char *name, int value)
{
	add_extrn_symbol(name);
	emit(".section .data");
	emit(".globl %s", name);
	emit("%s: .long %d", name, value);
}

void global_string(char *name, const char *value)
{
	add_extrn_symbol(name);
	emit(".section .rodata");
	emit(".globl %s", name);
	emit("%s: .long %s_str", name, name);
	emit("%s_str: .string %s", name, value);
}

void global_vector(char *name)
{
	add_extrn_symbol(name);
	emit(".section .data");
	emit(".globl %s", name);
	emit("%s: .long %s_vec", name, name);
	emit_raw("%s_vec: .long ", name);
}

void global_vector_extra(int count, int start)
{
	if (count <= 0)
		return;
	if (start)
	{
		emit_raw("0");
		count--;
	}
	while (count-- > 0)
		emit_raw(", 0");
}
