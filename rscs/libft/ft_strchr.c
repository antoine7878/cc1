/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_strchr.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:46:44 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:25 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

char *ft_strchr(char const *s, int c) {
	return (char *)ft_memchr(s, c, ft_strlen(s) + 1);
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char *s;

	s = "hello world";
	CHECK(ft_strchr(s, 'o') == s + 4);
	CHECK(ft_strchr(s, '\0') == s + 11);
	CHECK(ft_strchr(s, 'z') == NULL);
	DONE();
}
#endif
