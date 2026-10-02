# **************************************************************************** #
#                                                                              #
#                                                         :::      ::::::::    #
#    Makefile                                           :+:      :+:    :+:    #
#                                                     +:+ +:+         +:+      #
#    By: ale-tell <ale-tell@student.42.fr>          +#+  +:+       +#+         #
#                                                 +#+#+#+#+#+   +#+            #
#    Created: 2024/10/16 20:57:01 by ale-tell          #+#    #+#              #
#    Updated: 2024/10/22 21:43:45 by ale-tell         ###   ########.fr        #
#                                                                              #
# **************************************************************************** #

NAME = libft.a
CC = ../../fcc
CDIR = .
HDIR = .
CFLAGS = -U__GNUC__ -U__clang__ -U__GNUC_MINOR__ -U__GNUC_PATCHLEVEL__ -U__clang_major__ \
		 -D__restrict= -I$(HDIR)

SRC =	$(CDIR)/ft_atoi.c \
		$(CDIR)/ft_bzero.c \
		$(CDIR)/ft_isalnum.c \
		$(CDIR)/ft_isalpha.c \
		$(CDIR)/ft_isascii.c \
		$(CDIR)/ft_isdigit.c \
		$(CDIR)/ft_isprint.c \
		$(CDIR)/ft_itoa.c \
		$(CDIR)/ft_memchr.c \
		$(CDIR)/ft_memcmp.c \
		$(CDIR)/ft_memcpy.c \
		$(CDIR)/ft_memmove.c \
		$(CDIR)/ft_memset.c \
		$(CDIR)/ft_putchar_fd.c \
		$(CDIR)/ft_putendl_fd.c \
		$(CDIR)/ft_putnbr_fd.c \
		$(CDIR)/ft_putstr_fd.c \
		$(CDIR)/ft_split.c \
		$(CDIR)/ft_strchr.c \
		$(CDIR)/ft_strdup.c \
		$(CDIR)/ft_striteri.c \
		$(CDIR)/ft_strjoin.c \
		$(CDIR)/ft_strlcat.c \
		$(CDIR)/ft_strlcpy.c \
		$(CDIR)/ft_strlen.c \
		$(CDIR)/ft_strmapi.c \
		$(CDIR)/ft_strncmp.c \
		$(CDIR)/ft_strnstr.c \
		$(CDIR)/ft_strrchr.c \
		$(CDIR)/ft_strtrim.c \
		$(CDIR)/ft_substr.c \
		$(CDIR)/ft_tolower.c \
		$(CDIR)/ft_calloc.c \
		$(CDIR)/ft_toupper.c

SRC_BONUS = $(CDIR)/ft_lstnew.c \
			$(CDIR)/ft_lstadd_front.c \
			$(CDIR)/ft_lstsize.c \
			$(CDIR)/ft_lstlast.c \
			$(CDIR)/ft_lstadd_back.c \
			$(CDIR)/ft_lstdelone.c \
			$(CDIR)/ft_lstclear.c \
			$(CDIR)/ft_lstiter.c \
			$(CDIR)/ft_lstmap.c

ODIR = .
OBJ = $(patsubst $(CDIR)/%.c, $(ODIR)/%.o, $(SRC))
OBJ_BONUS = $(patsubst $(CDIR)/%.c, $(ODIR)/%.o, $(SRC_BONUS))

TDIR = tests
TEST = $(patsubst $(CDIR)/%.c, $(TDIR)/%, $(SRC) $(SRC_BONUS))
RUN = qemu-i386 -L /usr/i686-linux-gnu

all: $(NAME)

$(NAME): $(OBJ)
	ar rcs $@ $^

bonus: $(OBJ_BONUS) $(OBJ)
	ar rcs $(NAME) $^

$(ODIR)/%.o: $(CDIR)/%.c | $(ODIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(ODIR):
	mkdir -p $@

$(TDIR)/%: $(CDIR)/%.c test.h bonus | $(TDIR)
	$(CC) $(CFLAGS) -DTEST $< $(NAME) -o $@

$(TDIR):
	mkdir -p $@

test: $(TEST)
	@fail=0; for t in $(TEST); do $(RUN) ./$$t || fail=1; done; exit $$fail

clean:
	rm -f $(OBJ) $(OBJ_BONUS)
	rm -rf $(TDIR)

fclean: clean
	rm -f $(NAME)

re: fclean all

.PHONY: clean fclean all re test
