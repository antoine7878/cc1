use crate::models::{ProductionId, StateId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Shift(StateId),
    Reduce(ProductionId),
    Goto(StateId),
    Accept(ProductionId),
    Error,
}
