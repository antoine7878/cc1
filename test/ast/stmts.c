int f(int x) {
    int i;
    if (x > 0)
        return 1;
    else
        return 2;
    if (x)
        if (x > 1)
            return 3;
        else
            return 4;
    while (x > 0)
        x--;
    do
        x++;
    while (x < 10);
    for (i = 0; i < 10; i++)
        x += i;
    for (;;)
        break;
    for (i = 0;;) {
        break;
    }
    switch (x) {
    case 1:
        break;
    case 2:
    case 3:
        return x;
    default:
        break;
    }
    goto end;
    ;
    {
    }
    return 0;
end:
    return x;
}
