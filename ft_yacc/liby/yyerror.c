#include <stdio.h>

void yyerror(const char *msg) {
	fprintf(stderr, "%s\n", msg);
}
