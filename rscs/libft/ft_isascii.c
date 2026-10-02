/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_isascii.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:44:04 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:23 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int ft_isascii(int c) {
	if (0 <= c && c <= 127)
		return 1;
	return 0;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	CHECK(ft_isascii(0) && ft_isascii(127) && !ft_isascii(128));
	DONE();
}
#endif
