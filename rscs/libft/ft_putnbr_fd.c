/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_putnbr_fd.c                                     :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:46:22 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:25 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void ft_putnbr_fd(int n, int fd) {
	if (n < 0) {
		ft_putchar_fd('-', fd);
		if (n < -9)
			ft_putnbr_fd(-(n / 10), fd);
		ft_putchar_fd(-(n % 10) + '0', fd);
		return;
	}
	if (n > 9)
		ft_putnbr_fd(n / 10, fd);
	ft_putchar_fd((n % 10) + '0', fd);
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	int fds[2];
	char buf[32];

	pipe(fds);
	ft_putnbr_fd(-2147483647 - 1, fds[1]);
	ft_putnbr_fd(0, fds[1]);
	ft_putnbr_fd(42, fds[1]);
	read_back(fds, buf, sizeof(buf));
	STR(buf, "-2147483648042");
	DONE();
}
#endif
