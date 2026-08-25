NAME = "./ft_yacc/target/release/ft_yacc"

all: $(NAME)

$(NAME):
	cargo build --release

test: $(NAME)
	make -C ../ft_lex/
	cargo test

clean:
	cargo clean

re: clean all

.PHONY: all clean re test
