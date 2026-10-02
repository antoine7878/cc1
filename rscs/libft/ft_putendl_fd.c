/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_putendl_fd.c                                    :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:46:16 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:25 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void ft_putendl_fd(char *s, int fd) {
	ft_putstr_fd(s, fd);
	ft_putchar_fd('\n', fd);
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	int fds[2];
	char buf[32];

	pipe(fds);
	ft_putendl_fd("abc", fds[1]);
	read_back(fds, buf, sizeof(buf));
	STR(buf, "abc\n");
	DONE();
}
#endif
