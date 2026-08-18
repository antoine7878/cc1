NAME = target/debug/cc1

FT_LEX = ft_lex/target/release/ft_lex
LEX_FILE = src/parser/c.l
LEXER = src/parser/lex.rs

FT_YACC = ft_yacc/target/release/ft_yacc
YACC_FILE = src/parser/c.y
PARSER = src/parser/yacc.rs

# ----- cc1 --------------------

all: $(NAME)

$(NAME):
	cargo build

# ----- ft_lex --------------------

$(FT_LEX):
	$(MAKE) -C ft_lex

$(LEXER): $(LEX_FILE)
	$(FT_LEX) -cx rust $(LEX_FILE) -o $(LEXER)

# ----- ft_yacc --------------------

$(FT_YACC):
	$(MAKE) -C ft_yacc

$(PARSER): $(YACC_FILE)
	$(FT_YACC) -x rust $(YACC_FILE) -o $(PARSER)

# ----- test --------------------

test: $(NAME)
	./$(NAME) test/hello.c

ast:
	clang -std=iso9899:1990 -pedantic -Xclang -ast-dump test/hello.c

c:
	clang -std=iso9899:1990 -pedantic test/hello.c

clean:
	cargo clean
	rm -rf $(LEXER) $(PARSER)
	make -C ft_lex clean
	make -C ft_yacc clean

re: clean all

.PHONY: all clean re test lexer parser $(FT_LEX) $(FT_YACC) $(NAME)
