/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_strjoin.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:47:11 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:26 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

char *ft_strjoin(char const *s1, char const *s2) {
	char *ret;
	size_t len;

	len = ft_strlen(s1) + ft_strlen(s2) + 1;
	ret = (char *)ft_calloc(sizeof(char), len);
	if (!ret)
		return NULL;
	ret[0] = '\0';
	ft_strlcpy(ret, s1, len);
	ft_strlcat(ret, s2, len);
	return ret;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char *s;

	s = ft_strjoin("ab", "cd");
	STR(s, "abcd");
	free(s);
	DONE();
}
#endif
