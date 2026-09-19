use crate::ast::Tag;
use crate::codegen::LlvmType;
use crate::semantic::{ParamTypes, QualifiedType, ResolvedType, sema};

#[derive(Debug, Clone, Copy)]
pub enum ParamAttr {
    Direct,
    ByVal { ty: LlvmType, align: u32 },
    SRet { ty: LlvmType, align: u32 },
}

#[derive(Debug, Clone, Copy)]
pub enum ReturnAttr {
    Void,
    Direct(LlvmType),
    Sret { ty: LlvmType, align: u32 },
}

pub fn classify_param(qty: QualifiedType) -> (LlvmType, ParamAttr) {
    match qty.id.resolve() {
        ResolvedType::Tag(t) if t.resolve().kind != Tag::Enum => {
            (LlvmType::Ptr, ParamAttr::ByVal { ty: qty.llvm(), align: 4 })
        }
        _ => (qty.llvm(), ParamAttr::Direct),
    }
}

impl ReturnAttr {
    pub fn classify_return(qty: QualifiedType) -> ReturnAttr {
        let sema = sema();
        match qty.id.resolve() {
            ResolvedType::Void => ReturnAttr::Void,
            ResolvedType::Tag(t) if t.resolve().kind != Tag::Enum => {
                ReturnAttr::Sret { ty: qty.llvm(), align: sema.layout(&qty.id).align }
            }
            _ => ReturnAttr::Direct(qty.llvm()),
        }
    }

    pub fn ret_llvm(&self) -> LlvmType {
        match self {
            ReturnAttr::Sret { .. } | ReturnAttr::Void => LlvmType::Void,
            ReturnAttr::Direct(t) => *t,
        }
    }

    pub fn params<F, G, R>(&self, params: &ParamTypes, f: F, g: G) -> impl Iterator<Item = R>
    where
        F: FnOnce(&LlvmType, &u32) -> R,
        G: Fn(&QualifiedType) -> R,
    {
        let first = match self {
            ReturnAttr::Sret { ty, align } => Some(f(ty, align)),
            _ => None,
        };
        first.into_iter().chain(params.as_slice().iter().map(g))
    }
}
