/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_isdigit.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:44:09 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:23 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int ft_isdigit(int c) {
	if ('0' <= c && c <= '9')
		return 1;
	return 0;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	CHECK(ft_isdigit('0') && ft_isdigit('9') && !ft_isdigit('a'));
	DONE();
}
#endif
