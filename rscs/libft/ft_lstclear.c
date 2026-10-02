/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_lstclear.c                                      :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:44:45 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:23 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void ft_lstclear(t_list **lst, void (*del)(void *)) {
	t_list *l;
	t_list *m;

	if (!*lst)
		return;
	l = *lst;
	while (l) {
		m = l->next;
		ft_lstdelone(l, del);
		l = m;
	}
	*lst = NULL;
}

#ifdef TEST
#include "test.h"

static int g_del;

static void test_del(void *p)
{
	(void)p;
	g_del++;
}

int	main(void)
{
	t_list *l;

	l = ft_lstnew("a");
	l->next = ft_lstnew("b");
	ft_lstclear(&l, test_del);
	CHECK(l == NULL && g_del == 2);
	DONE();
}
#endif
