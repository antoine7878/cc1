use core::fmt;

use crate::codegen::{LlvmName, LlvmSymbol, LlvmType, ParamAttr};

#[derive(Debug, Clone)]
pub struct LlvmParam {
    pub sym: LlvmSymbol,
    pub attr: ParamAttr,
}

impl LlvmParam {
    pub fn new(sym: LlvmSymbol, attr: ParamAttr) -> Self {
        Self { sym, attr }
    }

    pub fn unnamed(ty: LlvmType, attr: ParamAttr) -> Self {
        let sym = LlvmSymbol::new(ty, LlvmName::None);
        Self { sym, attr }
    }
}

impl fmt::Display for LlvmParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.attr {
            ParamAttr::Direct => write!(f, "{}", self.sym.ty)?,
            ParamAttr::ByVal { ty, align } => write!(f, "ptr byval({ty}) align {align}")?,
            ParamAttr::SRet { ty, align } => write!(f, "ptr sret({ty}) align {align}")?,
        }
        match self.sym.name {
            LlvmName::None => Ok(()),
            name => write!(f, " {name}"),
        }
    }
}
