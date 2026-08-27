use crate::semantic::QualifiedType;

#[derive(Clone, Copy, Debug)]
pub struct ImplicitCast {
    pub kind: CastKind,
    pub to: QualifiedType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CastKind {
    LValueToRValue,     // 6.2.2.1
    ArrayToPointer,     // 6.2.2.1
    FunctionToPointer,  // 6.2.2.1
    IntegerPromotion,   // 6.2.1.1
    IntegerConversion,  // 6.2.1.2
    IntegerToFloating,  // 6.2.1.3
    FloatingToInteger,  // 6.2.1.4
    FloatingConversion, // 6.2.1.4
    PointerConversion,  // 6.3.16.1
    NullPointer,        // 6.2.2.3
    ToVoid,             // 6.3.4
}
