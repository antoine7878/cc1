# ----- cc1 --------------------

NAME = cc1

all: $(NAME)

$(NAME): $(LEXER) $(PARSER)
	cargo build

# ----- ft_lex --------------------

FT_LEX = ft_lex/target/release/ft_lex
LEX_FILE = src/c.l
LEXER = lex_yy.rs

$(FT_LEX):
	$(MAKE) -C ft_lex

$(LEXER): $(FT_LEX) $(LEX_FILE)
	$(FT_LEX) -cx rust $(LEX_FILE)

# ----- ft_yacc --------------------

FT_YACC = ft_yacc/target/release/ft_yacc
YACC_FILE = src/c.y
PARSER = c_tab.rs

$(FT_YACC):
	$(MAKE) -C ft_yacc

$(PARSER): $(FT_YACC) $(YACC_FILE)
	$(FT_YACC) -dx rust $(YACC_FILE)

# ----- test --------------------

test:
	echo "2+3*4"

clean:
	cargo clean
	make -C ft_lex clean
	make -C ft_yacc clean

re: clean all

.PHONY: all clean re test
