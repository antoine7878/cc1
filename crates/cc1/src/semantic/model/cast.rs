use std::fmt;

use libft::Span;

use crate::ast::{self};
use crate::semantic::ValueCategory::RValue;
use crate::semantic::{
    Diag, Diagnostic, DiagnosticSink, QualifiedType, ResolvedExpression, ResolvedType, ResolvedTypeId, Sema,
    ValueCategory, layout,
};

#[derive(Clone, Copy, Debug)]
pub struct ImplicitCast {
    pub kind: CastKind,
    pub to: QualifiedType,
}
impl ImplicitCast {
    fn new(kind: CastKind, to: QualifiedType) -> Self {
        Self { kind, to }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastKind {
    IntegerPromotion,   // 6.2.1.1
    IntegerConversion,  // 6.2.1.2
    IntegerToFloating,  // 6.2.1.3
    FloatingConversion, // 6.2.1.4
    FloatingToInteger,  // 6.2.1.4

    FunctionToPointer, // 6.2.2.1
    LValueToRValue,    // 6.2.2.1
    ArrayToPointer,    // 6.2.2.1

    NullPointer,       // 6.2.2.3
    ToVoid,            // 6.3.4
    PointerToInteger,  // 6.3.4
    IntegerToPointer,  // 6.3.4
    PointerConversion, // 6.3.16.1
}

pub fn convert_operand(sema: &mut Sema, re: &mut ResolvedExpression, span: &Span) {
    function_to_pointer(sema, re);
    array_to_pointer(sema, re);
    lvalue_to_rvalue(sema, re, span);
}

pub fn function_to_pointer(sema: &mut Sema, re: &mut ResolvedExpression) {
    let ResolvedType::Function { .. } = re.ty.id.resolve_with(sema) else { return };
    let to = QualifiedType::plain(sema.types.pointer(re.ty));
    re.casts.push(ImplicitCast::new(CastKind::FunctionToPointer, to));
}

pub fn array_to_pointer(sema: &mut Sema, re: &mut ResolvedExpression) {
    let ResolvedType::Array { elem, .. } = re.ty.id.resolve_with(sema) else { return };
    let to = QualifiedType::plain(sema.types.pointer(*elem));
    re.casts.push(ImplicitCast::new(CastKind::ArrayToPointer, to))
}

pub fn lvalue_to_rvalue(sema: &mut Sema, re: &mut ResolvedExpression, span: &Span) {
    if !matches!(re.kind, ValueCategory::LValue) {
        return;
    }
    let ty = re.ty.id.resolve_with(sema);
    if matches!(ty, ResolvedType::Array { .. } | ResolvedType::Function { .. }) {
        return;
    }
    if !ty.is_complete(sema) {
        sema.add_diag(Diag::err((), Diagnostic::IncompleteType(re.ty)), span);
    }
    let to = QualifiedType::plain(re.ty.id);
    re.casts.push(ImplicitCast::new(CastKind::LValueToRValue, to));
}

pub fn promote(sema: &Sema, re: &mut ResolvedExpression) {
    match re.casted_ty().id.resolve_with(sema) {
        ResolvedType::Char
        | ResolvedType::SignedChar
        | ResolvedType::UnsignedChar
        | ResolvedType::Short
        | ResolvedType::UnsignedShort => (),
        ResolvedType::UnsignedInt if re.bit_width.is_some_and(|w| w < (layout::INT.size * layout::CHAR_BIT) as i32) => (),
        &ResolvedType::Tag(id) if id.resolve_with(sema).kind == ast::Tag::Enum => (),
        _ => return,
    }

    let kind = CastKind::IntegerPromotion;
    let to = QualifiedType::plain(sema.builtins.int);
    re.casts.push(ImplicitCast::new(kind, to));
}

pub fn convert(sema: &Sema, from_re: &mut ResolvedExpression, to_id: ResolvedTypeId, is_null_ptr: bool) {
    let from = from_re.casted_ty();
    let implicit_cast = match (from.id.resolve_with(sema), to_id.resolve_with(sema)) {
        _ if from.id == to_id => return,
        (f, t) if f.is_arithmetic(sema) && t.is_arithmetic(sema) => return num_conv(sema, from_re, to_id),
        (_, ResolvedType::Void) => ImplicitCast::new(CastKind::ToVoid, QualifiedType::plain(to_id)),
        (_, ResolvedType::Pointer(_)) if is_null_ptr => {
            ImplicitCast::new(CastKind::NullPointer, QualifiedType::plain(to_id))
        }
        (ResolvedType::Pointer(_), ResolvedType::Pointer(_)) => {
            ImplicitCast::new(CastKind::PointerConversion, QualifiedType::plain(to_id))
        }
        (ResolvedType::Pointer(_), t) if t.is_integral(sema) => {
            ImplicitCast::new(CastKind::PointerToInteger, QualifiedType::plain(to_id))
        }
        (f, ResolvedType::Pointer(_)) if f.is_integral(sema) => {
            ImplicitCast::new(CastKind::IntegerToPointer, QualifiedType::plain(to_id))
        }
        _ => unreachable!(),
    };
    from_re.casts.push(implicit_cast)
}

pub fn to_void(sema: &Sema, re: &mut ResolvedExpression) {
    let cast = ImplicitCast::new(CastKind::ToVoid, QualifiedType::plain(sema.builtins.void));
    re.casts.push(cast);
}

fn num_conv(sema: &Sema, from_re: &mut ResolvedExpression, to_id: ResolvedTypeId) {
    if let Some(cast) = arithmetic_conversion(sema, from_re.casted_ty(), QualifiedType::plain(to_id)) {
        from_re.casts.push(cast);
    }
}

pub fn arithmetic_conversion(sema: &Sema, from: QualifiedType, to: QualifiedType) -> Option<ImplicitCast> {
    if from.id == to.id {
        return None;
    }
    let (f, t) = (from.id.resolve_with(sema), to.id.resolve_with(sema));
    let kind = match (f.is_integral(sema), t.is_integral(sema)) {
        (true, true) => CastKind::IntegerConversion,
        (true, false) => CastKind::IntegerToFloating,
        (false, true) => CastKind::FloatingToInteger,
        (false, false) => CastKind::FloatingConversion,
    };
    Some(ImplicitCast::new(kind, QualifiedType::plain(to.id)))
}

pub fn usual_arithmetic(
    sema: &mut Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
) -> Result<(QualifiedType, ValueCategory), Diagnostic> {
    use ResolvedType as R;

    let l = lhs.casted_ty().id.resolve_with(sema);
    let r = rhs.casted_ty().id.resolve_with(sema);
    if !l.is_arithmetic(sema) || !r.is_arithmetic(sema) {
        return Ok((lhs.casted_ty(), RValue));
    }
    if l.is_integral(sema) && r.is_integral(sema) {
        promote(sema, lhs);
        promote(sema, rhs);
    }
    if lhs.casted_ty().id == rhs.casted_ty().id {
        return Ok((lhs.casted_ty(), RValue));
    }
    let to = match (l, r) {
        (R::LongDouble, _) | (_, R::LongDouble) => sema.builtins.long_double,
        (R::Double, _) | (_, R::Double) => sema.builtins.double,
        (R::Float, _) | (_, R::Float) => sema.builtins.float,
        (_, R::UnsignedLong) | (R::UnsignedLong, _) => sema.builtins.unsigned_long,
        (R::UnsignedInt, R::Long) | (R::Long, R::UnsignedInt) => sema.builtins.unsigned_long,
        (_, R::Long) | (R::Long, _) => sema.builtins.long,
        (_, R::UnsignedInt) | (R::UnsignedInt, _) => sema.builtins.unsigned_int,
        _ => sema.builtins.int,
    };
    num_conv(sema, lhs, to);
    num_conv(sema, rhs, to);
    Ok((lhs.casted_ty(), RValue))
}

fn is_object_or_incomplete(sema: &Sema, ty: ResolvedTypeId) -> bool {
    !matches!(ty.resolve_with(sema), ResolvedType::Function { .. } | ResolvedType::Void)
}

fn can_assign_pointer(sema: &Sema, lp: QualifiedType, rp: QualifiedType) -> bool {
    lp.is_compatible_ignoring_qualifiers(sema, &rp)
        || (lp.id == sema.builtins.void && is_object_or_incomplete(sema, rp.id))
        || (rp.id == sema.builtins.void && is_object_or_incomplete(sema, lp.id))
}

#[derive(Clone, Copy, Debug)]
pub enum AssignmentContext {
    Return,
    Assignment,
    Argument(usize),
    Initialization,
}

impl AssignmentContext {
    pub fn discarded(&self, to: QualifiedType, from: QualifiedType) -> Diagnostic {
        match self {
            AssignmentContext::Return => Diagnostic::ReturnDiscardedQualifiers(to, from),
            AssignmentContext::Assignment => Diagnostic::AssignmentDiscardedQualifiers(to, from),
            AssignmentContext::Argument(n) => Diagnostic::ArgumentDiscardedQualifiers(*n, to, from),
            AssignmentContext::Initialization => Diagnostic::InitDiscardedQualifiers(to, from),
        }
    }

    pub fn incompatible(&self, to: QualifiedType, from: QualifiedType) -> Diagnostic {
        match self {
            AssignmentContext::Return => Diagnostic::ReturnIncompatibleTypes(to, from),
            AssignmentContext::Assignment => Diagnostic::AssignmentIncompatibleTypes(to, from),
            AssignmentContext::Argument(n) => Diagnostic::ArgumentIncompatibleTypes(*n, to, from),
            AssignmentContext::Initialization => Diagnostic::InitIncompatibleTypes(to, from),
        }
    }
}

pub fn assignment_conversion(
    sema: &mut Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
    is_null_ptr: bool,
    assign_ctx: AssignmentContext,
) -> Result<QualifiedType, Diagnostic> {
    let l = lhs.ty.id.resolve_with(sema);
    let rhs_ty = rhs.casted_ty();
    let r = rhs_ty.id.resolve_with(sema);
    match (l, r) {
        (l, r) if l.is_arithmetic(sema) && r.is_arithmetic(sema) => (),
        (&ResolvedType::Tag(id), _) if !id.resolve_with(sema).is_enum() && lhs.ty.is_compatible(sema, &rhs_ty) => (),
        (ResolvedType::Pointer(lp), ResolvedType::Pointer(rp)) if can_assign_pointer(sema, *lp, *rp) => {
            if !lp.has_qualifiers_of(rp) {
                return Err(assign_ctx.discarded(lhs.ty, rhs_ty));
            }
        }
        (ResolvedType::Pointer(_), _) if is_null_ptr => (),
        _ => return Err(assign_ctx.incompatible(lhs.ty, rhs_ty)),
    }
    convert(sema, rhs, lhs.ty.id, is_null_ptr);
    Ok(lhs.ty)
}

pub fn default_argument_promotions(sema: &Sema, re: &mut ResolvedExpression) {
    if re.casted_ty().id == sema.builtins.float {
        num_conv(sema, re, sema.builtins.double);
    } else {
        promote(sema, re);
    }
}

pub fn pointer_integer_arithmetic(
    sema: &mut Sema,
    pointer: &mut ResolvedExpression,
    integral: &mut ResolvedExpression,
) -> Result<(QualifiedType, ValueCategory), Diagnostic> {
    let ResolvedType::Pointer(inner) = pointer.casted_ty().id.resolve_with(sema) else {
        return Err(Diagnostic::Poisoned);
    };
    match inner.id.resolve_with(sema) {
        t if !t.is_complete(sema) => Err(Diagnostic::InvalidOperand),
        ResolvedType::Function { .. } => Err(Diagnostic::InvalidOperand),
        _ => {
            promote(sema, integral);
            Ok((pointer.casted_ty(), RValue))
        }
    }
}

pub fn pointer_minus_pointer(
    sema: &Sema,
    lhs: &mut ResolvedExpression,
    rhs: &mut ResolvedExpression,
) -> Result<(QualifiedType, ValueCategory), Diagnostic> {
    let ResolvedType::Pointer(lp) = lhs.casted_ty().id.resolve_with(sema) else {
        return Err(Diagnostic::Poisoned);
    };
    let ResolvedType::Pointer(rp) = rhs.casted_ty().id.resolve_with(sema) else {
        return Err(Diagnostic::Poisoned);
    };
    let l = lp.id.resolve_with(sema);
    let r = rp.id.resolve_with(sema);
    if !l.is_object(sema) || !r.is_object(sema) {
        return Err(Diagnostic::InvalidOperand);
    }
    if !lp.is_compatible_ignoring_qualifiers(sema, rp) {
        return Err(Diagnostic::InvalidOperand);
    }
    Ok((QualifiedType::plain(sema.builtins.ptrdiff_t), RValue))
}

impl fmt::Display for CastKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CastKind::IntegerPromotion => write!(f, "promo"),
            CastKind::IntegerConversion => write!(f, "i->i"),
            CastKind::IntegerToFloating => write!(f, "i->f"),
            CastKind::FloatingConversion => write!(f, "fc"),
            CastKind::FloatingToInteger => write!(f, "f->i"),
            CastKind::FunctionToPointer => write!(f, "fn->p"),
            CastKind::LValueToRValue => write!(f, "l->r"),
            CastKind::ArrayToPointer => write!(f, "a->p"),
            CastKind::NullPointer => write!(f, "->null"),
            CastKind::ToVoid => write!(f, "->void"),
            CastKind::PointerToInteger => write!(f, "p->i"),
            CastKind::IntegerToPointer => write!(f, "i->p"),
            CastKind::PointerConversion => write!(f, "p->p"),
        }
    }
}
