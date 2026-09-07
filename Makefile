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
	clang -E -std=c89 rscs/hello.c > rscs/hello.i
	./$(NAME) -m32 rscs/hello.i

ctest: $(NAME)
	cargo nextest run -p cc1

ttest:
	cargo build --release -p ft_lex -p ft_yacc
	cargo nextest run -p ft_lex -p ft_yacc -p libft -p cc1


CFF = -m32 -std=iso9899:1990 -pedantic-errors -Wno-deprecated-non-prototype -Wno-strict-prototypes -fno-asm -fno-builtin
# CFF = -m32 -std=iso9899:1990

c:
	gcc -c $(CFF) rscs/hello.c -o /dev/null

cc:
	gcc $(CFF) rscs/hello.c
	./a.out
	rm ./a.out

empty :=
space := $(empty) $(empty)

COV_SKIP = \
	src/main.rs \
	src/report.rs \
	src/ast/mod.rs \
	src/ast/name.rs \
	src/pipeline.rs \
	src/ast/print.rs \
	src/parser/lex.rs \
	src/utils/table.rs \
	src/parser/yacc.rs \
	src/ast/display.rs \
	src/parser/span.rs \
	src/parser/driver.rs \
	src/ast/type_specifier.rs \
	src/semantic/diagnosis.rs

coverage: $(NAME)
	cargo llvm-cov nextest --ignore-filename-regex '$(subst $(space),|,$(strip $(COV_SKIP)))'

clean:
	cargo clean -p cc1
	cargo clean -p ft_lex
	cargo clean -p ft_yacc
	cargo clean -p libft
	rm -rf $(LEXER) $(PARSER)

re: clean all

.PHONY: all clean re test ctest ttest lexer parser $(FT_LEX) $(FT_YACC) $(NAME)
