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
