/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_bzero.c                                         :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:42:46 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:42:57 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void ft_bzero(void *s, size_t n) {
	ft_memset(s, 0, n);
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char a[4];

	strcpy(a, "xxx");
	ft_bzero(a, 2);
	CHECK(a[0] == 0 && a[1] == 0 && a[2] == 'x');
	DONE();
}
#endif
