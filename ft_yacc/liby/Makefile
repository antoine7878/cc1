NAME = ./liby.a
SRC = ./yyerror.c ./main.c
OBJ = ${SRC:.c=.o}
CC = clang
CFLAGS = -Wall -Wextra -Werror

all : $(NAME)

$(NAME): $(OBJ)
	ar rcs $@ $^

clean:
	rm -f $(OBJ)

fclean: clean
	rm -f $(NAME)

re: fclean all

.PHONY: all clean fclean re
