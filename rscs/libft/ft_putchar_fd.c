/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_putchar_fd.c                                    :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:46:08 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:25 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int	write(int fd, const void *buf, unsigned int n);

void	ft_putchar_fd(char c, int fd)
{
	if (fd < 0)
		return ;
	write(fd, &c, 1);
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	int fds[2];
	char buf[32];

	pipe(fds);
	ft_putchar_fd('x', fds[1]);
	read_back(fds, buf, sizeof(buf));
	STR(buf, "x");
	DONE();
}
#endif
