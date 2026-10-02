/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   ft_lstdelone.c                                     :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: ale-tell <ale-tell@42student.fr>           +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/11/13 15:44:52 by ale-tell          #+#    #+#             */
/*   Updated: 2024/11/13 15:48:24 by ale-tell         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "libft.h"

void ft_lstdelone(t_list *lst, void (*del)(void *)) {
	del(lst->content);
	free(lst);
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
	ft_lstdelone(ft_lstnew("a"), test_del);
	CHECK(g_del == 1);
	DONE();
}
#endif
