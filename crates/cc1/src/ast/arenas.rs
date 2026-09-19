use crate::arena::{Has, HasMut, Installed, Owned};
use crate::ast::statement::StatementId;
use crate::ast::{
    Declarator, DeclaratorArena, DeclaratorId, Enum, EnumArena, EnumId, Enumerator, EnumeratorArena, EnumeratorId,
    Expression, ExpressionArena, ExpressionId, NameId, NameInterner, Statement, StatementArena, StringConstId,
    StringConstInterner, StringConstant, Struct, StructArena, StructId, Union, UnionArena, UnionId,
};
use crate::context::ctx;

#[derive(Debug, Default)]
pub struct AstArenas {
    pub names: NameInterner,
    pub strings: StringConstInterner,
    pub structs: StructArena,
    pub enums: EnumArena,
    pub unions: UnionArena,
    pub enumerators: EnumeratorArena,
    pub expressions: ExpressionArena,
    pub declarators: DeclaratorArena,
    pub statements: StatementArena,
}

impl Installed for AstArenas {
    fn installed() -> &'static Self {
        &ctx().arenas
    }
}

impl Has<String> for AstArenas {
    fn get(&self, id: NameId) -> &String {
        self.names.get(id)
    }
}

impl Owned for String {
    type Holder = AstArenas;
}

impl Has<StringConstant> for AstArenas {
    fn get(&self, id: StringConstId) -> &StringConstant {
        self.strings.get(id)
    }
}

impl Owned for StringConstant {
    type Holder = AstArenas;
}

impl Has<Struct> for AstArenas {
    fn get(&self, id: StructId) -> &Struct {
        self.structs.get(id)
    }
}

impl HasMut<Struct> for AstArenas {
    fn get_mut(&mut self, id: StructId) -> &mut Struct {
        self.structs.get_mut(id)
    }
}

impl Owned for Struct {
    type Holder = AstArenas;
}

impl Has<Enum> for AstArenas {
    fn get(&self, id: EnumId) -> &Enum {
        self.enums.get(id)
    }
}

impl HasMut<Enum> for AstArenas {
    fn get_mut(&mut self, id: EnumId) -> &mut Enum {
        self.enums.get_mut(id)
    }
}

impl Owned for Enum {
    type Holder = AstArenas;
}

impl Has<Union> for AstArenas {
    fn get(&self, id: UnionId) -> &Union {
        self.unions.get(id)
    }
}

impl HasMut<Union> for AstArenas {
    fn get_mut(&mut self, id: UnionId) -> &mut Union {
        self.unions.get_mut(id)
    }
}

impl Owned for Union {
    type Holder = AstArenas;
}

impl Has<Enumerator> for AstArenas {
    fn get(&self, id: EnumeratorId) -> &Enumerator {
        self.enumerators.get(id)
    }
}

impl HasMut<Enumerator> for AstArenas {
    fn get_mut(&mut self, id: EnumeratorId) -> &mut Enumerator {
        self.enumerators.get_mut(id)
    }
}

impl Owned for Enumerator {
    type Holder = AstArenas;
}

impl Has<Expression> for AstArenas {
    fn get(&self, id: ExpressionId) -> &Expression {
        self.expressions.get(id)
    }
}

impl HasMut<Expression> for AstArenas {
    fn get_mut(&mut self, id: ExpressionId) -> &mut Expression {
        self.expressions.get_mut(id)
    }
}

impl Owned for Expression {
    type Holder = AstArenas;
}

impl Has<Declarator> for AstArenas {
    fn get(&self, id: DeclaratorId) -> &Declarator {
        self.declarators.get(id)
    }
}

impl HasMut<Declarator> for AstArenas {
    fn get_mut(&mut self, id: DeclaratorId) -> &mut Declarator {
        self.declarators.get_mut(id)
    }
}

impl Owned for Declarator {
    type Holder = AstArenas;
}

impl Has<Statement> for AstArenas {
    fn get(&self, id: StatementId) -> &Statement {
        self.statements.get(id)
    }
}

impl HasMut<Statement> for AstArenas {
    fn get_mut(&mut self, id: StatementId) -> &mut Statement {
        self.statements.get_mut(id)
    }
}

impl Owned for Statement {
    type Holder = AstArenas;
}
