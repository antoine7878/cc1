mod common;

// ---- 6.5.2.1 scalar members are laid out in declaration order -------------

size!(size_single_char, "struct S { char a; };", "struct S", 1);

size!(size_single_short, "struct S { short a; };", "struct S", 2);

size!(size_single_int, "struct S { int a; };", "struct S", 4);

size!(size_single_double, "struct S { double a; };", "struct S", 8);

size!(size_single_pointer, "struct S { int *p; };", "struct S", 4);

size!(size_three_chars, "struct S { char a; char b; char c; };", "struct S", 3);

size!(size_two_ints, "struct S { int a; int b; };", "struct S", 8);

// ---- a member is padded up to its own alignment ---------------------------

size!(size_char_then_short, "struct S { char a; short b; };", "struct S", 4);

size!(size_char_then_int, "struct S { char a; int b; };", "struct S", 8);

size!(
    size_char_char_then_int,
    "struct S { char a; char b; int c; };",
    "struct S",
    8
);

size!(size_short_then_int, "struct S { short a; int b; };", "struct S", 8);

size!(size_char_then_double, "struct S { char a; double b; };", "struct S", 12);

size!(size_char_then_pointer, "struct S { char a; int *p; };", "struct S", 8);

// ---- the struct is padded up to the alignment of its widest member --------

size!(size_int_then_char, "struct S { int a; char b; };", "struct S", 8);

size!(size_short_then_char, "struct S { short a; char b; };", "struct S", 4);

size!(size_double_then_char, "struct S { double a; char b; };", "struct S", 12);

size!(size_pointer_then_char, "struct S { int *p; char c; };", "struct S", 8);

size!(
    size_char_int_char,
    "struct S { char a; int b; char c; };",
    "struct S",
    12
);

size!(
    size_long_double_then_char,
    "struct S { long double a; char b; };",
    "struct S",
    16
);

// ---- 6.5.2.1 a union is as large as its widest member ---------------------

size!(size_union_char_int, "union U { char a; int b; };", "union U", 4);

size!(size_union_char_double, "union U { char a; double b; };", "union U", 8);

size!(size_union_single_char, "union U { char a; };", "union U", 1);

// size!(
//     size_union_padded_to_alignment,
//     "union U { char a[7]; short b; };",
//     "union U",
//     8
// );

// ---- array members contribute their whole extent -------------------------

// size!(size_array_of_char, "struct S { char a[3]; };", "struct S", 3);
// size!(size_array_then_int, "struct S { char a[3]; int b; };", "struct S", 8);
// size!(size_array_of_int, "struct S { int a[4]; };", "struct S", 16);

// ---- an aggregate member keeps its own layout ----------------------------

size!(
    size_nested_struct,
    "struct I { char a; int b; }; struct S { char x; struct I i; };",
    "struct S",
    12
);

size!(
    size_nested_struct_of_chars,
    "struct I { char a; char b; }; struct S { struct I i; char c; };",
    "struct S",
    3
);

size!(
    size_nested_union,
    "union U { char a; int b; }; struct S { char x; union U u; };",
    "struct S",
    8
);

// ---- 6.5.2.2 an enumeration has the size of int --------------------------

size!(size_enum, "enum E { A };", "enum E", 4);

size!(
    size_struct_of_enum,
    "enum E { A }; struct S { char c; enum E e; };",
    "struct S",
    8
);
