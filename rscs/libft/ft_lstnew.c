/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_lstnew.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:45:17 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:24 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

t_list *ft_lstnew(void *content) {
	t_list *link;

	link = malloc(sizeof(t_list));
	if (!link)
		return NULL;
	link->content = content;
	link->next = NULL;
	return link;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	t_list *l;

	l = ft_lstnew("a");
	CHECK(l && l->next == NULL);
	STR(l->content, "a");
	free(l);
	DONE();
}
#endif
