/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_memset.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:45:56 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:25 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void *ft_memset(void *s, int c, size_t n) {
	size_t i;

	i = 0;
	while (i < n) {
		((char *)s)[i] = (char)c;
		i++;
	}
	return s;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char a[8];

	CHECK(ft_memset(a, 'x', 5) == a);
	CHECK(memcmp(a, "xxxxx", 5) == 0);
	DONE();
}
#endif
