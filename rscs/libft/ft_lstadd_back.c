/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_lstadd_back.c                                   :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:44:30 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:23 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void ft_lstadd_back(t_list **lst, t_list *new) {
	if (!*lst)
		*lst = new;
	else
		ft_lstlast(*lst)->next = new;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	t_list *l;

	l = NULL;
	ft_lstadd_back(&l, ft_lstnew("a"));
	ft_lstadd_back(&l, ft_lstnew("b"));
	STR(l->content, "a");
	STR(l->next->content, "b");
	CHECK(l->next->next == NULL);
	free(l->next);
	free(l);
	DONE();
}
#endif
