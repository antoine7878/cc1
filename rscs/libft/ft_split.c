/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_split.c                                         :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:46:35 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:25 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

static int ft_countwords(char const *s, char c) {
	size_t i;
	size_t j;

	i = 0;
	j = 0;
	while (s[i]) {
		while (s[i] && s[i] == c)
			i++;
		if (!s[i])
			return j;
		while (s[i] && s[i] != c)
			i++;
		j++;
	}
	return j;
}

static int ft_wordlen(char const *s, char c) {
	size_t i;

	i = 0;
	while (s[i] && s[i] != c)
		i++;
	return i;
}

static char **free_split(char **words) {
	size_t i;

	i = 0;
	while (words[i])
		free(words[i++]);
	if (words)
		free(words);
	return NULL;
}

char **ft_split(char const *s, char c) {
	size_t i;
	size_t j;
	char **words;
	size_t wordlen;

	i = 0;
	j = 0;
	words = (char **)ft_calloc(sizeof(char *), ft_countwords(s, c) + 1);
	if (!words)
		return NULL;
	while (s[i]) {
		while (s[i] && s[i] == c)
			i++;
		if (s[i]) {
			wordlen = ft_wordlen(s + i, c);
			words[j] = ft_substr(s, i, wordlen);
			if (!words[j++])
				return free_split(words);
			i += wordlen;
		}
	}
	words[j] = 0;
	return words;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	char **v;

	v = ft_split("  a bb  c ", ' ');
	STR(v[0], "a");
	STR(v[1], "bb");
	STR(v[2], "c");
	CHECK(v[3] == NULL);
	free(v[0]);
	free(v[1]);
	free(v[2]);
	free(v);
	v = ft_split("", ' ');
	CHECK(v[0] == NULL);
	free(v);
	DONE();
}
#endif
