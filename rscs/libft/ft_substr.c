/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_substr.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:48:08 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:26 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

char *ft_substr(char const *s, unsigned int start, size_t len) {
	char *ret;
	size_t slen;

	slen = ft_strlen(s);
	if (start > slen)
		return ft_strdup("");
	if (start + len > slen)
		len = slen - start;
	ret = (char *)ft_calloc(sizeof(char), len + 1);
	if (!ret)
		return NULL;
	ft_memcpy(ret, s + start, len);
	ret[len] = '\0';
	return ret;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char *s;

	s = ft_substr("hello", 1, 3);
	STR(s, "ell");
	free(s);
	s = ft_substr("hello", 3, 10);
	STR(s, "lo");
	free(s);
	s = ft_substr("hello", 10, 3);
	STR(s, "");
	free(s);
	DONE();
}
#endif
