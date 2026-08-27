use crate::semantic::QualifiedType;

#[derive(Clone, Copy, Debug)]
pub struct ImplicitCast {
    pub kind: CastKind,
    pub to: QualifiedType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CastKind {
    // A char, a short int. or an int bit-field. or their signed or unsigned varieties. or an
    // enumeration type. may be used in an expression wherever an int or unsigned int may be
    // used If an int can represent all values of the original type. the value is converted to an inf;
    // otherwise, it is converted to an unsigned int. These are called the integral p~wnotions.”
    // All other arithmetic types are unchanged by the integral promotions.
    IntegerPromotion,   // 6.2.1.1
    IntegerConversion,  // 6.2.1.2
    IntegerToFloating,  // 6.2.1.3
    FloatingToInteger,  // 6.2.1.4
    FloatingConversion, // 6.2.1.4
    FunctionToPointer,  // 6.2.2.1
    LValueToRValue,     // 6.2.2.1
    ArrayToPointer,     // 6.2.2.1
    NullPointer,        // 6.2.2.3
    ToVoid,             // 6.3.4
    PointerConversion,  // 6.3.16.1
}

// #[derive(Clone, Copy, Debug)]
// pub struct Typed { pub id: ExpressionId,
//     pub ty: QualifiedType, // after the casts pushed so far
//     pub kind: ExpressionKind,
// }
//
// pub fn lvalue(sema: &mut Sema, expr: &mut Typed) {
//     // 6.2.2.1
// }
//
// pub fn promote(sema: &mut Sema, expr: &mut Typed) {
//     // 6.2.1.1
// }
//
// pub fn usual_arithmetic(sema: &mut Sema, lhs: &mut Typed, rhs: &mut Typed) -> QualifiedType {
//     // 6.2.1.5
// }
//
// // convert is the dispatcher — the only place a (from, to) pair turns into a CastKind:
// pub fn convert(sema: &mut Sema, expr: &mut Typed, to: QualifiedType) {}
//
// // fn promote() {}
// // fn lvalue_conversion() {} fn usual_arithmetic() {}
// // fn assignment_conversion() {}
// // fn default_argument_promotions() {}
