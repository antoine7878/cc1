use std::iter::zip;

use crate::ast::{Name, Storage};
use crate::define_arena;
use crate::semantic::model::cast::default_argument_promotions;
use crate::semantic::{ExpressionKind, QualifiedType, ResolvedExpression, Sema, SymbolId};
use libft::Span;

define_arena!(
    FunctionDef,
    FunctionDefArena,
    FunctionDefId,
    Sema,
    crate::semantic::sema(),
    functions
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionDef {
    pub sym: SymbolId,
    pub return_ty: QualifiedType,
    pub parameters: Vec<SymbolId>,
    pub labels: Vec<Name>,
}

impl FunctionDefArena {
    pub fn declare(&mut self, sym: SymbolId, return_ty: QualifiedType) -> FunctionDefId {
        self.alloc(FunctionDef {
            sym,
            return_ty,
            parameters: Vec::new(),
            labels: Vec::new(),
        })
    }

    pub fn complete(&mut self, id: FunctionDefId, parameters: Vec<SymbolId>) {
        self.get_mut(id).parameters = parameters;
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum ParamTypes {
    Unspecified,
    Prototype {
        params: Vec<QualifiedType>,
        is_variadic: bool,
    },
}

#[rustfmt::skip]
impl ParamTypes {
    pub fn is_compatible(&self, sema: &Sema, other: &Self) -> bool {
        match (self, other) {
            (
                ParamTypes::Prototype { params: p1, is_variadic: v1 },
                ParamTypes::Prototype { params: p2, is_variadic: v2 },
            ) => v1 == v2 && p1.len() == p2.len() && zip(p1, p2).all(|(a, b)| a.is_compatible(sema, b)),
            (
                ParamTypes::Prototype { params, is_variadic },
                ParamTypes::Unspecified,
            )
            | (
                ParamTypes::Unspecified,
                ParamTypes::Prototype { params, is_variadic },
            ) => !is_variadic && params.iter().all(|p| p.is_compatible(sema, &promoted(sema, *p))),
            (ParamTypes::Unspecified, ParamTypes::Unspecified) => true,
        }
    }

    pub fn is_compatible_with_identifiers(&self, sema: &Sema, identifiers: &[QualifiedType]) -> bool {
        match self {
            ParamTypes::Unspecified => true,
            ParamTypes::Prototype { params, .. } => {
                params.len() == identifiers.len()
                    && zip(params, identifiers).all(|(p, i)| p.is_compatible(sema, &promoted(sema, *i)))
            }
        }
    }
}

fn promoted(sema: &Sema, ty: QualifiedType) -> QualifiedType {
    let mut re = ResolvedExpression::new(ty, ExpressionKind::RValue);
    default_argument_promotions(sema, &mut re);
    re.casted_ty()
}

#[derive(Clone, Debug)]
pub struct ParamInfo {
    pub name: Option<Name>,
    pub ty: QualifiedType,
    pub storage: Option<Storage>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum DeclaredParams {
    Unspecified,
    Names(Vec<Name>),
    Prototype { params: Vec<ParamInfo>, is_variadic: bool },
}

impl DeclaredParams {
    pub fn types(&self) -> ParamTypes {
        match self {
            DeclaredParams::Unspecified | DeclaredParams::Names(_) => ParamTypes::Unspecified,
            DeclaredParams::Prototype { params, is_variadic } => ParamTypes::Prototype {
                params: params.iter().map(|param| param.ty).collect(),
                is_variadic: *is_variadic,
            },
        }
    }
}
