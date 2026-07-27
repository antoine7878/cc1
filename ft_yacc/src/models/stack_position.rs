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

    #[allow(unused)]
    pub fn len(&self) -> usize {
        let mut ret = 2;
        if let Some(p) = &self.stack_position {
            ret += p.to_string().len() - 1;
        }
        if let Some(t) = &self.utype {
            ret += 2 + t.len();
        }
        ret
    }
}
