/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_memmove.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:45:52 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:25 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void *ft_memmove(void *dest, void const *src, size_t n) {
	char *d;
	char *s;

	d = (char *)dest;
	s = (char *)src;
	if (src > dest)
		return ft_memcpy(dest, src, n);
	else if (src < dest) {
		while (n-- != 0)
			d[n] = s[n];
	}
	return dest;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char a[8];

	strcpy(a, "abcdef");
	CHECK(ft_memmove(a + 2, a, 4) == a + 2);
	STR(a, "ababcd");
	strcpy(a, "abcdef");
	ft_memmove(a, a + 2, 5);
	STR(a, "cdef");
	DONE();
}
#endif
