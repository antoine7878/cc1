use crate::common::Unit;

macro_rules! op {
    ($name:ident, $src:expr, $label:expr) => {
        #[test]
        fn $name() {
            let unit = Unit::parse($src);
            assert!(
                unit.parsed(),
                "cc1 failed to parse `{}`:\n{}",
                stringify!($name),
                $src
            );
            let expressions = unit.expressions();
            assert!(
                expressions.iter().any(|e| e == $label),
                "`{}` expected label `{}` in {:?}:\n{}",
                stringify!($name),
                $label,
                expressions,
                $src
            );
        }
    };
}

op!(post_inc, "void f(int a) { a++; }", "post ++");
op!(post_dec, "void f(int a) { a--; }", "post --");
op!(pre_inc, "void f(int a) { ++a; }", "pre ++");
op!(pre_dec, "void f(int a) { --a; }", "pre --");
op!(addr, "void f(int a) { int *p; p = &a; }", "Addr");
op!(deref, "void f(int *p) { int a; a = *p; }", "Deref");
op!(unary_plus, "void f(int a) { int b; b = +a; }", "Plus");
op!(unary_minus, "void f(int a) { int b; b = -a; }", "Minus");
op!(bit_not, "void f(int a) { int b; b = ~a; }", "BitNot");
op!(logical_not, "void f(int a) { int b; b = !a; }", "LogicalNot");

op!(add, "void f(int a, int b) { int c; c = a + b; }", "Add");
op!(sub, "void f(int a, int b) { int c; c = a - b; }", "Sub");
op!(mul, "void f(int a, int b) { int c; c = a * b; }", "Mul");
op!(div, "void f(int a, int b) { int c; c = a / b; }", "Div");
op!(mod_, "void f(int a, int b) { int c; c = a % b; }", "Mod");
op!(left, "void f(int a, int b) { int c; c = a << b; }", "Left");
op!(right, "void f(int a, int b) { int c; c = a >> b; }", "Right");
op!(greater, "void f(int a, int b) { int c; c = a > b; }", "Greater");
op!(lower, "void f(int a, int b) { int c; c = a < b; }", "Lower");
op!(greater_eq, "void f(int a, int b) { int c; c = a >= b; }", "GreaterEq");
op!(lower_eq, "void f(int a, int b) { int c; c = a <= b; }", "LowerEq");
op!(eq, "void f(int a, int b) { int c; c = a == b; }", "Eq");
op!(neq, "void f(int a, int b) { int c; c = a != b; }", "Neq");
op!(bit_and, "void f(int a, int b) { int c; c = a & b; }", "BitAnd");
op!(bit_or, "void f(int a, int b) { int c; c = a | b; }", "BitOr");
op!(bit_xor, "void f(int a, int b) { int c; c = a ^ b; }", "BitXor");
op!(logical_and, "void f(int a, int b) { int c; c = a && b; }", "LogicalAnd");
op!(logical_or, "void f(int a, int b) { int c; c = a || b; }", "LogicalOr");

op!(assign, "void f(int a) { int b; b = a; }", "Assign");
op!(mul_assign, "void f(int a, int b) { a *= b; }", "MulAssign");
op!(div_assign, "void f(int a, int b) { a /= b; }", "DivAssign");
op!(mod_assign, "void f(int a, int b) { a %= b; }", "ModAssign");
op!(add_assign, "void f(int a, int b) { a += b; }", "AddAssign");
op!(sub_assign, "void f(int a, int b) { a -= b; }", "SubAssign");
op!(left_assign, "void f(int a, int b) { a <<= b; }", "LeftAssign");
op!(right_assign, "void f(int a, int b) { a >>= b; }", "RightAssign");
op!(bit_and_assign, "void f(int a, int b) { a &= b; }", "BitAndAssign");
op!(bit_xor_assign, "void f(int a, int b) { a ^= b; }", "BitXorAssign");
op!(bit_or_assign, "void f(int a, int b) { a |= b; }", "BitOrAssign");

op!(
    dot,
    "struct S { int x; }; void f(struct S s) { int a; a = s.x; }",
    "Dot access"
);
op!(
    arrow,
    "struct S { int x; }; void f(struct S *s) { int a; a = s->x; }",
    "Ptr access"
);

op!(array_access, "void f(int a[]) { int b; b = a[0]; }", "Array access");
op!(
    ternary,
    "void f(int a, int b, int c) { int d; d = a ? b : c; }",
    "Ternary"
);
op!(
    function_call,
    "int g(int x); void f(void) { int a; a = g(1); }",
    "Fn call"
);
op!(sizeof_expr, "void f(int a) { int s; s = sizeof(a); }", "Sizeof");
op!(sizeof_type, "void f(void) { int s; s = sizeof(int); }", "Sizeof");
op!(cast, "void f(double d) { int a; a = (int)d; }", "Cast");
op!(comma_list, "void f(int a, int b) { a, b; }", "List");
op!(constant_expression, "enum E { A = 1 };", "ConstantExpression");
