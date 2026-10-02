/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_lstadd_front.c                                  :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:44:39 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:23 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void ft_lstadd_front(t_list **lst, t_list *new) {
	new->next = *lst;
	*lst = new;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	t_list *l;

	l = ft_lstnew("b");
	ft_lstadd_front(&l, ft_lstnew("a"));
	STR(l->content, "a");
	STR(l->next->content, "b");
	CHECK(l->next->next == NULL);
	free(l->next);
	free(l);
	DONE();
}
#endif
