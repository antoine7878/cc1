/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_strlcat.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:47:16 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:47:19 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

size_t ft_strlcat(char *dest, char const *src, size_t size) {
	size_t src_len;
	size_t dest_len;

	src_len = ft_strlen(src);
	dest_len = ft_strlen(dest);
	if (dest_len >= size)
		return src_len + size;
	if (size > dest_len + src_len)
		ft_memcpy(dest + dest_len, src, src_len + 1);
	else {
		ft_memcpy(dest + dest_len, src, size - dest_len - 1);
		dest[size - 1] = '\0';
	}
	return src_len + dest_len;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char buf[8];

	strcpy(buf, "he");
	CHECK(ft_strlcat(buf, "llo", sizeof(buf)) == 5);
	STR(buf, "hello");
	CHECK(ft_strlcat(buf, "abc", 3) == 6);
	STR(buf, "hello");
	DONE();
}
#endif
