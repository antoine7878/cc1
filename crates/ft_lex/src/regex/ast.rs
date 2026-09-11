use crate::error::LexError;
use crate::regex::{Token, Tokenizer};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Ast {
    pub expression: Expression,
    pub start_anchor: bool,
    pub end_anchor: bool,
}

impl TryFrom<&mut Tokenizer> for Ast {
    type Error = LexError;

    fn try_from(tokenizer: &mut Tokenizer) -> Result<Self, Self::Error> {
        if tokenizer.peek().is_none() {
            return Ok(Self { expression: Expression::Empty, start_anchor: false, end_anchor: false });
        }
        let start_anchor = tokenizer.is_start_anchor();
        if start_anchor {
            tokenizer.next();
        }

        let mut expression = Expression::try_from(&mut *tokenizer)?;

        let end_anchor = if tokenizer.peek() == Some(Token::Dollar) {
            tokenizer.next();
            true
        } else {
            false
        };

        match tokenizer.next() {
            Some(Token::Slash) => {
                let trail = Expression::try_from(&mut *tokenizer)?;
                expression = Expression::TrailingContext(Box::new(expression), Box::new(trail))
            }
            Some(t) => return Err(LexError::AstParsing(format!("remaining tokens: {t}"))),
            None => (),
        };

        Ok(Self { expression, start_anchor, end_anchor })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Expression {
    #[default]
    Empty,
    Atom(Atom),
    Disjunction(Vec<Expression>),
    Concat(Vec<Expression>),
    Repeat(Box<Expression>, usize, Option<usize>),
    TrailingContext(Box<Expression>, Box<Expression>),
}

impl TryFrom<&mut Tokenizer> for Expression {
    type Error = LexError;

    fn try_from(tokenizer: &mut Tokenizer) -> Result<Self, Self::Error> {
        let mut disjunctions: Vec<Expression> = Vec::new();

        disjunctions.push(Self::parse_concat(tokenizer)?);
        while let Some(Token::Pipe) = tokenizer.peek() {
            tokenizer.next();
            disjunctions.push(Self::parse_concat(tokenizer)?);
        }

        if disjunctions.len() == 1 {
            Ok(disjunctions.pop().unwrap_or(Expression::Empty))
        } else {
            if matches!(disjunctions.last(), Some(Expression::Empty)) {
                return Err(LexError::AstParsing("| without left side".to_string()));
            }
            Ok(Expression::Disjunction(disjunctions.into_iter().collect::<_>()))
        }
    }
}

impl Expression {
    fn parse_concat(tokenizer: &mut Tokenizer) -> Result<Expression, LexError> {
        let mut concats: Vec<Expression> = Vec::new();

        while let Some(tok) = tokenizer.peek()
            && tok != Token::ClosePar
            && tok != Token::Pipe
            && tok != Token::Slash
            && tok != Token::Dollar
        {
            concats.push(Self::parse_repeat(tokenizer)?);
        }

        match concats.len() {
            0 => Ok(Expression::Empty),
            1 => Ok(concats.pop().unwrap_or(Expression::Empty)),
            _ => Ok(Expression::Concat(concats.into_iter().collect::<Vec<_>>())),
        }
    }

    fn parse_repeat(tokenizer: &mut Tokenizer) -> Result<Expression, LexError> {
        let base = Expression::Atom(Atom::try_from(&mut *tokenizer)?);

        if let Some(tok) = tokenizer.peek()
            && tok.is_duplication()
        {
            let (min, max) = Expression::try_duplication(tokenizer)?;
            Ok(Expression::Repeat(Box::new(base), min, max))
        } else {
            Ok(base)
        }
    }

    fn try_duplication(tokenizer: &mut Tokenizer) -> Result<(usize, Option<usize>), LexError> {
        match tokenizer.next() {
            Some(Token::Star) => Ok((0, None)),
            Some(Token::Plus) => Ok((1, None)),
            Some(Token::Question) => Ok((0, Some(1))),
            Some(Token::OpenCurly) => Self::parse_range(tokenizer),
            _ => Err(LexError::AstParsing("failed parsing duplication".to_string())),
        }
    }

    pub fn parse_range(tokenizer: &mut Tokenizer) -> Result<(usize, Option<usize>), LexError> {
        let min = tokenizer.next_number()?;
        let max = match tokenizer.peek() {
            Some(Token::Byte(b',')) => {
                tokenizer.next();
                match tokenizer.peek() {
                    Some(Token::CloseCurly) => None,
                    _ => Some(tokenizer.next_number()?),
                }
            }
            _ => Some(min),
        };

        if let Some(max) = max
            && (min > max || max == 0)
        {
            return Err(LexError::AstParsing("bad iteration values".to_string()));
        }
        match tokenizer.next() {
            Some(Token::CloseCurly) => Ok((min, max)),
            None => Err(LexError::AstParsing("unclosed }}".to_string())),
            Some(tok) => Err(LexError::AstParsing(format!("Oupsi {}", u8::from(tok)))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Atom {
    Literal(u8),
    AnyByte,
    Group(Box<Expression>),
    Bracket(BracketExpr),
}

impl TryFrom<&mut Tokenizer> for Atom {
    type Error = LexError;

    fn try_from(tokenizer: &mut Tokenizer) -> Result<Self, Self::Error> {
        match tokenizer.next() {
            Some(Token::Byte(c)) => Ok(Self::Literal(c)),
            Some(Token::Dot) => Ok(Self::AnyByte),
            Some(Token::OpenPar) => {
                let exp = Self::Group(Box::new(Expression::try_from(&mut *tokenizer)?));

                if tokenizer.next() != Some(Token::ClosePar) {
                    return Err(LexError::AstParsing("( not closed".to_string()));
                }
                Ok(exp)
            }
            Some(Token::OpenBracket) => Ok(Self::Bracket(BracketExpr::try_from(tokenizer)?)),
            Some(tok) => Ok(Self::Literal(u8::from(tok))),
            _ => Err(LexError::AstParsing("errror parsing Atom".to_string())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BracketExpr {
    pub negated: bool,
    pub items: Vec<BracketItem>,
}

impl TryFrom<&mut Tokenizer> for BracketExpr {
    type Error = LexError;

    fn try_from(tokenizer: &mut Tokenizer) -> Result<Self, Self::Error> {
        let mut negated = false;
        if let Some(Token::Byte(b'^')) = tokenizer.peek() {
            tokenizer.next();
            negated = true
        }
        let mut items: Vec<BracketItem> = Vec::new();
        let new_items: Vec<BracketItem> = (&mut *tokenizer).try_into()?;
        items.extend(new_items);
        while let Some(tok) = tokenizer.peek() {
            if tok == Token::CloseBracket {
                tokenizer.next();
                break;
            }
            let new_items: Vec<BracketItem> = (&mut *tokenizer).try_into()?;
            items.extend(new_items);
        }
        Ok(Self { negated, items })
    }
}

impl TryFrom<&mut Tokenizer> for Vec<BracketItem> {
    type Error = LexError;
    fn try_from(tokenizer: &mut Tokenizer) -> Result<Self, Self::Error> {
        let first = tokenizer.next();
        match (first, tokenizer.peek()) {
            (Some(Token::OpenBracket), Some(Token::Byte(b':'))) => {
                tokenizer.next();
                let name = tokenizer
                    .next_name(Token::Byte(b':'), Token::CloseBracket)
                    .ok_or(LexError::AstParsing("class name not found".to_string()))?;
                Ok(vec![BracketItem::Class(PosixClass::try_from(name)?)])
            }
            (Some(Token::OpenBracket), Some(Token::Byte(b'.'))) => {
                tokenizer.next();
                let name = tokenizer
                    .next_name(Token::Byte(b'.'), Token::CloseBracket)
                    .ok_or(LexError::AstParsing("collation not found".to_string()))?;
                Ok(vec![BracketItem::Collation(name)])
            }
            (Some(Token::OpenBracket), Some(Token::Byte(b'='))) => {
                tokenizer.next();
                let name = tokenizer
                    .next_name(Token::Byte(b'='), Token::CloseBracket)
                    .ok_or(LexError::AstParsing("Equivalence class not found".to_string()))?;
                Ok(vec![BracketItem::Equivalence(name)])
            }
            (Some(Token::Byte(start)), Some(Token::Byte(b'-'))) => {
                tokenizer.next();
                match tokenizer.peek() {
                    Some(Token::CloseBracket) => Ok(vec![BracketItem::Byte(start), BracketItem::Byte(b'-')]),
                    Some(tok) => {
                        let end = u8::from(tok);
                        if end < start {
                            return Err(LexError::AstParsing(format!("invalid range {start} > {end}")));
                        }
                        Ok(vec![BracketItem::Range(start, end)])
                    }
                    _ => Err(LexError::AstParsing("unclosed [".to_string())),
                }
            }
            (Some(tok), _) => Ok(vec![BracketItem::Byte(u8::from(tok))]),
            _ => Err(LexError::AstParsing("could not parse barcket item".into())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BracketItem {
    Byte(u8),
    Range(u8, u8),
    Class(PosixClass),
    Collation(Vec<u8>),
    Equivalence(Vec<u8>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum PosixClass {
    #[default]
    Alnum,
    Alpha,
    Ascii,
    Blank,
    Cntrl,
    Digit,
    Graph,
    Lower,
    Print,
    Punct,
    Space,
    Upper,
    Word,
    Xdigit,
}

impl TryFrom<Vec<u8>> for PosixClass {
    type Error = LexError;

    fn try_from(name: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(match name.as_slice() {
            b"alnum" => Self::Alnum,
            b"alpha" => Self::Alpha,
            b"ascii" => Self::Ascii,
            b"blank" => Self::Blank,
            b"cntrl" => Self::Cntrl,
            b"digit" => Self::Digit,
            b"graph" => Self::Graph,
            b"lower" => Self::Lower,
            b"print" => Self::Print,
            b"punct" => Self::Punct,
            b"space" => Self::Space,
            b"upper" => Self::Upper,
            b"word" => Self::Word,
            b"xdigit" => Self::Xdigit,
            s => {
                return Err(LexError::AstParsing(format!(
                    "{} not a posix class",
                    String::from_utf8(s.to_vec()).unwrap()
                )));
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::assert_panics;

    fn test_ok(line: &str, expected: Ast) {
        let mut tokenizer = Tokenizer::from(line);
        let expr = Ast::try_from(&mut tokenizer).unwrap();

        assert_eq!(expr, expected);
    }

    #[test]
    fn basic() {
        test_ok(
            "a}",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Concat(vec![
                    Expression::Atom(Atom::Literal(b'a')),
                    Expression::Atom(Atom::Literal(b'}')),
                ]),
            },
        );

        test_ok(
            "{a}",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Concat(vec![
                    Expression::Atom(Atom::Literal(b'{')),
                    Expression::Atom(Atom::Literal(b'a')),
                    Expression::Atom(Atom::Literal(b'}')),
                ]),
            },
        );

        test_ok(
            "{1}",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Concat(vec![
                    Expression::Atom(Atom::Literal(b'{')),
                    Expression::Atom(Atom::Literal(b'1')),
                    Expression::Atom(Atom::Literal(b'}')),
                ]),
            },
        );
    }

    #[test]
    fn quotes() {
        test_ok(
            r#""a""#,
            Ast { start_anchor: false, end_anchor: false, expression: Expression::Atom(Atom::Literal(b'a')) },
        )
    }

    #[rustfmt::skip]
    #[test]
    fn duplication() {
        test_ok(
            "b{42}",
            Ast {
                start_anchor: false,
                end_anchor: false,
            expression: Expression::Repeat(Box::new(Expression::Atom(Atom::Literal(b'b'))), 42, Some(42)),
            }
        );
        test_ok(
            "b{6}",
            Ast {
                start_anchor: false,
                end_anchor: false,
            expression: Expression::Repeat(Box::new(Expression::Atom(Atom::Literal(b'b'))), 6, Some(6)),
            }
        );
        test_ok(
            "b{100}",
            Ast {
                start_anchor: false,
                end_anchor: false,
            expression: Expression::Repeat(Box::new(Expression::Atom(Atom::Literal(b'b'))), 100, Some(100)),
            }
        );
        test_ok(
            "b{100,}",
            Ast {
                start_anchor: false,
                end_anchor: false,
            expression: Expression::Repeat(Box::new(Expression::Atom(Atom::Literal(b'b'))), 100, None),
            }
        );
        test_ok(
            "b{4,9}",
            Ast {
                start_anchor: false,
                end_anchor: false,
            expression: Expression::Repeat(Box::new(Expression::Atom(Atom::Literal(b'b'))), 4, Some(9)),
            }
        );
    }

    #[test]
    fn brackets_basic() {
        test_ok(
            "[a]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: false,
                    items: vec![BracketItem::Byte(b'a')],
                })),
            },
        );

        test_ok(
            "[^a]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: true,
                    items: vec![BracketItem::Byte(b'a')],
                })),
            },
        );

        test_ok(
            "[abc]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: false,
                    items: vec![BracketItem::Byte(b'a'), BracketItem::Byte(b'b'), BracketItem::Byte(b'c')],
                })),
            },
        );
    }

    #[test]
    fn brackets_classes() {
        test_ok(
            "[[:alnum:]]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: false,
                    items: vec![BracketItem::Class(PosixClass::Alnum)],
                })),
            },
        );
        test_ok(
            "[[.alnum.]]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: false,
                    items: vec![BracketItem::Collation("alnum".into())],
                })),
            },
        );

        test_ok(
            "[[=alnum=]]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: false,
                    items: vec![BracketItem::Equivalence("alnum".into())],
                })),
            },
        );
        test_ok(
            "[^abc]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: true,
                    items: vec![BracketItem::Byte(b'a'), BracketItem::Byte(b'b'), BracketItem::Byte(b'c')],
                })),
            },
        );
        test_ok(
            "[^[:alnum:]]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: true,
                    items: vec![BracketItem::Class(PosixClass::Alnum)],
                })),
            },
        );
        test_ok(
            "[^[.alnum.]]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: true,
                    items: vec![BracketItem::Collation("alnum".into())],
                })),
            },
        );
        test_ok(
            "[^[=alnum=]]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: true,
                    items: vec![BracketItem::Equivalence("alnum".into())],
                })),
            },
        );
    }

    #[test]
    fn brackets_literal() {
        test_ok(
            "[-]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: false,
                    items: vec![BracketItem::Byte(b'-')],
                })),
            },
        );
        test_ok(
            "[a-]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: false,
                    items: vec![BracketItem::Byte(b'a'), BracketItem::Byte(b'-')],
                })),
            },
        );
        test_ok(
            "[-a]",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Bracket(BracketExpr {
                    negated: false,
                    items: vec![BracketItem::Byte(b'-'), BracketItem::Byte(b'a')],
                })),
            },
        );
    }

    #[test]
    fn brackets_group() {
        test_ok(
            "(abc)",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Atom(Atom::Group(Box::new(Expression::Concat(vec![
                    Expression::Atom(Atom::Literal(b'a')),
                    Expression::Atom(Atom::Literal(b'b')),
                    Expression::Atom(Atom::Literal(b'c')),
                ])))),
            },
        );
        test_ok(
            "(ab)(cd)",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Concat(vec![
                    Expression::Atom(Atom::Group(Box::new(Expression::Concat(vec![
                        Expression::Atom(Atom::Literal(b'a')),
                        Expression::Atom(Atom::Literal(b'b')),
                    ])))),
                    Expression::Atom(Atom::Group(Box::new(Expression::Concat(vec![
                        Expression::Atom(Atom::Literal(b'c')),
                        Expression::Atom(Atom::Literal(b'd')),
                    ])))),
                ]),
            },
        );
        test_ok(
            "(ab)|(cd)",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::Disjunction(vec![
                    Expression::Atom(Atom::Group(Box::new(Expression::Concat(vec![
                        Expression::Atom(Atom::Literal(b'a')),
                        Expression::Atom(Atom::Literal(b'b')),
                    ])))),
                    Expression::Atom(Atom::Group(Box::new(Expression::Concat(vec![
                        Expression::Atom(Atom::Literal(b'c')),
                        Expression::Atom(Atom::Literal(b'd')),
                    ])))),
                ]),
            },
        );
    }

    #[test]
    fn trailing_context() {
        test_ok(
            "a/b",
            Ast {
                start_anchor: false,
                end_anchor: false,
                expression: Expression::TrailingContext(
                    Box::new(Expression::Atom(Atom::Literal(b'a'))),
                    Box::new(Expression::Atom(Atom::Literal(b'b'))),
                ),
            },
        )
    }

    #[test]
    fn anchor() {
        test_ok(
            "^abc$",
            Ast {
                start_anchor: true,
                end_anchor: true,
                expression: Expression::Concat(vec![
                    Expression::Atom(Atom::Literal(b'a')),
                    Expression::Atom(Atom::Literal(b'b')),
                    Expression::Atom(Atom::Literal(b'c')),
                ]),
            },
        )
    }

    fn test_build(line: &str) {
        println!("{}", line);
        let mut tokenizer = Tokenizer::from(line);
        let _expr = Expression::try_from(&mut tokenizer).unwrap();
        println!("{:?}", _expr);
    }

    #[test]
    fn errors() {
        assert_panics(|| test_build("[[:salut:]"));
        assert_panics(|| test_build("a|"));
    }
}
