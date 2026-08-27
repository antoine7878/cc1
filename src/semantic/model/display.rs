use crate::parser::Context;
use crate::semantic::{ParamTypes, QualifiedType, ResolvedType};

impl Context {
    pub fn describe(&self, qt: &QualifiedType) -> String {
        let mut out = String::new();
        if qt.is_const {
            out.push_str("const ");
        }
        if qt.is_volatile {
            out.push_str("volatile ");
        }
        match qt.ty.resolve(self) {
            ResolvedType::Void => out.push_str("void"),
            ResolvedType::Char => out.push_str("char"),
            ResolvedType::SignedChar => out.push_str("signed char"),
            ResolvedType::UnsignedChar => out.push_str("unsigned char"),
            ResolvedType::Short => out.push_str("short"),
            ResolvedType::UnsignedShort => out.push_str("unsigned short"),
            ResolvedType::Int => out.push_str("int"),
            ResolvedType::UnsignedInt => out.push_str("unsigned int"),
            ResolvedType::Long => out.push_str("long"),
            ResolvedType::UnsignedLong => out.push_str("unsigned long"),
            ResolvedType::Float => out.push_str("float"),
            ResolvedType::Double => out.push_str("double"),
            ResolvedType::LongDouble => out.push_str("long double"),
            ResolvedType::Pointer(inner) => {
                out.push('*');
                match inner.ty.resolve(self) {
                    ResolvedType::Function { .. } | ResolvedType::Array { .. } => {
                        out.push_str(&format!("({})", self.describe(inner)))
                    }
                    _ => out.push_str(&self.describe(inner)),
                }
            }
            ResolvedType::Tag(id) => {
                let def = id.resolve(self);
                let name = def.name.map(|n| n.id.resolve(self).as_str()).unwrap_or("<anonymous>");
                out.push_str(&format!("{} {}", def.kind(), name));
                if !def.is_complete {
                    out.push_str(" (incomplete)");
                }
            }
            ResolvedType::Array { elem, len } => {
                let (mut elem, mut len) = (elem, len);
                let mut dimensions = String::new();
                loop {
                    dimensions.push('[');
                    if let Some(len) = len {
                        dimensions.push_str(&len.to_string());
                    }
                    dimensions.push(']');
                    let ResolvedType::Array { elem: inner, len: size } = elem.ty.resolve(self) else { break };
                    (elem, len) = (inner, size);
                }
                out.push_str(&self.describe(elem));
                out.push_str(&dimensions);
            }
            ResolvedType::Function { ret, params } => {
                out.push_str(&self.describe(ret));
                out.push('(');
                if let ParamTypes::Prototype { params, is_variadic } = params {
                    let described: Vec<String> = params.iter().map(|param| self.describe(param)).collect();
                    match described.is_empty() {
                        true => out.push_str("void"),
                        false => out.push_str(&described.join(", ")),
                    }
                    if *is_variadic {
                        out.push_str(", ...")
                    }
                }
                out.push(')');
            }
        }
        out
    }
}
