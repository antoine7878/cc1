#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
	int state;
	char *buf_pos;
} yy_accept_data;

#define YY_READ_LEN 8192

#define YY_STACK_SIZE 8192
/* DEFINES */

// Globals
#ifdef YY_ARRAY
char yytext[YY_READ_LEN] = {0};
#else
char *yytext = NULL; /* matched token */
#endif

FILE *yyin = NULL;	/* input file */
FILE *yyout = NULL; /* output file */
size_t yyleng = 0;	/* matched token len */

int yylex(void);
int yymore(void);
int yyless(int n);
int input(void);
int unput(int c);
int yywrap(void);
int main(int argc, char *argv[]);

static int yy_act;								   /* action to execute */
static int yy_current_state;					   /* current dfa state */
static yy_accept_data yy_state_buf[YY_STACK_SIZE]; /* stack of visited accepting states */
static size_t yy_state_buf_len = 0;				   /* len of the state buffer */
static int yy_start_condition = 0;				   /* intial start start */
static char *yy_buf = NULL;						   /* input buffer */
static size_t yy_buf_len = 0;					   /* input buffer len */
static char *yy_buf_pos = NULL;					   /* current position in the input buffer */
static char *yy_run_pos = NULL;					   /* running position in the input buffer */

static char yy_stash = '\0';			   /* char stash for yytext termination */
static int yy_buf_empty = 0;			   /* set to 1 when the last byte of yy_buf is read */
static char *yy_last_trailing_buf_pos = 0; /* buffer position as last state accepting a tail */
static int yy_last_trailing_fragment = -1; /* last trailing tag seen */
static int yy_bol = 1;					   /* 1 if the yy_buf_pos is right after a '\n' */
static int yy_ismore = 0;				   /* flag for yy_more */
#ifndef ARRAY
static int yy_buf_alloc_count = 0; /* Number of time yy_buf id alloacted */
#endif

#ifndef ECHO
#define ECHO fprintf(yyout, "%s", yytext)
#endif

#define REJECT                                                                                     \
	{                                                                                              \
		*(yy_state_buf_top()->buf_pos + 1) = yy_stash;                                             \
		int n = yy_next_accept[yy_state_buf_top()->state * YY_RULE_COUNT + yy_act];                \
		if (n >= 0) {                                                                              \
			yy_act = n;                                                                            \
		} else {                                                                                   \
			yy_state_buf_pop();                                                                    \
			yy_act = yy_accept[yy_state_buf_top()->state];                                         \
		}                                                                                          \
		goto action;                                                                               \
	}

#define BEGIN yy_start_condition =

static void yy_error(const char *msg) {
	fprintf(stderr, "fatal ft_lex: %s\n", msg);
}

/* TABLES */

/* REMOVE */
static const int yy_char_eq[0] = {};
static const int yy_base[0] = {};
static const int yy_start[0] = {};
static const int yy_trailling[0] = {};
static const int yy_accept[0] = {};
static const int yy_next_accept[0] = {};

#define YY_CLASS_COUNT 42

/* REMOVE */

static yy_accept_data *yy_state_buf_top() {
	return &yy_state_buf[yy_state_buf_len - 1];
}

static yy_accept_data *yy_state_buf_pop() {
	yy_accept_data *top = yy_state_buf_top();
	yy_state_buf_len--;
	return top;
}

static void yy_state_buf_push(int state, char *run_pos) {
	yy_accept_data *top = yy_state_buf_top() + 1;
	yy_state_buf_len++;
	top->state = state;
	top->buf_pos = run_pos;
}

static void yy_state_buf_clear() {
	yy_state_buf_len = 1;
	yy_state_buf[0].state = 0;
	yy_state_buf[0].buf_pos = yy_buf_pos;
}

static void yy_state_buf_shift(size_t offset) {
	if (yy_last_trailing_buf_pos)
		yy_last_trailing_buf_pos += offset;
	for (size_t i = 0; i < yy_state_buf_len; i++) {
		yy_state_buf[i].buf_pos += offset;
	}
}

// INPUT BUFFER
static void yy_buffer_load(char **run_pos) {
	size_t bytes_to_keep = *run_pos - yy_buf_pos;
	if (bytes_to_keep > 0)
		memmove(yy_buf, yy_buf_pos, bytes_to_keep);

	*run_pos += (yy_buf - yy_buf_pos);
	yy_state_buf_shift(yy_buf - yy_buf_pos);

	yy_buf_pos = yy_buf;
	size_t read_try_len = yy_buf_len - bytes_to_keep;
	if (read_try_len == 0) {
#ifdef YY_ARRAY
		yy_error("Token longer than array buffer");
#else
		yy_buf_alloc_count++;
		yy_buf_len = YY_READ_LEN * yy_buf_alloc_count;
		char *old_buf = yy_buf;
		yy_buf = realloc(yy_buf, yy_buf_len + 2);
		read_try_len = yy_buf_len - bytes_to_keep;
		size_t offset = yy_buf - old_buf;
		yy_buf_pos += offset;
		*run_pos += offset;
		yy_state_buf_shift(offset);
#endif
	}

	size_t read_len;
	while (1) {
		read_len = fread(yy_buf + bytes_to_keep, 1, read_try_len, yyin);
		if (read_len == read_try_len)
			break;
		if (yywrap() != 0) {
			yy_buf_empty = 1;
			yy_buf_len = read_len + bytes_to_keep;
			break;
		}
		read_try_len -= read_len;
		bytes_to_keep += read_len;
	}
	yy_buf[yy_buf_len] = '\0';
	yy_buf[yy_buf_len + 1] = '\0';
}

int yymore() {
	yy_ismore = 1;
	return 0;
}

void yy_terminated_text() {
	yy_stash = *yy_run_pos;
	*yy_run_pos = '\0';
	yyleng = yy_run_pos - yy_buf_pos;
#ifdef YY_ARRAY
	memmove(yytext, yy_buf_pos, yyleng + 1);
#else
	yytext = yy_buf_pos;
#endif
}

int yyless(int n) {
	*yy_run_pos = yy_stash;
	yy_run_pos -= yyleng - n;
	yy_terminated_text();
	return 0;
}

int input() {
	yy_run_pos++;
	if (*yy_run_pos == '\0')
		yy_buffer_load(&yy_run_pos);
	char c = yy_stash;
	yy_stash = *yy_run_pos;
	return c;
}

int unput(int c) {
	yy_run_pos--;
	*yy_run_pos = c;
	yy_stash = c;
	return 0;
}

static void yy_try_accept(char *run_pos, int state) {
	if (yy_trailling[state] >= 0) {
		yy_last_trailing_fragment = yy_trailling[state];
		yy_last_trailing_buf_pos = run_pos;
	}
	if (yy_accept[state] < 0)
		return;
	yy_state_buf_push(state, run_pos);
}

// YYLEX
static void yy_init_lex() {
	if (!yyin)
		yyin = stdin;
	if (!yyout)
		yyout = stdout;

	if (yy_buf_len > 1)
		return;
	yy_buf = malloc(1);
	*yy_buf = 0;

	yy_buf_pos = yy_buf;
	yy_run_pos = yy_buf;
}

int yylex(void) {
	int yy_current_state;
	int eq_class;

	yy_init_lex();

	while (1) {
		if (yy_stash)
			*yy_run_pos = yy_stash;
		if (yy_stash == 0 && yy_buf_empty == 1)
			goto retlab;
		if (yy_ismore == 0) {
			yy_buf_pos = yy_run_pos;
			yy_bol = yy_buf_pos == yy_buf || *(yy_buf_pos - 1) == '\n';
		} else {
			yy_ismore = 0;
		}

		yy_state_buf_clear();
		yy_last_trailing_buf_pos = NULL;
		yy_last_trailing_fragment = -1;
		yy_current_state = yy_start[yy_start_condition + yy_bol];

		while (1) {
			if (*yy_run_pos == '\0')
				yy_buffer_load(&yy_run_pos);

			eq_class = yy_char_eq[*yy_run_pos];
			yy_current_state = yy_base[yy_current_state * YY_CLASS_COUNT + eq_class];
			if (yy_current_state == 0) {
				break;
			}
			yy_try_accept(yy_run_pos, yy_current_state);
			++yy_run_pos;
		}
		yy_act = yy_accept[yy_state_buf_top()->state];
	action:

		if (yy_last_trailing_buf_pos != NULL &
			yy_accept[yy_state_buf_top()->state] == yy_last_trailing_fragment) {
			yy_state_buf_top()->buf_pos = yy_last_trailing_buf_pos;
		}

		yy_run_pos = yy_state_buf_top()->buf_pos + 1;
		yy_terminated_text();

		switch (yy_act) {

		case -1:
			ECHO;
			break;

			/* ACTIONS */

		default:
			yy_error("scanner internal error: action not found");
			break;
		}
	}
retlab:
#ifndef YY_ARRAY
	free(yy_buf);
#endif
	return 0;
}
