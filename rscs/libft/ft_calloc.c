/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_calloc.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:43:10 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:22 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void *ft_calloc(size_t nmemb, size_t size) {
	char *ret;
	size_t len;

	len = nmemb * size;
	if (nmemb != 0 && len / nmemb != size)
		return NULL;
	ret = (char *)(malloc(len));
	if (!ret)
		return NULL;
	ft_bzero(ret, len);
	return ret;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	int *p;

	p = ft_calloc(4, sizeof(int));
	CHECK(p && p[0] == 0 && p[3] == 0);
	free(p);
	DONE();
}
#endif
