NAME = target/debug/cc1

FT_LEX = ft_lex/target/release/ft_lex
LEX_FILE = src/parser/c.l
LEXER = src/parser/lex.rs

FT_YACC = ft_yacc/target/release/ft_yacc
YACC_FILE = src/parser/c.y
PARSER = src/parser/yacc.rs

# ----- cc1 --------------------

all: $(NAME)

$(NAME): $(LEXER) $(PARSER)
	cargo build

# ----- ft_lex --------------------

$(FT_LEX):
	$(MAKE) -C ft_lex

$(LEXER): $(LEX_FILE) | $(FT_LEX)
	$(FT_LEX) -cx rust $(LEX_FILE) -o $(LEXER)

# ----- ft_yacc --------------------

$(FT_YACC):
	$(MAKE) -C ft_yacc

$(PARSER): $(YACC_FILE) | $(FT_YACC)
	$(FT_YACC) -x rust $(YACC_FILE) -o $(PARSER)

# ----- test --------------------

test: $(NAME)
	clang -E -std=c89 test/hello.c > test/hello.i
	./$(NAME) test/hello.i

ctest: $(NAME)
	cargo nextest run

CCF = -std=iso9899:1990 -pedantic-errors -Wno-deprecated-non-prototype -Wno-strict-prototypes -fno-asm -fno-builtin

c:
	gcc -c $(CCF) test/hello.c -o /dev/null

cc:
	gcc $(CCF) test/hello.c
	./a.out
	rm ./a.out

clean:
	cargo clean
	rm -rf $(LEXER) $(PARSER)
	make -C ft_lex clean
	make -C ft_yacc clean

re: clean all

.PHONY: all clean re test lexer parser $(FT_LEX) $(FT_YACC) $(NAME)
