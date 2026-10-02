# cc1

A C90 compiler front end, written in Rust, that emits LLVM IR.
Parsing is done with custom Rust versions of Lex and Yacc: `ft_lex` and `ft_yacc`

- ISO/IEC 9899:1990 compliant
- Targets i386
- `fcc.py` compiler driver
- `cc1` the compiler to LLVM IR
- Lowering with `llc` and assembly with `ar`, linking with `clang`

## Usage

`./fcc.py -h`

`fcc.py` is the cc-style front. `-E`, `-S`, `-c` stop where you'd expect;
`-e` stops after `cc1` and leaves a `.ll`. Input files are routed by
extension (`.c`, `.i`, `.ll`, `.s`, `.o`).

## Test in Lima

```
alias dm="make -f lima.mk"
```

The test suite runs on macOS host inside Lima.
`lima.mk` runs make rule in a native aarch64 Lima VM;
i386 binaries are built with `i686-linux-gnu-gcc` and run with `qemu-i386`.

```
dm up
dm start / lm stop
dm ctest
dm run
dm down
```

## ft_lex and ft_yacc

`ft_lex` builds is table-driven DFA Rust lexer generator.
`ft_yacc` is a LALR(1) Rust parser generator with.
