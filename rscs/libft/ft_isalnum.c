/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_isalnum.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:43:17 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:23 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int ft_isalnum(int c) {
	if (ft_isalpha(c) || ft_isdigit(c))
		return 1;
	return 0;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	CHECK(ft_isalnum('a') && ft_isalnum('9') && !ft_isalnum(' '));
	DONE();
}
#endif
