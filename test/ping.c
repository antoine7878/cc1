#include "ping.h"
#include "arg_parser.h"
#include "constants.h"

#include <limits.h>
#include <stdint.h>
#include <string.h>

static const int options[];

int main(int argc, char *argv[]) {
	int ping;
	memset(&ping, 0, sizeof(int));

	parse_args(argc, argv, options, errorf);
	check_args(&ping);
	init_ping(&ping, argv[1]);
	loop_ping(&ping);
	ping_finish(&ping);
	free_ping(&ping);
	return ping.stats.err_tot > 0 || ping.stats.send_tot != ping.stats.recv_tot;
}
