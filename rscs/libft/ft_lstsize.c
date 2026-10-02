/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_lstsize.c                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:45:23 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:24 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

int ft_lstsize(t_list *lst) {
	int size;

	if (!lst)
		return 0;
	size = 1;
	while (lst->next) {
		lst = lst->next;
		size++;
	}
	return size;
}

#ifdef TEST
#include "test.h"

int	main(void)
{
	t_list *l;

	CHECK(ft_lstsize(NULL) == 0);
	l = ft_lstnew("a");
	l->next = ft_lstnew("b");
	CHECK(ft_lstsize(l) == 2);
	free(l->next);
	free(l);
	DONE();
}
#endif
