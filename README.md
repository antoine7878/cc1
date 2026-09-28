# cc1 WIP

A C90 compiler front end, written in Rust, that emits LLVM IR.
Parsing is done with custom Rust versions of Lex and Yacc: `ft_lex` and `ft_yacc`

- Soon™ ISO/IEC 9899:1990 compliant
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
alias lm="make -f lima.mk"
```

The test suite only runs inside Lima. `lima.mk` runs any make rule in a native
aarch64 Lima VM; i386 binaries are built with `i686-linux-gnu-gcc` and run with
`qemu-i386`. The repo is mounted at `/work`, cargo builds into the VM disk. The shell is zsh (oh-my-zsh,
zsh-vi-mode) with the `t` and `re` aliases.

```
lm up               # create and provision the VM (first time)
lm start / lm stop
lm ctest            # any make target runs inside the VM
lm run              # shell in it
lm down             # delete the VM
```

## ft_lex and ft_yacc

Both are usable on their own, they are not tied to `cc1`.
`ft_lex` builds is table-driven DFA Rust lexer generator.
`ft_yacc` is a LALR(1) Rust parser generator with.
