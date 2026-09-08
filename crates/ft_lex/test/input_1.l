%%

"/*"  {
            int c, prev = 0;
            while ((c = input()) != 0) {
                if (c == '/' && prev == '*')
                    break;
                prev = c;
            }
        }

%%

int main(int argc, char **argv) {
    yyin = fmemopen(argv[1], strlen(argv[1]), "r");
    while (yylex()) ;
}
