use crate::arena::{Loan, OptionPoisoned};
use crate::ast::{StatementNode, statement::StatementId};
use crate::semantic::{Diagnosis, ResolvedStatement, Sema};

type Operands<'s, const N: usize> = Loan<'s, Sema, StatementId, ResolvedStatement, N>;

fn operands<'s, const N: usize>(sema: &'s mut Sema, nodes: [&StatementNode; N]) -> Result<Operands<'s, N>, Diagnosis> {
    Loan::take(sema, nodes.map(|n| n.id)).ok_poisoned()
}

pub fn st() {}
