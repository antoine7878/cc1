use std::iter::zip;

use crate::ast::{Name, Storage};
use crate::define_arena;
use crate::parser::Span;
use crate::semantic::model::cast::default_argument_promotions;
use crate::semantic::{ExpressionKind, QualifiedType, ResolvedExpression, Sema, SymbolArena, SymbolId};

define_arena!(FunctionDef, FunctionDefArena, FunctionDefId, Sema, sema, functions);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionDef {
    pub sym: SymbolId,
    pub parameters: Vec<SymbolId>,
    pub is_complete: bool,
}

impl FunctionDefArena {
    pub fn declare(&mut self, sym: SymbolId) -> FunctionDefId {
        self.alloc(FunctionDef {
            sym,
            parameters: Vec::new(),
            is_complete: false,
        })
    }

    pub fn ty(&self, id: FunctionDefId, symbols: &SymbolArena) -> Option<QualifiedType> {
        symbols.get(self.get(id).sym).ty
    }

    pub fn complete(&mut self, id: FunctionDefId, parameters: Vec<SymbolId>) {
        let def = self.get_mut(id);
        def.parameters = parameters;
        def.is_complete = true;
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
