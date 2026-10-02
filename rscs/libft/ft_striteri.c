/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_striteri.c                                      :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:47:04 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:26 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void ft_striteri(char *s, void (*f)(unsigned int, char *)) {
	size_t i;

	i = 0;
	while (s[i]) {
		f(i, s + i);
		i++;
	}
}

#ifdef TEST
#include "test.h"

static void test_inc(unsigned int i, char *c)
{
	*c += (char)i;
}

int	main(void)
{
	char s[4];

	strcpy(s, "aaa");
	ft_striteri(s, test_inc);
	STR(s, "abc");
	DONE();
}
#endif
