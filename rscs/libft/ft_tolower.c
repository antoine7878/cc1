/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_tolower.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:48:12 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:27 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int ft_tolower(int c) {
	if ('A' <= c && c <= 'Z')
		return c + 32;
	return c;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	CHECK(ft_tolower('A') == 'a' && ft_tolower('a') == 'a' && ft_tolower('1') == '1');
	DONE();
}
#endif
