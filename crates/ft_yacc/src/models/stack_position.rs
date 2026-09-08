#[derive(Debug, Clone)]
pub struct StackPosition {
    pub action_position: usize,
    pub stack_position: Option<isize>,
    pub utype: Option<String>,
    pub line_no: usize,
}

impl StackPosition {
    pub fn new(action_position: usize, stack_position: Option<isize>, utype: Option<String>, line_no: usize) -> Self {
        Self {
            action_position,
            stack_position,
            utype,
            line_no,
        }
    }
}
