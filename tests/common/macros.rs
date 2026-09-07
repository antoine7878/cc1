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

            let mut got = unit.diagnosis().iter();
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
                "`{name}` unexpected extra diagnosis {extra:?}:\n{}\n{}",
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
                unit.diagnosis().is_empty(),
                "unexpected diagnosis:\n{}\n{}",
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
    (@build $name:ident, $src:expr, $u:ident, $diagnosis:pat, $guard:expr, $shapes:expr) => {
        test_case!($name, {
            let $u = $crate::common::Unit::compile($src);
            assert!($u.parsed(), "cc1 failed to parse:\n{}", $src);
            let got: Vec<_> = $u.diagnosis().iter().map(|diag| diag.inner.clone()).collect();
            assert!(
                matches!(got.as_slice(), [$diagnosis] if $guard),
                "expected one {}, got {got:?}:\n{}\n{}",
                stringify!($diagnosis),
                $src,
                $u.render()
            );
            let expected: Vec<$crate::common::Shape> = $shapes;
            assert_eq!($u.shapes(), expected, "{}", $src);
        });
    };
    ($name:ident, $src:expr, |$u:ident| $diagnosis:pat if $guard:expr, $shapes:expr $(,)?) => {
        rejects_shaped!(@build $name, $src, $u, $diagnosis, $guard, $shapes);
    };
    ($name:ident, $src:expr, $diagnosis:pat, $shapes:expr $(,)?) => {
        rejects_shaped!(@build $name, $src, _u, $diagnosis, true, $shapes);
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
            let cc1::semantic::Diag {
                res: value,
                diagnosis,
            } = cc1::ast::Value::parse($src, &cc1::target::I386);
            assert_eq!(
                $crate::common::repr(Some(value)),
                $expected,
                "Value::parse({:?})",
                $src
            );
            assert!(
                diagnosis.is_none(),
                "Value::parse({:?}) reported {diagnosis:?}",
                $src
            );
        });
    };
}

#[macro_export]
macro_rules! too_large {
    ($name:ident, $src:expr, $expected:expr) => {
        test_case!($name, {
            let cc1::semantic::Diag {
                res: value,
                diagnosis,
            } = cc1::ast::Value::parse($src, &cc1::target::I386);
            assert_eq!(
                $crate::common::repr(Some(value)),
                $expected,
                "Value::parse({:?})",
                $src
            );
            assert!(
                matches!(diagnosis, Some(cc1::semantic::Diagnosis::IntegerConstantTooLarge)),
                "Value::parse({:?}) reported {diagnosis:?}",
                $src
            );
        });
    };
}
