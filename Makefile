NAME = target/debug/cc1

FT_LEX = target/release/ft_lex
LEX_FILE = src/parser/c.l
LEXER = src/parser/lex.rs

FT_YACC = target/release/ft_yacc
YACC_FILE = src/parser/c.y
PARSER = src/parser/yacc.rs

# ----- cc1 --------------------

all: $(NAME)

$(NAME): $(LEXER) $(PARSER)
	cargo build

# ----- ft_lex / ft_yacc --------------------

$(FT_LEX) $(FT_YACC):
	cargo build --release -p ft_lex -p ft_yacc

$(LEXER): $(LEX_FILE) | $(FT_LEX)
	$(FT_LEX) -c $(LEX_FILE) -o $(LEXER)

$(PARSER): $(YACC_FILE) | $(FT_YACC)
	$(FT_YACC) $(YACC_FILE) -o $(PARSER)

# ----- test --------------------

test: $(NAME)
	clang -E -std=c89 test/hello.c > test/hello.i
	./$(NAME) test/hello.i

ctest: $(NAME)
	cargo nextest run -p cc1

ttest:
	cargo build --release -p ft_lex -p ft_yacc
	cargo nextest run -p ft_lex -p ft_yacc -p libft

coverage: $(NAME)
	cargo llvm-cov nextest

CFF = -std=iso9899:1990 -pedantic-errors -Wno-deprecated-non-prototype -Wno-strict-prototypes -fno-asm -fno-builtin
# CFF = -pedantic-errors

c:
	gcc -c $(CFF) test/hello.c -o /dev/null

cc:
	gcc $(CFF) test/hello.c
	./a.out
	rm ./a.out

clean:
	cargo clean
	rm -rf $(LEXER) $(PARSER)

re: clean all

.PHONY: all clean re test ttest lexer parser $(FT_LEX) $(FT_YACC) $(NAME)
