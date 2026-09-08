NAME = target/debug/cc1

FT_LEX  = target/release/ft_lex
FT_YACC = target/release/ft_yacc

GEN_CRATES = cc1
GEN = $(foreach c,$(GEN_CRATES),crates/$(c)/src/parser/lex.rs crates/$(c)/src/parser/yacc.rs)

# ----- cc1 --------------------

all: $(NAME)

$(NAME): $(GEN)
	cargo build

# ----- ft_lex / ft_yacc --------------------

$(FT_LEX) $(FT_YACC):
	cargo build --release -p ft_lex -p ft_yacc

crates/%/src/parser/lex.rs: crates/%/src/parser/c.l | $(FT_LEX)
	$(FT_LEX) -c $< -o $@

crates/%/src/parser/yacc.rs: crates/%/src/parser/c.y | $(FT_YACC)
	$(FT_YACC) $< -o $@

# ----- test --------------------

test: $(NAME)
	clang -E -std=c89 rscs/hello.c > rscs/hello.i
	./$(NAME) -m32 rscs/hello.i

ctest: $(NAME)
	cargo nextest run -p cc1

ttest:
	cargo build --release -p ft_lex -p ft_yacc
	cargo nextest run

CFF = -m32 -std=iso9899:1990

c:
	gcc -c $(CFF) rscs/hello.c -o /dev/null

cc:
	gcc $(CFF) rscs/hello.c
	./a.out
	rm ./a.out

empty :=
space := $(empty) $(empty)

COV_SKIP = \
	crates/cc1/src/main.rs \
	crates/cc1/src/report.rs \
	crates/cc1/src/ast/mod.rs \
	crates/cc1/src/ast/name.rs \
	crates/cc1/src/pipeline.rs \
	crates/cc1/src/ast/print.rs \
	crates/cc1/src/parser/lex.rs \
	crates/cc1/src/utils/table.rs \
	crates/cc1/src/parser/yacc.rs \
	crates/cc1/src/ast/display.rs \
	crates/cc1/src/parser/span.rs \
	crates/cc1/src/parser/driver.rs \
	crates/cc1/src/ast/type_specifier.rs \
	crates/cc1/src/semantic/diagnosis.rs

coverage: $(NAME)
	cargo llvm-cov nextest --ignore-filename-regex '$(subst $(space),|,$(strip $(COV_SKIP)))'

clean:
	cargo clean
	rm -f $(GEN)

re: clean all

.PHONY: all clean re test ctest ttest c cc coverage $(FT_LEX) $(FT_YACC) $(NAME)
