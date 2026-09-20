macro_rules! test_case {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() $body
    };
    (ignore $reason:literal, $name:ident, $body:block) => {
        #[test]
        #[ignore = $reason]
        fn $name() $body
    };
}

#[macro_export]
macro_rules! accept {
    ($name:ident, $src:expr) => {
        test_case!($name, {
            $crate::common::run_accept(stringify!($name), $src);
        });
    };
}

#[macro_export]
macro_rules! reject {
    ($name:ident, $src:expr) => {
        test_case!($name, {
            $crate::common::run_reject(stringify!($name), $src);
        });
    };
}

#[macro_export]
macro_rules! syntax {
    ($name:ident, $src:expr) => {
        test_case!($name, {
            $crate::common::run_syntax(stringify!($name), $src);
        });
    };
    (ignore $reason:literal, $name:ident, $src:expr) => {
        test_case!(ignore $reason, $name, {
            $crate::common::run_syntax(stringify!($name), $src);
        });
    };
}

#[macro_export]
macro_rules! value {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_value(stringify!($name), $src, $expected);
        });
    };
}

#[macro_export]
macro_rules! exits {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_exit(stringify!($name), $src, $expected);
        });
    };
    (ignore $reason:literal, $name:ident, $src:expr, $expected:expr) => {
        test_case!(ignore $reason, $name, {
            $crate::common::run_exit(stringify!($name), $src, $expected);
        });
    };
}

#[macro_export]
macro_rules! exits_linked {
    ($name:ident, $src:expr, $helper:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_exit_linked(stringify!($name), $src, $helper, $expected);
        });
    };
}

#[macro_export]
macro_rules! emits {
    ($name:ident, $src:expr, $needle:literal) => {
        test_case!($name, {
            $crate::common::run_emits(stringify!($name), $src, $needle, true);
        });
    };
    (not $name:ident, $src:expr, $needle:literal) => {
        test_case!($name, {
            $crate::common::run_emits(stringify!($name), $src, $needle, false);
        });
    };
}

#[macro_export]
macro_rules! stmts {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_statements(stringify!($name), $src, $expected);
        });
    };
}

#[macro_export]
macro_rules! labels {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_labels(stringify!($name), $src, $expected);
        });
    };
}

#[macro_export]
macro_rules! pool {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_pool(stringify!($name), $src, $expected);
        });
    };
}

#[macro_export]
macro_rules! literal {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_literal(stringify!($name), $src, $expected);
        });
    };
    (ignore $reason:literal, $name:ident, $src:expr, $expected:expr) => {
        test_case!(ignore $reason, $name, {
            $crate::common::run_literal(stringify!($name), $src, $expected);
        });
    };
}

#[macro_export]
macro_rules! recover {
    ($name:ident, $src:expr, [$($diag:pat),* $(,)?], $forbidden:expr) => {
        test_case!($name, {
            let name = stringify!($name);
            let unit = $crate::common::Unit::compile($src);

            let diagnostics = unit.diagnostics();
            let mut got = diagnostics.iter();
            $(
                let next = got.next().map(|diag| diag.inner.clone());
                assert!(
                    matches!(next, Some($diag)),
                    "`{name}` expected {}, got {next:?}:\n{}\n{}",
                    stringify!($diag),
                    $src,
                    unit.render()
                );
            )*
            let extra: Vec<_> = got.map(|diag| diag.inner.clone()).collect();
            assert!(
                extra.is_empty(),
                "`{name}` unexpected extra diagnostic {extra:?}:\n{}\n{}",
                $src,
                unit.render()
            );

            $crate::common::assert_unmentioned(name, $src, &unit, $forbidden);
        });
    };
}

#[macro_export]
macro_rules! size {
    ($name:ident, $decl:expr, $ty:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_size(stringify!($name), $decl, $ty, $expected);
        });
    };
}

#[macro_export]
macro_rules! facts {
    ($name:ident, $src:expr) => {
        test_case!($name, {
            $crate::common::run_facts(stringify!($name), $src);
        });
    };
}

#[macro_export]
macro_rules! bits {
    ($name:ident, $decl:expr, $tag:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_bits(stringify!($name), $decl, $tag, $expected);
        });
    };
}

#[macro_export]
macro_rules! offsets {
    ($name:ident, $decl:expr, $tag:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_offsets(stringify!($name), $decl, $tag, $expected);
        });
    };
}

#[macro_export]
macro_rules! uses {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_uses(stringify!($name), $src, $expected);
        });
    };
}

#[macro_export]
macro_rules! inits {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_initializers(stringify!($name), $src, $expected);
        });
    };
}

#[macro_export]
macro_rules! member_refs {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_member_refs(stringify!($name), $src, $expected);
        });
    };
}

#[macro_export]
macro_rules! placements {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            $crate::common::run_placements(stringify!($name), $src, $expected);
        });
    };
}

#[macro_export]
macro_rules! shaped {
    ($name:ident, $src:expr, $shapes:expr $(,)?) => {
        test_case!($name, {
            let unit = $crate::common::Unit::compile($src);
            assert!(unit.parsed(), "cc1 failed to parse:\n{}", $src);
            assert!(
                unit.diagnostics().is_empty(),
                "unexpected diagnostic:\n{}\n{}",
                $src,
                unit.render()
            );
            let expected: Vec<$crate::common::Shape> = $shapes;
            assert_eq!(unit.shapes(), expected, "{}", $src);
        });
    };
    (ignore $reason:literal, $name:ident, $src:expr, $shapes:expr $(,)?) => {
        test_case!(ignore $reason, $name, {
            let unit = $crate::common::Unit::compile($src);
            assert!(unit.parsed(), "cc1 failed to parse:\n{}", $src);
            let expected: Vec<$crate::common::Shape> = $shapes;
            assert_eq!(unit.shapes(), expected, "{}", $src);
        });
    };
}

#[macro_export]
macro_rules! rejects_shaped {
    (@build $name:ident, $src:expr, $u:ident, $diagnostic:pat, $guard:expr, $shapes:expr) => {
        test_case!($name, {
            let $u = $crate::common::Unit::compile($src);
            assert!($u.parsed(), "cc1 failed to parse:\n{}", $src);
            let got: Vec<_> = $u.diagnostics().iter().map(|diag| diag.inner.clone()).collect();
            assert!(
                matches!(got.as_slice(), [$diagnostic] if $guard),
                "expected one {}, got {got:?}:\n{}\n{}",
                stringify!($diagnostic),
                $src,
                $u.render()
            );
            let expected: Vec<$crate::common::Shape> = $shapes;
            assert_eq!($u.shapes(), expected, "{}", $src);
        });
    };
    ($name:ident, $src:expr, |$u:ident| $diagnostic:pat if $guard:expr, $shapes:expr $(,)?) => {
        rejects_shaped!(@build $name, $src, $u, $diagnostic, $guard, $shapes);
    };
    ($name:ident, $src:expr, $diagnostic:pat, $shapes:expr $(,)?) => {
        rejects_shaped!(@build $name, $src, _u, $diagnostic, true, $shapes);
    };
}

#[macro_export]
macro_rules! folds {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            assert_eq!($crate::common::folded($src), $expected, "{}", $src);
        });
    };
    (ignore $reason:literal, $name:ident, $src:expr, $expected:expr) => {
        test_case!(ignore $reason, $name, {
            assert_eq!($crate::common::folded($src), $expected, "{}", $src);
        });
    };
}

#[macro_export]
macro_rules! folded {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            let expected: [&str; _] = $expected;
            assert_eq!($crate::common::fold_values($src), expected, "{}", $src);
        });
    };
}

#[macro_export]
macro_rules! tree {
    ($name:ident, $src:expr, $symbol:expr, $ty:expr) => {
        test_case!($name, {
            let unit = $crate::common::accepted($src);
            assert_eq!(unit.symbol_ty_tree($symbol), $ty, "{}", $src);
        });
    };
}

#[macro_export]
macro_rules! reports {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            let got = $crate::common::Unit::compile($src).messages();
            assert_eq!(got, $expected, "{:?}", $src);
        });
    };
    (ignore $reason:literal, $name:ident, $src:expr, $expected:expr) => {
        test_case!(ignore $reason, $name, {
            let got = $crate::common::Unit::compile($src).messages();
            assert_eq!(got, $expected, "{:?}", $src);
        });
    };
}

/// A constant whose value is representable by one of the types of its list (6.1.3.2).
#[macro_export]
macro_rules! constant {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            let cc1::semantic::Diag { res: value, diagnostic } = cc1::ast::ConstValue::parse($src);
            assert_eq!($crate::common::repr(Some(value)), $expected, "Value::parse({:?})", $src);
            assert!(diagnostic.is_none(), "ConstValue::parse({:?}) reported {diagnostic:?}", $src);
        });
    };
}

#[macro_export]
macro_rules! too_large {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            let cc1::semantic::Diag { res: value, diagnostic } = cc1::ast::ConstValue::parse($src);
            assert_eq!($crate::common::repr(Some(value)), $expected, "ConstValue::parse({:?})", $src);
            assert!(
                matches!(diagnostic, Some(cc1::semantic::Diagnostic::IntegerConstantTooLarge)),
                "ConstValue::parse({:?}) reported {diagnostic:?}",
                $src
            );
        });
    };
}
