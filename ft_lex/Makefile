NAME = "./ft_lex/target/release/ft_lex"

all: $(NAME)

$(NAME):
	cargo build

clean:
	cargo clean

re: clean all

test: $(NAME)
	cargo test

.PHONY: all clean re
