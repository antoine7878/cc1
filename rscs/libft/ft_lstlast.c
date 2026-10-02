/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_lstlast.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:45:00 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:24 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

t_list *ft_lstlast(t_list *lst) {
	if (!lst)
		return NULL;
	while (lst->next) {
		lst = lst->next;
	}
	return lst;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	t_list *l;

	CHECK(ft_lstlast(NULL) == NULL);
	l = ft_lstnew("a");
	l->next = ft_lstnew("b");
	CHECK(ft_lstlast(l) == l->next);
	free(l->next);
	free(l);
	DONE();
}
#endif
