/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_itoa.c                                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:44:20 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:23 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

static size_t ft_intlen(int nb) {
	size_t ret;

	ret = 1;
	while (nb != 0) {
		nb /= 10;
		ret++;
	}
	return ret - 1;
}

static int ft_abs(int nb) {
	if (nb < 0)
		return -nb;
	return nb;
}

char *ft_itoa(int n) {
	char *ret;
	char *retn;
	size_t len;
	int nb;

	if (n == 0)
		return ft_strdup("0");
	nb = n;
	len = ft_intlen(n) - 1;
	ret = (char *)ft_calloc(sizeof(char), len + 2);
	if (!ret)
		return ret;
	while (nb != 0) {
		ret[len--] = ft_abs(nb % 10) + '0';
		nb /= 10;
	}
	if (n < 0) {
		retn = ft_strjoin("-", ret);
		free(ret);
		return retn;
	}
	return ret;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char *s;

	s = ft_itoa(-2147483647 - 1);
	STR(s, "-2147483648");
	free(s);
	s = ft_itoa(0);
	STR(s, "0");
	free(s);
	s = ft_itoa(42);
	STR(s, "42");
	free(s);
	DONE();
}
#endif
