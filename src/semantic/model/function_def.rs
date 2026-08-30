use std::iter::zip;

use crate::ast::{Name, Storage};
use crate::define_arena;
use crate::parser::Span;
use crate::semantic::model::cast::default_argument_promotions;
use crate::semantic::{ExpressionKind, QualifiedType, ResolvedExpression, Sema, SymbolArena, SymbolId};

define_arena!(
    FunctionDef,
    FunctionDefArena,
    FunctionDefId,
    crate::semantic::Sema,
    functions
);

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
    // 6.5.4.3 For two function types to be compatible, both shall specify compatible return types.
    pub fn is_compatible(&self, sema: &Sema, other: &Self) -> bool {
        match (self, other) {
            // 6.5.4.3 Moreover, the parameter type lists, if both are present, shall agree in the number
            // of parameters and in use of the ellipsis terminator; corresponding parameters shall have
            // compatible types.
            (
                ParamTypes::Prototype { params: p1, is_variadic: v1 },
                ParamTypes::Prototype { params: p2, is_variadic: v2 },
            ) => v1 == v2 && p1.len() == p2.len() && zip(p1, p2).all(|(a, b)| a.is_compatible(sema, b)),
            // 6.5.4.3 If one type has a parameter type list and the other type is specified by a function
            // declarator that is not part of a function definition and that contains an empty identifier
            // list, the parameter list shall not have an ellipsis terminator and the type of each parameter
            // shall be compatible with the type that results from the application of the default argument
            // promotions.
            (
                ParamTypes::Prototype { params, is_variadic },
                ParamTypes::Unspecified,
            )
            | (
                ParamTypes::Unspecified,
                ParamTypes::Prototype { params, is_variadic },
            ) => !is_variadic && params.iter().all(|p| p.is_compatible(sema, &promoted(sema, *p))),
            // 6.5.4.3 the parameter type lists, if both are present: neither list is present here, only the
            // return types have to be compatible.
            (ParamTypes::Unspecified, ParamTypes::Unspecified) => true,
        }
    }

    // 6.5.4.3 If one type has a parameter type list and the other type is specified by a function
    // definition that contains a (possibly empty) identifier list, both shall agree in the number of
    // parameters, and the type of each prototype parameter shall be compatible with the type that results
    // from the application of the default argument promotions to the type of the corresponding identifier.
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

// 6.5.4.3 the type that results from the application of the default argument promotions
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
