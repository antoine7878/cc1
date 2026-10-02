/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_strncmp.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:47:42 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:26 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int ft_strncmp(char const *s1, char const *s2, size_t n) {
	size_t i;

	i = 0;
	if (n == 0)
		return 0;
	while (s1[i] && s2[i] && s1[i] == s2[i] && i < n - 1)
		i++;
	return (unsigned char)s1[i] - (unsigned char)s2[i];
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	CHECK(ft_strncmp("abc", "abd", 2) == 0);
	CHECK(ft_strncmp("abc", "abd", 3) < 0 && ft_strncmp("abd", "abc", 3) > 0);
	CHECK(ft_strncmp("ab", "abc", 5) < 0);
	DONE();
}
#endif
