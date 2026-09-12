# cc1 WIP

A C90 compiler front end, written in Rust, that emits LLVM IR.
Parsing is done with custom Rust versions of Lex and Yacc: `ft_lex` and `ft_yacc`

- Soon™ ISO/IEC 9899:1990 compliant
- Support targets i386 and x86_64
- `fcc` compiler driver
- `cpp` C PreProcessor (just `clang -E` for now)
- `cc1` the compiler to LLVM IR
- Lowering with `llc` and assembly with `ar`, linking with `clang`

## Usage

`cargo run --bin fcc -- -h`

`fcc` is the cc-style front. `-E`, `-S`, `-c` stop where you'd expect;
`-e` stops after `cc1` and leaves a `.ll`. Input files are routed by
extension (`.c`, `.i`, `.ll`, `.s`, `.o`).

## Test in Docker

```
alias dm="make -f docker.mk"
```

`docker.mk` allow to run any make rule in a docker with i386 support

```
dm up               # build image, start container with the repo mounted
dm ctest            # any make target runs inside the container
dm run              # shell in it
```

## ft_lex and ft_yacc

Both are usable on their own, they are not tied to `cc1`.
`ft_lex` builds is table-driven DFA Rust lexer generator.
`ft_yacc` is a LALR(1) Rust parser generator with.
