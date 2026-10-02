/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_lstmap.c                                        :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:45:12 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:24 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

t_list *ft_lstmap(t_list *lst, void *(*f)(void *), void (*del)(void *)) {
	t_list *first;
	t_list *prev;
	t_list *curr;
	void *content;

	first = NULL;
	while (lst) {
		content = f(lst->content);
		if (!content && first)
			return (ft_lstclear(&first, del), NULL);
		curr = ft_lstnew(content);
		if (!curr) {
			del(content);
			if (first)
				return (ft_lstclear(&first, del), NULL);
		}
		if (!first)
			first = curr;
		else
			prev->next = curr;
		prev = curr;
		lst = lst->next;
	}
	return first;
}

#ifdef TEST
#include "test.h"

static void *test_dup(void *p)
{
	return (ft_strdup(p));
}

int	main(void)
{
	t_list *l;
	t_list *m;

	l = ft_lstnew("a");
	l->next = ft_lstnew("b");
	m = ft_lstmap(l, test_dup, free);
	CHECK(ft_lstsize(m) == 2 && m->content != l->content);
	STR(m->content, "a");
	STR(m->next->content, "b");
	ft_lstclear(&m, free);
	free(l->next);
	free(l);
	DONE();
}
#endif
