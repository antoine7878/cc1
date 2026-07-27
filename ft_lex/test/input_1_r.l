%%

"/*"  {
    let mut prev: u8 = 0;
    loop {
        match (self.input(), prev) {
            (0, _) => break,
            (b'/', b'*') => break,
            (c, _) => prev = c,
        }
    }
}

%%
