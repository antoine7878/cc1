CC1 = target/debug/cc1
FCC = target/debug/fcc

FT_LEX  = target/release/ft_lex
FT_YACC = target/release/ft_yacc

C_L = crates/cc1/src/parser/c.l
C_Y = crates/cc1/src/parser/c.y

LEX_RS =  crates/cc1/src/parser/lex.rs
YACC_RS = crates/cc1/src/parser/yacc.rs

all: $(LEX_RS) $(YACC_RS)
	cargo build -p cpp -p cc1 -p fcc

$(FT_LEX) $(FT_YACC):
	cargo build --release -p ft_lex -p ft_yacc

$(LEX_RS): $(C_L) | $(FT_LEX)
	$(FT_LEX) -c $< -o $@

$(YACC_RS): $(C_Y)
	$(FT_YACC) $< -o $@

# ----- test --------------------

# test: all
# 	clang -E -std=c89 rscs/hello.c > rscs/hello.i
# 	./$(CC1) -m32 rscs/hello.i

test: all
	rm -f ./hello.ll ./hello.s ./hello.o ./a.out
	cargo run --bin fcc -- -e ./rscs/hello.c -o /dev/stdout
	@# ./a.out || echo $$?

otest: all
	rm -f ./hello.ll ./hello.s ./hello.o ./a.out
	cargo run --bin fcc -- -e ./rscs/hello.c  -o ./rscs/hello.ll
	opt -passes=verify -S ./rscs/hello.ll

ftest: all
	rm -f ./hello.ll ./hello.s ./hello.o ./a.out
	cargo run --bin fcc -- ./rscs/hello.c -o ./rscs/a.out
	./rscs/a.out || echo $$?

ctest: all
	cargo nextest run -p cc1

ttest: all
	cargo nextest run

# ----- reference --------------------

CFF = -m32 -std=iso9899:1990 -pedantic-errors

c:
	gcc -c $(CFF) rscs/hello.c -o /dev/null

cc:
	gcc $(CFF) rscs/hello.c
	./a.out
	rm ./a.out

llvm:
	clang $(CFF) -O0 -S -m64 -emit-llvm rscs/hello.c -o hello_64.ll
	clang $(CFF) -O0 -S -m32 -emit-llvm rscs/hello.c -o hello_32.ll

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

coverage: all
	cargo llvm-cov nextest --ignore-filename-regex '$(subst $(space),|,$(strip $(COV_SKIP)))'

clean:
	cargo clean
	rm -f $(LEX_RS) $(YACC_RS)

re: clean all

.PHONY: all clean re test ctest ttest c cc coverage $(FT_LEX) $(FT_YACC) $(NAME)
