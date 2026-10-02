/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_toupper.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:49:03 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:49:05 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int ft_toupper(int c) {
	if ('a' <= c && c <= 'z')
		return c - 32;
	return c;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	CHECK(ft_toupper('a') == 'A' && ft_toupper('A') == 'A' && ft_toupper('1') == '1');
	DONE();
}
#endif
