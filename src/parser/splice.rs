use std::io::{Read, Result};

pub struct SpliceReader<R: Read> {
    inner: R,
    buf: [u8; 4096],
    len: usize,
    pos: usize,
    held: Option<u8>,
    backslash: bool,
    deferred: usize,
    flushing: bool,
    eof: bool,
}

impl<R: Read> SpliceReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buf: [0; 4096],
            len: 0,
            pos: 0,
            held: None,
            backslash: false,
            deferred: 0,
            flushing: false,
            eof: false,
        }
    }

    fn next_byte(&mut self) -> Result<Option<u8>> {
        if self.pos == self.len {
            if self.eof {
                return Ok(None);
            }
            self.len = self.inner.read(&mut self.buf)?;
            self.pos = 0;
            if self.len == 0 {
                self.eof = true;
                return Ok(None);
            }
        }
        let b = self.buf[self.pos];
        self.pos += 1;
        Ok(Some(b))
    }
}

impl<R: Read> Read for SpliceReader<R> {
    fn read(&mut self, out: &mut [u8]) -> Result<usize> {
        let mut n = 0;
        while n < out.len() {
            if self.flushing {
                if self.deferred == 0 {
                    self.flushing = false;
                    continue;
                }
                self.deferred -= 1;
                out[n] = b'\n';
                n += 1;
                continue;
            }

            let b = match self.held.take() {
                Some(b) => b,
                None => match self.next_byte()? {
                    Some(b) => b,
                    None => {
                        if self.backslash {
                            self.backslash = false;
                            out[n] = b'\\';
                            n += 1;
                            continue;
                        }
                        if self.deferred > 0 {
                            self.flushing = true;
                            continue;
                        }
                        break;
                    }
                },
            };

            if self.backslash {
                self.backslash = false;
                if b == b'\n' {
                    self.deferred += 1;
                    continue;
                }
                out[n] = b'\\';
                n += 1;
                if n == out.len() {
                    self.held = Some(b);
                    break;
                }
            }

            if b == b'\\' {
                self.backslash = true;
                continue;
            }
            out[n] = b;
            n += 1;
            if b == b'\n' && self.deferred > 0 {
                self.flushing = true;
            }
        }
        Ok(n)
    }
}

#[cfg(test)]
mod test {
    use super::SpliceReader;
    use std::io::Read;

    fn splice(src: &str) -> String {
        let mut out = String::new();
        SpliceReader::new(src.as_bytes()).read_to_string(&mut out).unwrap();
        out
    }

    fn splice_chunked(src: &str, chunk: usize) -> String {
        let mut reader = SpliceReader::new(src.as_bytes());
        let mut out = Vec::new();
        let mut buf = vec![0u8; chunk];
        loop {
            match reader.read(&mut buf).unwrap() {
                0 => break,
                n => out.extend_from_slice(&buf[..n]),
            }
        }
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn no_splice_is_identity() {
        assert_eq!(splice("int x;\nint y;\n"), "int x;\nint y;\n");
    }

    #[test]
    fn splices_inside_string() {
        assert_eq!(splice("\"a\\\nb\";\n"), "\"ab\";\n\n");
    }

    #[test]
    fn splices_inside_identifier() {
        assert_eq!(splice("a\\\nb;\n"), "ab;\n\n");
    }

    #[test]
    fn splices_inside_keyword() {
        assert_eq!(splice("in\\\nt x;\n"), "int x;\n\n");
    }

    #[test]
    fn splices_inside_operator() {
        assert_eq!(splice("a =\\\n= b;\n"), "a == b;\n\n");
    }

    #[test]
    fn escaped_backslash_before_splice() {
        assert_eq!(splice("\"a\\\\\\\nb\";\n"), "\"a\\\\b\";\n\n");
    }

    #[test]
    fn lone_backslash_is_preserved() {
        assert_eq!(splice("\"a\\tb\";\n"), "\"a\\tb\";\n");
    }

    #[test]
    fn consecutive_splices_defer_every_newline() {
        assert_eq!(splice("a\\\n\\\nb;\n"), "ab;\n\n\n");
    }

    #[test]
    fn splice_at_eof_without_trailing_newline() {
        assert_eq!(splice("a\\\n"), "a\n");
    }

    #[test]
    fn trailing_backslash_at_eof_is_preserved() {
        assert_eq!(splice("a\\"), "a\\");
    }

    #[test]
    fn chunk_boundaries_do_not_change_output() {
        let src = "in\\\nt a;\n\"x\\\ny\";\na =\\\n= b;\n";
        let whole = splice(src);
        for chunk in 1..=8 {
            assert_eq!(splice_chunked(src, chunk), whole, "chunk size {chunk}");
        }
    }
}
