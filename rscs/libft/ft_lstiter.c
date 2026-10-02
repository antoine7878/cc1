/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_lstiter.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:44:57 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:24 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void ft_lstiter(t_list *lst, void (*f)(void *)) {
	while (lst) {
		f(lst->content);
		lst = lst->next;
	}
}

#ifdef TEST
#include "test.h"

static void test_upper(void *p)
{
	*(char *)p = (char)ft_toupper(*(char *)p);
}

int	main(void)
{
	t_list *l;

	l = ft_lstnew(ft_strdup("ab"));
	l->next = ft_lstnew(ft_strdup("cd"));
	ft_lstiter(l, test_upper);
	STR(l->content, "Ab");
	STR(l->next->content, "Cd");
	ft_lstclear(&l, free);
	DONE();
}
#endif
