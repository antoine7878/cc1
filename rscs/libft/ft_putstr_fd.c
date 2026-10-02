/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_putstr_fd.c                                     :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:46:27 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:25 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void ft_putstr_fd(char *s, int fd) {
	while (*s) {
		ft_putchar_fd(*s, fd);
		s++;
	}
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	int fds[2];
	char buf[32];

	pipe(fds);
	ft_putstr_fd("abc", fds[1]);
	read_back(fds, buf, sizeof(buf));
	STR(buf, "abc");
	DONE();
}
#endif
