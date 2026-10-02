/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_memcpy.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:45:42 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:24 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void *ft_memcpy(void *dest, void const *src, size_t n) {
	size_t i;
	char *d;
	char *s;

	d = (char *)dest;
	s = (char *)src;
	i = 0;
	while (i < n) {
		d[i] = s[i];
		i++;
	}
	return dest;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char b[8];

	CHECK(ft_memcpy(b, "hello", 6) == b);
	STR(b, "hello");
	DONE();
}
#endif
