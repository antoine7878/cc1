/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_strdup.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:46:59 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:25 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

char *ft_strdup(char const *s) {
	char *ret;
	size_t i;

	i = 0;
	ret = (char *)ft_calloc(sizeof(char), ft_strlen(s) + 1);
	if (!ret)
		return 0;
	while (s[i]) {
		ret[i] = s[i];
		i++;
	}
	ret[i] = '\0';
	return ret;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char *s;

	s = ft_strdup("abc");
	STR(s, "abc");
	free(s);
	DONE();
}
#endif
