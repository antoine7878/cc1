#[derive(Debug)]
pub struct Context {
    num_count: u32,
    op_count: u32,
}

impl Context {
    pub fn new() -> Self {
        Self {
            num_count: 0,
            op_count: 0,
        }
    }

    pub fn increment_op(&mut self) {
        self.op_count += 1;
    }

    pub fn increment_num(&mut self) {
        self.num_count += 1;
    }
}
