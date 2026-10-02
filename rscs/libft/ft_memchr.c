/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_memchr.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:45:28 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:24 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void *ft_memchr(void const *s, int c, size_t n) {
	size_t i;
	char *src;
	char ch;

	src = (char *)s;
	ch = c;
	i = 0;
	while (i < n && src[i] != ch)
		i++;
	if (i < n)
		return (char *)s + i;
	else
		return NULL;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char *s;

	s = "hello";
	CHECK(ft_memchr(s, 'l', 5) == s + 2);
	CHECK(ft_memchr(s, 'z', 5) == NULL);
	CHECK(ft_memchr(s, 'o', 4) == NULL);
	DONE();
}
#endif
