/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_strlen.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:47:30 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:26 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

size_t ft_strlen(char const *str) {
	size_t ret;

	ret = 0;
	while (str[ret])
		ret++;
	return ret;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	CHECK(ft_strlen("") == 0 && ft_strlen("hello") == 5);
	DONE();
}
#endif
