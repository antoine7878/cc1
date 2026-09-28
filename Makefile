CC1 = target/debug/cc1

FT_LEX  = target/release/ft_lex
FT_YACC = target/release/ft_yacc

C_L = crates/cc1/src/parser/c.l
C_Y = crates/cc1/src/parser/c.y

LEX_RS =  crates/cc1/src/parser/lex.rs
YACC_RS = crates/cc1/src/parser/yacc.rs

all: $(LEX_RS) $(YACC_RS)
	cargo build -p cc1

generators:
	cargo build --release -p ft_lex -p ft_yacc

$(LEX_RS): $(C_L) | generators
	$(FT_LEX) -c $< -o $@

$(YACC_RS): $(C_Y) | generators
	$(FT_YACC) $< -o $@

# ----- test --------------------

test: all
	rm -f ./hello.ll ./hello.s ./hello.o ./a.out
	./fcc -e ./rscs/hello.c -o-

ctest: all
	cargo nextest run -p cc1

ttest: all
	cargo nextest run

ftest: all cc
	./fcc ./rscs/hello.c -o ./rscs/a.out
	$(I386_RUN) ./rscs/a.out || echo $$?

# ----- reference --------------------

CFF = -std=iso9899:1990 -pedantic-errors

I386_CC  = i686-linux-gnu-gcc
I386_RUN = qemu-i386 -L /usr/i686-linux-gnu

c:
	$(I386_CC) -c $(CFF) rscs/hello.c -o /dev/null

cc:
	$(I386_CC) $(CFF) rscs/hello.c
	$(I386_RUN) ./a.out || echo $$?
	rm ./a.out

llvm:
	clang --target=i686-linux-gnu $(CFF) -O0 -S -emit-llvm rscs/hello.c -o ./rscs/hello.ll

empty :=
space := $(empty) $(empty)

COV_SKIP = \
	crates/cc1/src/main.rs \
	crates/cc1/src/report.rs \
	crates/cc1/src/ast/mod.rs \
	crates/cc1/src/ast/name.rs \
	crates/cc1/src/ast/print.rs \
	crates/cc1/src/parser/lex.rs \
	crates/cc1/src/parser/yacc.rs \
	crates/cc1/src/ast/display.rs \
	crates/cc1/src/parser/driver.rs \
	crates/cc1/src/ast/type_specifier.rs

coverage: all
	cargo llvm-cov nextest --ignore-filename-regex '$(subst $(space),|,$(strip $(COV_SKIP)))'

clean:
	cargo clean
	rm -f $(LEX_RS) $(YACC_RS)

re: clean all

.PHONY: all clean re test ctest ttest ftest c cc coverage parser-tools llvm
