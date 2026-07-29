#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

#[derive(Debug)]
pub enum Expr {
    Char(i8),
    UChar(u8),
    UInt(u32),
    ULong(u64),
    Int(i32),
    Long(i64),
    Var(String),
    Binary {
        op: BinOp,
        lhs: NodeId,
        rhs: NodeId,
    },
    Unary {
        op: UnOp,
        expr: NodeId,
    },
    Call {
        callee: NodeId,
        args: Vec<NodeId>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp { Add, Sub, Mul, Div }

#[derive(Debug, Clone, Copy)]
pub enum UnOp { Neg }

pub struct Arena {
    nodes: Vec<Expr>,
}

impl Arena {
    pub fn new() -> Self {
        Arena { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, expr: Expr) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(expr);
        id
    }

    pub fn get(&self, id: NodeId) -> &Expr {
        &self.nodes[id.0]
    }

    pub fn get_mut(&mut self, id: NodeId) -> &mut Expr {
        &mut self.nodes[id.0]
    }
}
