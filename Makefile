NAME = target/debug/cc1

FT_LEX = ft_lex/target/release/ft_lex
LEX_FILE = src/c.l
LEXER = src/lex_yy.rs

FT_YACC = ft_yacc/target/release/ft_yacc
YACC_FILE = src/c.y
PARSER = src/c_tab.rs

# ----- cc1 --------------------

all: $(NAME)

$(NAME): $(LEXER) $(PARSER)
	cargo build

# ----- ft_lex --------------------

$(FT_LEX):
	$(MAKE) -C ft_lex

lexer: $(LEXER)
$(LEXER): $(FT_LEX) $(LEX_FILE)
	$(FT_LEX) -cx rust $(LEX_FILE) -o $(LEXER)

# ----- ft_yacc --------------------

$(FT_YACC):
	$(MAKE) -C ft_yacc

parser: $(PARSER)
$(PARSER): $(FT_YACC) $(YACC_FILE)
	$(FT_YACC) -x rust $(YACC_FILE) -b src/c -v

# ----- test --------------------

test: $(NAME)
	echo ";" | ./$(NAME)


clean:
	cargo clean
	rm -rf $(LEXER) $(PARSER)
	make -C ft_lex clean
	make -C ft_yacc clean

re: clean all

.PHONY: all clean re test lexer parser
