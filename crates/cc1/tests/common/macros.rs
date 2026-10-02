macro_rules! valid {
    ($(#[$attr:meta])* $name:ident, $source:expr, $exit:expr, $stdout:expr $(, $option:ident = $value:expr)* $(,)?) => {
        $(#[$attr])*
        #[test]
        fn $name() {
            let options = $crate::common::Options { $($option: $value,)* ..Default::default() };
            $crate::common::valid(stringify!($name), $source, $exit, $stdout, options);
        }
    };
}

macro_rules! invalid {
    ($(#[$attr:meta])* $name:ident, $source:expr, $diagnostics:expr $(, $option:ident = $value:expr)* $(,)?) => {
        $(#[$attr])*
        #[test]
        fn $name() {
            let options = $crate::common::Options { $($option: $value,)* ..Default::default() };
            $crate::common::invalid(stringify!($name), $source, $diagnostics, options);
        }
    };
}
