/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_strnstr.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:47:48 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:26 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"
#include <string.h>

char *ft_strnstr(char const *str, char const *to_find, size_t len) {
	size_t i;
	size_t j;

	i = 0;
	if (!to_find[i])
		return (char *)str;
	while (str[i] && i < len) {
		j = 0;
		while (i + j < len && to_find[j] && to_find[j] == str[j + i])
			j++;
		if (to_find[j] == '\0')
			return (char *)str + i;
		i++;
	}
	return NULL;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char *s;

	s = "hello world";
	CHECK(ft_strnstr(s, "wor", 11) == s + 6);
	CHECK(ft_strnstr(s, "wor", 7) == NULL);
	CHECK(ft_strnstr(s, "", 3) == s);
	DONE();
}
#endif
