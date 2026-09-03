use cc1::semantic::CastKind;

#[derive(Debug, PartialEq, Clone)]
pub enum Ty {
    Void,
    Char,
    SChar,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    Float,
    Double,
    LDouble,
    Ptr(Box<Ty>),
    Array(Box<Ty>, Option<usize>),
    Func {
        ret: Box<Ty>,
        params: Option<Vec<Ty>>,
        variadic: bool,
    },
    Struct {
        tag: Option<String>,
        complete: bool,
    },
    Union {
        tag: Option<String>,
        complete: bool,
    },
    Enum {
        tag: Option<String>,
        complete: bool,
    },
    Const(Box<Ty>),
    Volatile(Box<Ty>),
}

impl Ty {
    pub fn ptr(inner: Ty) -> Ty {
        Ty::Ptr(Box::new(inner))
    }

    pub fn arr(elem: Ty, len: usize) -> Ty {
        Ty::Array(Box::new(elem), Some(len))
    }

    pub fn flex(elem: Ty) -> Ty {
        Ty::Array(Box::new(elem), None)
    }

    pub fn func(ret: Ty, params: impl IntoIterator<Item = Ty>) -> Ty {
        Ty::Func {
            ret: Box::new(ret),
            params: Some(params.into_iter().collect()),
            variadic: false,
        }
    }

    pub fn func0(ret: Ty) -> Ty {
        Ty::Func {
            ret: Box::new(ret),
            params: Some(Vec::new()),
            variadic: false,
        }
    }

    pub fn func_variadic(ret: Ty, params: impl IntoIterator<Item = Ty>) -> Ty {
        Ty::Func {
            ret: Box::new(ret),
            params: Some(params.into_iter().collect()),
            variadic: true,
        }
    }

    pub fn noproto(ret: Ty) -> Ty {
        Ty::Func {
            ret: Box::new(ret),
            params: None,
            variadic: false,
        }
    }

    pub fn strukt(tag: &str) -> Ty {
        Ty::Struct {
            tag: Some(tag.to_string()),
            complete: true,
        }
    }

    pub fn strukt_incomplete(tag: &str) -> Ty {
        Ty::Struct {
            tag: Some(tag.to_string()),
            complete: false,
        }
    }

    pub fn anon_struct() -> Ty {
        Ty::Struct {
            tag: None,
            complete: true,
        }
    }

    pub fn union(tag: &str) -> Ty {
        Ty::Union {
            tag: Some(tag.to_string()),
            complete: true,
        }
    }

    pub fn enom(tag: &str) -> Ty {
        Ty::Enum {
            tag: Some(tag.to_string()),
            complete: true,
        }
    }

    pub fn konst(inner: Ty) -> Ty {
        Ty::Const(Box::new(inner))
    }

    pub fn vol(inner: Ty) -> Ty {
        Ty::Volatile(Box::new(inner))
    }
}

#[derive(Debug, PartialEq)]
pub struct Shape {
    pub ty: Option<Ty>,
    pub lvalue: bool,
    pub casts: Vec<(CastKind, Ty)>,
    pub result_cast: Option<(CastKind, Ty)>,
}

impl Shape {
    pub fn rvalue(ty: Ty) -> Self {
        Self {
            ty: Some(ty),
            lvalue: false,
            casts: Vec::new(),
            result_cast: None,
        }
    }

    pub fn lvalue(ty: Ty) -> Self {
        Self {
            ty: Some(ty),
            lvalue: true,
            casts: Vec::new(),
            result_cast: None,
        }
    }

    pub fn unresolved() -> Self {
        Self {
            ty: None,
            lvalue: false,
            casts: Vec::new(),
            result_cast: None,
        }
    }

    pub fn then(mut self, kind: CastKind, to: Ty) -> Self {
        self.casts.push((kind, to));
        self
    }

    /// The conversion applied to the value of `E1 op E2` before it is stored back into
    /// `E1` in a compound assignment (6.3.16.2).
    pub fn result(mut self, kind: CastKind, to: Ty) -> Self {
        self.result_cast = Some((kind, to));
        self
    }
}

pub fn rv(ty: Ty) -> Shape {
    Shape::rvalue(ty)
}

pub fn lv(ty: Ty) -> Shape {
    Shape::lvalue(ty)
}

pub fn none() -> Shape {
    Shape::unresolved()
}

pub fn ints(n: usize) -> Vec<Shape> {
    (0..n).map(|_| rv(Ty::Int)).collect()
}
