/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_isalpha.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:43:59 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:23 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int ft_isalpha(int c) {
	if (('A' <= c && c <= 'Z') || ('a' <= c && c <= 'z'))
		return 1;
	return 0;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	CHECK(ft_isalpha('a') && ft_isalpha('Z') && !ft_isalpha('1'));
	DONE();
}
#endif
