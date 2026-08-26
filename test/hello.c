union U {
	char a;
	int : 0;
};

/*
...|...|...|...|...|
ss..bbb.aaaa
*/

// on veut 1
enum s { OUI = sizeof(union U) };

// #include <stdalign.h>
// #include <stdio.h>
//
// int main(void) {
// 	printf("%i\n", OUI);
// 	return 0;
// }
