#pragma once

/* YYSTYPE_INT */

/* DEBUGGING */
#ifndef YYDEBUG
#define YYDEBUG 0
#endif
/* DEBUGGING */

#ifndef YYDEBUG
#define YYDEBUG 1
#endif

#if YYDEBUG
extern int yydebug;
#endif

typedef enum yytokentype {
	YYEMPTY = -2,
	/* TOKENS */
} yytoken_kind_t;

#ifdef YYSTYPE_INT
typedef int YYSTYPE;

#else
typedef union YYSTYPE {
	/* UNION */
} YYSTYPE;
#endif

extern YYSTYPE yylval;

int yyparse(void);
