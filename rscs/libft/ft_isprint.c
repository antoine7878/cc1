/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_isprint.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:44:13 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:23 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int ft_isprint(int c) {
	if (32 <= c && c <= 126)
		return 1;
	return 0;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	CHECK(ft_isprint(' ') && ft_isprint('~') && !ft_isprint(127));
	DONE();
}
#endif
