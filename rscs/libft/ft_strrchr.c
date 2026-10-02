/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_strrchr.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:47:52 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:26 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

static void *ft_memrchr(void const *s, int c, size_t n) {
	size_t i;
	char *src;
	char ch;

	src = (char *)s;
	ch = c;
	i = 0;
	if (n == 0)
		return (void *)s;
	while (i < n && src[n - i - 1] != ch)
		i++;
	if (i < n)
		return (char *)s + (n - i - 1);
	else
		return NULL;
}

char *ft_strrchr(char const *s, int c) {
	return ft_memrchr(s, c, ft_strlen(s) + 1);
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char *s;

	s = "hello world";
	CHECK(ft_strrchr(s, 'o') == s + 7);
	CHECK(ft_strrchr(s, '\0') == s + 11);
	CHECK(ft_strrchr(s, 'z') == NULL);
	DONE();
}
#endif
