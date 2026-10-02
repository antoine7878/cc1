#ifndef TEST_H
#define TEST_H

#include "libft.h"
#include <string.h>

int pipe(int fds[2]);
int read(int fd, void *buf, unsigned int n);
int close(int fd);

static int g_fail;

#define CHECK(e) ((e) ? (void)0 : fail(__LINE__, #e))
#define STR(a, b) CHECK(strcmp((a), (b)) == 0)
#define DONE() return (done(__FILE__))

static void fail(int line, char *expr)
{
	g_fail++;
	ft_putstr_fd("FAIL line ", 1);
	ft_putnbr_fd(line, 1);
	ft_putstr_fd(": ", 1);
	ft_putendl_fd(expr, 1);
}

static int done(char *file)
{
	ft_putstr_fd(file, 1);
	ft_putendl_fd(g_fail ? ": \033[31mKO\033[0m" : ": \033[32mOK\033[0m", 1);
	return (g_fail != 0);
}

static void read_back(int fds[2], char *buf, int size)
{
	int n;

	close(fds[1]);
	n = read(fds[0], buf, size - 1);
	close(fds[0]);
	buf[n < 0 ? 0 : n] = '\0';
}

#endif
