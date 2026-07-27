# Rlex

Implementation of POSIX Lex in Rust.

Lexer generator for C and Rust.

# Features

- Regex parsing to deterministic finite automaton
- DFA compression with character classes (-c)
- C and rust target language (-x <c|rust>)
- Fully POSIX 2004 Edition compliant
- Tested against flex for posix features
- No dependencies

# Rust input file language

## 1. General usage

```rust
%no_main
%tokens Operator Number

%%

[+\*%]        YYToken::Operator
[0-9]*        YYToken::Number
[ \t\n]*      YYToken::Default
.             println!("Unknown token: {}", self.yytext); YYToken::Default
%%

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut lexer = YYLex::default();

    while let Some(token) = lexer.next() {
        match token {
            Ok(YYToken::Operator) => println!("Operator: {}", lexer.yytext),
            Ok(YYToken::Number) => println!("Number: {}", lexer.yytext),
            Ok(YYToken::Default) => println!("OTHER"),
            Ok(YYToken::Reject) => unreachable!(),
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
    Ok(())
}
```

Compared to the original C lex language, the `yylex()` function is replaced with the `YYLex` struct, instantiated through its `new` method. `YYLex` implements the `Default` trait with `stdin`, `stdout`, and no `yycontinue` (a replacement for `yywrap`).

```rust
impl<R: Read, W: Write> YYLex<R, W> {
    pub fn new<F>(yyin: R, yyout: W, yycontinue: F) -> YYLex
    where
        F: FnMut() -> Option<R> + 'static
```

`YYLex` implements `Iterator`. The input file is parsed and tokens are retrieved via `lexer.next()`.

```rust
impl<R: Read, W: Write> Iterator for YYLex<R, W> {
    type Item = Result<YYToken, YYError>;

    fn next(&mut self) -> Option<Self::Item> {
        ...
    }
}


#[derive(Debug)]
pub enum YYError {
    Io(std::io::Error),
    InvalidToken(String),
    AcceptStack(&'static str),
}
```

## 2. Actions

In the rules section, action fragments are written in Rust instead of C. Every action must evaluate to a `YYToken`.

Example:

```lex
salut   print!("SALUT"); YYToken::Default
salut   YYToken::Default
[0-9]+  YYToken::Number
```

New `YYToken` variants are declared in the definition section:

```lex
%tokens Operator Number
```

Base implementation of `YYToken`

```rust
#[derive(Debug, Clone, Copy)]
pub enum YYToken {
    Default,
    Reject,
    /* TOKENS */
}
```

Returning `YYToken::Reject` corresponds to the POSIX `REJECT` macro.

## 5. `YYLex` methods

### 5.1 Start conditions

Start condition changes are done with a method call on the lexer:

```lex
bonjour   self.begin(S); YYToken::Default
```

Start conditions defined with %s and %x are global const, accessible in YYLex and actions.

### 5.2 Global functions

Inside actions, lex runtime operations are accessed through `self` rather than through calls to global functions:

```rust
- `self.yymore()`
- `self.yyless(n)`
- `self.input()`
- `self.unput(b'x')`
- `self.begin(S)`
- `self.echo()`
```

### 5.2 Global variables

Matched text is accessed via `self.yytext`.
No explicit `yyleng` is provided; `self.yytext.len()` should be used instead.
`yyin` and `yyout` are first defined at `YYLex` instantiation and are then accessed via `self`.

## 6. libl

### 6.1 main

A default `main` is provided by default; to remove it, add `%no_main` in the definition section.

### 6.1 yywrap

The `yywrap` equivalent is now a member of `YYLex`, provided during instantiation as `yycontinue: Box<dyn FnMut() -> Option<R>>`, with `R` being the type of `self.yyout`.
`yycontinue` should return `None` to end parsing at the end of the current file, or a reader to continue parsing.
