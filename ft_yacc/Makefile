NAME = "./ft_yacc/target/release/ft_yacc"

all: $(NAME)

$(NAME):
	cargo build --release

test: $(NAME)
	make -C ../ft_lex/
	./../ft_lex/target/release/ft_lex -x rust test/calc_r.l
	./target/release/ft_yacc -x rust test/calc_r.y
	rustc --edition=2024 calc_r_tab.rs
	./calc_r_tab "2+3*4"

clean:
	cargo clean

re: clean all

.PHONY: all clean re test
