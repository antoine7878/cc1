use crate::models::{ActionId, StackPosition, TokenId};

#[derive(Debug, Clone)]
pub struct Production {
    pub product: TokenId,
    pub recipe: Vec<TokenId>,
    pub action: Option<ActionId>,
    pub precedence: Option<usize>,
    pub line_no: usize,
    pub stack_positons: Vec<StackPosition>,
    pub mid_context: Option<Vec<TokenId>>,
}

impl Production {
    pub fn new(
        product: TokenId,
        recipe: Vec<TokenId>,
        action: Option<ActionId>,
        precedence: Option<usize>,
        line_no: usize,
        stack_positons: Vec<StackPosition>,
        mid_context: Option<Vec<TokenId>>,
    ) -> Self {
        Self {
            product,
            recipe,
            action,
            precedence,
            line_no,
            stack_positons,
            mid_context,
        }
    }
}
