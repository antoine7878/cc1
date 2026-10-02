/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_strtrim.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:47:57 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:26 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

char *ft_strtrim(char const *s1, char const *set) {
	size_t start;
	size_t end;

	start = 0;
	end = ft_strlen(s1) - 1;
	if (ft_strlen(set) == 0 || ft_strlen(s1) == 0)
		return ft_strdup(s1);
	while (start <= end && ft_strchr(set, s1[start]))
		start++;
	if (start > end)
		return ft_strdup("");
	while (end >= start && ft_strchr(set, s1[end]))
		end--;
	return ft_substr(s1, start, end - start + 1);
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char *s;

	s = ft_strtrim("xyhixy", "xy");
	STR(s, "hi");
	free(s);
	s = ft_strtrim("xxx", "x");
	STR(s, "");
	free(s);
	DONE();
}
#endif
