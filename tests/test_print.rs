mod common;

use cc1::ast::visit::{walk_declaration, walk_expression, walk_statement};
use cc1::ast::{DeclarationNode, Expression, ExpressionNode, StatementNode, Visitor};
use cc1::parser::Context;
use common::Unit;

fn dump(src: &str) -> String {
    let unit = Unit::parse(src);
    assert!(unit.parsed(), "cc1 failed to parse:\n{src}");
    unit.ast()
}

macro_rules! prints {
    ($name:ident, $src:expr, [$($line:expr),* $(,)?]) => {
        #[test]
        fn $name() {
            let expected = [$($line),*].join("\n");
            assert_eq!(dump($src), expected, "{}", $src);
        }
    };
}

prints!(
    print_object_declaration,
    "int x;",
    [
        "TranslationUnitNode <1:1, 1:6>",
        "`- DeclarationNode <1:1, 1:6>  int",
        "  `- InitDeclaratorNode <1:5>",
        "    `- DeclaratorNode <1:5> Ident x",
    ]
);

prints!(
    print_pointer_with_initializer,
    "char *p = \"a\";",
    [
        "TranslationUnitNode <1:1, 1:14>",
        "`- DeclarationNode <1:1, 1:14>  char",
        "  `- InitDeclaratorNode <1:6, 1:13>",
        "    |- DeclaratorNode <1:6, 1:7> Pointer",
        "    | `- DeclaratorNode <1:7> Ident p",
        "    `- InitializerNode <1:11, 1:13>",
        "      `- ExpressionNode <1:11, 1:13> StringLiteral a",
    ]
);

prints!(
    print_struct_definition,
    "struct S { int a; };",
    [
        "TranslationUnitNode <1:1, 1:20>",
        "`- DeclarationNode <1:1, 1:20>  struct S",
        "  `- Struct <1:1, 1:19> S",
        "    `- StructDeclaration <1:12, 1:17>  int",
        "      `- StructMemberDeclarator <1:16>",
        "        `- DeclaratorNode <1:16> Ident a",
    ]
);

prints!(
    print_enum_definition,
    "enum E { A = 1 };",
    [
        "TranslationUnitNode <1:1, 1:17>",
        "`- DeclarationNode <1:1, 1:17>  enum E",
        "  `- Enum <1:1, 1:16> E",
        "    `- Variant <1:10, 1:14> A",
        "      `- ExpressionNode <1:14> ConstantExpression",
        "        `- ExpressionNode <1:14> NumberLiteral",
    ]
);

prints!(
    print_function_definition,
    "int f(int a, int b) { return a + b; }",
    [
        "TranslationUnitNode <1:1, 1:37>",
        "`- FunctionDefinitionNode <1:1, 1:37> int",
        "  |- DeclaratorNode <1:5, 1:19> Function",
        "  | |- DeclaratorNode <1:5> Ident f",
        "  | `- FunctionParametersNode <1:7, 1:18>",
        "  |   |- ParameterDeclaration <1:7, 1:11>  int",
        "  |   | `- DeclaratorNode <1:11> Ident a",
        "  |   `- ParameterDeclaration <1:14, 1:18>  int",
        "  |     `- DeclaratorNode <1:18> Ident b",
        "  `- CompoundStatementNode <1:21, 1:37>",
        "    `- JumpStatementNode <1:23, 1:35> Return",
        "      `- ExpressionNode <1:30, 1:34> Add",
        "        |- ExpressionNode <1:30> Identifier a",
        "        `- ExpressionNode <1:34> Identifier b",
    ]
);

prints!(
    print_iteration_statement,
    "void f(void) { while (1) { x++; } }",
    [
        "TranslationUnitNode <1:1, 1:35>",
        "`- FunctionDefinitionNode <1:1, 1:35> void",
        "  |- DeclaratorNode <1:6, 1:12> Function",
        "  | |- DeclaratorNode <1:6> Ident f",
        "  | `- FunctionParametersNode <1:8, 1:11>",
        "  |   `- ParameterDeclaration <1:8, 1:11>  void",
        "  |     `- DeclaratorNode <1:8, 1:11> Abstract",
        "  `- CompoundStatementNode <1:14, 1:35>",
        "    `- IterationStatementNode <1:16, 1:33> While",
        "      |- ExpressionNode <1:23> NumberLiteral",
        "      `- CompoundStatementNode <1:26, 1:33>",
        "        `- ExpressionStatementNode <1:28, 1:31>",
        "          `- ExpressionNode <1:28, 1:30> post ++",
        "            `- ExpressionNode <1:28> Identifier x",
    ]
);

prints!(
    print_two_declarations,
    "int x;\nlong y;",
    [
        "TranslationUnitNode <1:1, 2:7>",
        "|- DeclarationNode <1:1, 1:6>  int",
        "| `- InitDeclaratorNode <1:5>",
        "|   `- DeclaratorNode <1:5> Ident x",
        "`- DeclarationNode <2:1, 2:7>  long",
        "  `- InitDeclaratorNode <2:6>",
        "    `- DeclaratorNode <2:6> Ident y",
    ]
);

#[derive(Default)]
struct Walk {
    declarations: usize,
    statements: usize,
    expressions: usize,
    names: Vec<String>,
}

impl Visitor for Walk {
    fn visit_declaration(&mut self, ctx: &Context, node: &DeclarationNode) {
        self.declarations += 1;
        walk_declaration(self, ctx, node);
    }

    fn visit_statement(&mut self, ctx: &Context, node: &StatementNode) {
        self.statements += 1;
        walk_statement(self, ctx, node);
    }

    fn visit_expression(&mut self, ctx: &Context, node: &ExpressionNode) {
        self.expressions += 1;
        if let Expression::Identifier(name) = node.id.resolve(ctx) {
            self.names.push(name.id.resolve(ctx).clone());
        }
        walk_expression(self, ctx, node);
    }
}

fn walked(src: &str) -> Walk {
    let unit = Unit::parse(src);
    assert!(unit.parsed(), "cc1 failed to parse:\n{src}");
    let mut walk = Walk::default();
    walk.visit_translation_unit(&unit.ctx, &unit.ctx.ast);
    walk
}

#[test]
fn a_visitor_sees_identifiers_in_source_order() {
    let walk = walked("int a, b, c; void f(void) { a = b + c; }");
    assert_eq!(walk.names, vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    assert_eq!(walk.expressions, 5);
    assert_eq!(walk.declarations, 1);
    assert_eq!(walk.statements, 1);
}

#[test]
fn a_visitor_sees_nested_operands_before_their_neighbours() {
    let walk = walked("void f(void) { a = b * c + d; }");
    assert_eq!(
        walk.names,
        vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]
    );
    assert_eq!(walk.expressions, 7);
}

#[test]
fn a_visitor_counts_every_statement_of_a_body() {
    let walk = walked("void f(void) { int i; i = 0; if (i) i = 1; else i = 2; }");
    assert_eq!(walk.declarations, 1);
    assert_eq!(walk.statements, 4);
    assert_eq!(walk.names, vec!["i".to_string(); 4]);
}
