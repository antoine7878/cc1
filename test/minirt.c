#include "minirt.h"
#include "libft.h"
#include "mlx.h"
#include <stddef.h>

typedef struct {
} t_mr;

typedef int t_img;
typedef int t_shape;

int main(int argc, char **argv) {
	t_mr mr;
	t_img img;

	ft_memset(&mr, 0, sizeof(t_mr));
	if (argc > 2)
		return (ft_dprintf(2, "Error\nToo many arguments\n"), 1);
	if (argc < 2)
		return (ft_dprintf(2, "Error\nNot enough arguments\n"), 1);
	if (ft_strrncmp(argv[1], ".rt", 3))
		return (ft_dprintf(2, "Error\nFile extension should be '.rt'\n"), 1);
	if (start_mlx(&mr, &img))
		return (close_all(&mr), 1);
	if (parse_args(argv[1], &mr))
		return (close_all(&mr), 1);
	print_shapes((t_shape **)mr.shapes);
	if (start_win(&mr))
		return (ft_dprintf(2, "Error\nCould not create mlx window\n"), -1);
	init_cam(&mr);
	render_frame(&mr);
	mlx_loop(mr.mlx);
	close_all(&mr);
	return 0;
}

// get_time();
