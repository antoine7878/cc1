/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_memcmp.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:45:35 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:24 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int ft_memcmp(void const *s1, void const *s2, size_t n) {
	size_t i;
	unsigned char *str1;
	unsigned char *str2;

	str1 = (unsigned char *)s1;
	str2 = (unsigned char *)s2;
	i = 0;
	if (n == 0)
		return 0;
	while (i < n - 1 && str1[i] == str2[i])
		i++;
	return str1[i] - str2[i];
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	CHECK(ft_memcmp("abc", "abd", 3) < 0 && ft_memcmp("abd", "abc", 3) > 0);
	CHECK(ft_memcmp("abc", "abd", 2) == 0);
	CHECK(ft_memcmp("\200", "\0", 1) > 0);
	DONE();
}
#endif
