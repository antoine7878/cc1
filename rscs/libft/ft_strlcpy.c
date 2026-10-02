/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_strlcpy.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:47:25 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:26 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

static size_t ft_min(size_t n1, size_t n2) {
	if (n1 < n2)
		return n1;
	return n2;
}

size_t ft_strlcpy(char *dest, char const *src, size_t size) {
	size_t src_len;
	size_t len;

	src_len = ft_strlen(src);
	if (size == 0)
		return src_len;
	len = ft_min(src_len, size - 1);
	ft_memcpy(dest, src, len);
	dest[len] = '\0';
	return src_len;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char buf[8];

	CHECK(ft_strlcpy(buf, "hello", 3) == 5);
	STR(buf, "he");
	CHECK(ft_strlcpy(buf, "hi", sizeof(buf)) == 2);
	STR(buf, "hi");
	DONE();
}
#endif
