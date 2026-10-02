/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_atoi.c                                          :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:42:38 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:22 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

static int ft_isspace(int c) {
	if ((9 <= c && c <= 13) || c == ' ')
		return 1;
	return 0;
}

int ft_atoi(char const *nptr) {
	int sign;
	int ret;
	size_t i;

	sign = 1;
	i = 0;
	ret = 0;
	while (ft_isspace(nptr[i]))
		i++;
	if (nptr[i] == '-') {
		sign = -1;
		i++;
	} else if (nptr[i] == '+') {
		i++;
	}
	while (ft_isdigit(nptr[i])) {
		ret = ret * 10 + (nptr[i] - '0');
		i++;
	}
	return sign * ret;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	CHECK(ft_atoi("  -42abc") == -42 && ft_atoi("+7") == 7 && ft_atoi("x1") == 0);
	CHECK(ft_atoi("2147483647") == 2147483647);
	CHECK(ft_atoi("-2147483648") == -2147483647 - 1);
	DONE();
}
#endif
