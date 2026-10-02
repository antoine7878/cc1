/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_strmapi.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 17:22:20 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 17:22:21 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

char *ft_strmapi(char const *s, char (*f)(unsigned int, char)) {
	char *ret;
	size_t i;

	ret = (char *)ft_calloc(sizeof(char), ft_strlen(s) + 1);
	if (!ret)
		return NULL;
	i = 0;
	while (s[i]) {
		ret[i] = f(i, s[i]);
		i++;
	}
	ret[i] = '\0';
	return ret;
}

#ifdef TEST
#include "test.h"

static char test_up(unsigned int i, char c)
{
	return (i % 2 ? c : (char)ft_toupper(c));
}

int	main(void)
{
	char *s;

	s = ft_strmapi("abcd", test_up);
	STR(s, "AbCd");
	free(s);
	DONE();
}
#endif
