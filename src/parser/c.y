%no_main
%feedback
%{
use crate::ast::{Qualifier, Type, ExpressionNode, Name, DeclarationSpecifier, Initializer, TypeSpecifier, ParameterDeclaration};
use crate::ast::{DeclarationNode, InitDeclaratorNode, DeclaratorNode, InitializerNode, Storage, FunctionParametersNode, Tag};
use crate::ast::{StructDeclaration, StructMemberDeclarator, VariantId, EnumId, LabeledStatementNode, StatementNode, Labeled, CompoundStatementNode};
use crate::ast::{ExpressionStatementNode, SelectionStatementNode, IterationStatementNode, JumpStatementNode, JumpStatement};
use crate::ast::{ExternalDeclarationNode, FunctionDefinitionNode, TranslationUnitNode};

use crate::parser::{YYLex, Context, Span};
use crate::utils::yyerror;

macro_rules! node{
    ($self:expr, $factory:ident, $method:ident $(, $arg:expr)*) => {{
        $self.lexer.ctx.arenas.$factory.$method($($arg,)*)
    }};
}

macro_rules! node_span {
    ($self:expr, $factory:ident, $method:ident $(, $arg:expr)* $(,)?) => {{
        let span = $self.span;
        $self.lexer.ctx.arenas.$factory.$method($($arg,)* span)
    }};
}

macro_rules! with_span {
    ($self:expr, $func:path $(, $arg:expr)* $(,)?) => {{
        let span = $self.span;
        $func($($arg,)* span)
    }};
}

macro_rules! push {
    ($vec:expr, $elem:expr) => {{
        $vec.push($elem);
        $vec
    }};
}

macro_rules! spec {
    ($self:expr, $specs:expr) => {{
        $self.lexer.ctx.note_specifiers($specs)
    }};
}

%}

%token<Name> IDENTIFIER CONSTANT TYPE_NAME
%token<String> STRING_LITERAL
%token TYPEDEF EXTERN STATIC AUTO REGISTER
%token CHAR SHORT INT LONG SIGNED UNSIGNED FLOAT DOUBLE CONST VOLATILE VOID
%token STRUCT UNION ENUM ELLIPSIS
%token CASE DEFAULT IF SWITCH WHILE DO FOR GOTO CONTINUE BREAK RETURN

%left ','
%left PREC_NO_COMMA
%right '=' SUB_ASSIGN LEFT_ASSIGN RIGHT_ASSIGN AND_ASSIGN MUL_ASSIGN
      DIV_ASSIGN MOD_ASSIGN ADD_ASSIGN XOR_ASSIGN OR_ASSIGN
%right '?' ':'
%left OR_OP
%left AND_OP
%left '|'
%left '^'
%left '&'
%left EQ_OP NE_OP
%left '<' '>' LE_OP GE_OP
%left LEFT_OP RIGHT_OP
%left '+' '-'
%left '*' '/' '%'
%right '!' '~' INC_OP DEC_OP POST_INC_OP POST_DEC_OP SIZEOF PREC_UNARY
%nonassoc '(' '[' '.' PTR_OP
%nonassoc PREC_THEN
%nonassoc ELSE

%type<String> string_literal
%type<Vec<Name>> identifier_list
%type<Name> merged_literal

%type<ExpressionNode> expression constant_expression

%type<Type> type_name
%type<DeclarationNode> declaration
%type<Vec<DeclarationNode>> declaration_list
%type<TypeSpecifier> type_specifier type_specifier_kw struct_or_union_specifier
%type<Storage> storage_class_specifier
%type<Qualifier> type_qualifier
%type<Vec<Qualifier>> type_qualifier_list
%type<Vec<DeclarationSpecifier>> declaration_specifiers declaration_specifiers_untyped declaration_specifiers_typed
%type<InitDeclaratorNode> init_declarator
%type<Vec<InitDeclaratorNode>> init_declarator_list
%type<InitializerNode> initializer
%type<Vec<InitializerNode>> initializer_list
%type<DeclaratorNode> declarator direct_declarator direct_abstract_declarator abstract_declarator
%type<FunctionParametersNode> parameter_type_list
%type<Vec<ParameterDeclaration>> parameter_list
%type<ParameterDeclaration> parameter_declaration
%type<Vec<DeclarationSpecifier>> specifier_qualifier_list specifier_qualifier_list_untyped specifier_qualifier_list_typed
%type<Vec<Vec<Qualifier>>> pointer

%type<Tag> struct_or_union
%type<Vec<StructDeclaration>> struct_declaration_list
%type<StructDeclaration> struct_declaration
%type<Vec<StructMemberDeclarator>> struct_declarator_list
%type<StructMemberDeclarator> struct_declarator
%type<EnumId> enum_specifier
%type<Vec<VariantId>> enumerator_list
%type<VariantId> enumerator

%type<StatementNode> statement
%type<Vec<StatementNode>> statement_list
%type<LabeledStatementNode> labeled_statement
%type<CompoundStatementNode> compound_statement
%type<ExpressionStatementNode> expression_statement
%type<SelectionStatementNode> selection_statement
%type<IterationStatementNode> iteration_statement
%type<JumpStatementNode> jump_statement

%type<FunctionDefinitionNode> function_definition
%type<ExternalDeclarationNode> external_declaration
%type<Vec<ExternalDeclarationNode>> external_declaration_list
%type<()> translation_unit enter_scope exit_scope enter_struct exit_struct reopen_params

%start translation_unit

%%

enter_scope:                                                                                { self.lexer.ctx.push_scope(); } ;
exit_scope:                                                                                 { self.lexer.ctx.pop_scope(); } ;
reopen_params:                                                                              { self.lexer.ctx.unstash_scope(); } ;
enter_struct:                                                                               { self.lexer.ctx.enter_struct(); } ;
exit_struct:                                                                                { self.lexer.ctx.exit_struct(); } ;

translation_unit /* (TranslationUnitNode) */
	: external_declaration_list                                                             { let ast = with_span!(self, TranslationUnitNode::new, $1); self.lexer.ctx.ast = ast; }
	;

external_declaration_list /* Vec<ExternalDeclarationNode> */
	: external_declaration                                                                  { vec![$1] }
	| external_declaration_list external_declaration                                        { push!($<mut>1, $2) }
	;

external_declaration /* ExternalDeclarationNode */
	: function_definition                                                                   { with_span!(self, ExternalDeclarationNode::function, $1) }
	| declaration                                                                           { with_span!(self, ExternalDeclarationNode::declaration, $1) }
	;

function_definition /* FunctionDefinitionNode */
	: declaration_specifiers declarator reopen_params declaration_list compound_statement   { self.lexer.ctx.pop_scope(); with_span!(self, FunctionDefinitionNode::new, $1, $2, $4, $5) }
	| declaration_specifiers declarator reopen_params compound_statement                    { self.lexer.ctx.pop_scope(); with_span!(self, FunctionDefinitionNode::new, $1, $2, vec![], $4) }
	| declarator reopen_params declaration_list compound_statement                          { self.lexer.ctx.pop_scope(); with_span!(self, FunctionDefinitionNode::new, vec![], $1, $3, $4) }
	| declarator reopen_params compound_statement                                           { self.lexer.ctx.pop_scope(); with_span!(self, FunctionDefinitionNode::new, vec![], $1, vec![], $3) }
	;

constant_expression /* ExpressionNode */
    : expression %prec PREC_NO_COMMA                                                        { node_span!(self, expressions, constant_expression, $1) }
    ;

string_literal /* String */
      : STRING_LITERAL                                                                      { $1 }
      | string_literal STRING_LITERAL                                                       { $<mut>1.push_str(&$2); $1 }
      ;

merged_literal /* Name */
      : string_literal                                                                      { self.lexer.ctx.arenas.names.add($1, self.lexer.span.clone()) }
      ;

expression /* ExpressionNode */
    : '(' expression ')'                                                                    { $2 }
    | IDENTIFIER                                                                            { node_span!(self, expressions, identifier, $1) }
    | CONSTANT                                                                              { node_span!(self, expressions, constant, $1)}
    | merged_literal                                                                        { node_span!(self, expressions, string_literal, $1) }
    | expression '[' expression ']'                                                         { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '(' ')'                                                                    { node_span!(self, expressions, function_call, $1, None) }
    | expression '(' expression ')'                                                         { node_span!(self, expressions, function_call, $1, Some($3)) }
    | expression '.' IDENTIFIER                                                             { node_span!(self, expressions, access, $1, $2, $3) }
    | expression PTR_OP IDENTIFIER                                                          { node_span!(self, expressions, access, $1, $2, $3) }
    | SIZEOF '(' type_name ')'                                                              { node_span!(self, expressions, sizeof_type, $3) }
    | SIZEOF expression %prec PREC_UNARY                                                    { node_span!(self, expressions, sizeof_expr, $2) }
    | '(' type_name ')' expression %prec PREC_UNARY                                         { node_span!(self, expressions, cast, $2, $4) }
    | expression INC_OP                                                                     { node_span!(self, expressions, unary, YYToken::POST_INC_OP, $1) }
    | expression DEC_OP                                                                     { node_span!(self, expressions, unary, YYToken::POST_DEC_OP, $1) }
    | INC_OP expression                                                                     { node_span!(self, expressions, unary, $1, $2) }
    | DEC_OP expression                                                                     { node_span!(self, expressions, unary, $1, $2) }
    | '&' expression %prec PREC_UNARY                                                       { node_span!(self, expressions, unary, $1, $2) }
    | '*' expression %prec PREC_UNARY                                                       { node_span!(self, expressions, unary, $1, $2) }
    | '+' expression %prec PREC_UNARY                                                       { node_span!(self, expressions, unary, $1, $2) }
    | '-' expression %prec PREC_UNARY                                                       { node_span!(self, expressions, unary, $1, $2) }
    | '~' expression                                                                        { node_span!(self, expressions, unary, $1, $2) }
    | '!' expression                                                                        { node_span!(self, expressions, unary, $1, $2) }
    | expression '+' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '-' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '*' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '/' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '%' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression LEFT_OP expression                                                         { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression RIGHT_OP expression                                                        { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '<' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '>' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression LE_OP expression                                                           { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression GE_OP expression                                                           { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression EQ_OP expression                                                           { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression NE_OP expression                                                           { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression '&' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression '^' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression '|' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression AND_OP expression                                                          { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression OR_OP expression                                                           { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression '=' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression MUL_ASSIGN expression                                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression DIV_ASSIGN expression                                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression MOD_ASSIGN expression                                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression ADD_ASSIGN expression                                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression SUB_ASSIGN expression                                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression LEFT_ASSIGN expression                                                     { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression RIGHT_ASSIGN expression                                                    { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression AND_ASSIGN expression                                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression XOR_ASSIGN expression                                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression OR_ASSIGN expression                                                       { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression ',' expression                                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '?' expression ':' expression                                              { node_span!(self, expressions, ternary, $1, $3, $5) }
    ;

declaration /* DeclarationNode */
	: declaration_specifiers ';'                                                            { with_span!(self, DeclarationNode::new, $1, vec![]) }
	| declaration_specifiers init_declarator_list ';'                                       { with_span!(self, DeclarationNode::new, $1, $2) }
	;

declaration_specifiers /* Vec<DeclarationSpecifier> */
	: declaration_specifiers_untyped                                                        { $1 }
	| declaration_specifiers_typed                                                          { $1 }
	;

declaration_specifiers_untyped /* Vec<DeclarationSpecifier> — no type yet */
	: storage_class_specifier                                                               { spec!(self, vec![DeclarationSpecifier::Storage($1)]) }
	| type_qualifier                                                                        { spec!(self, vec![DeclarationSpecifier::Qualifier($1)]) }
	| declaration_specifiers_untyped storage_class_specifier                                { spec!(self, push!($<mut>1, DeclarationSpecifier::Storage($2))) }
	| declaration_specifiers_untyped type_qualifier                                         { spec!(self, push!($<mut>1, DeclarationSpecifier::Qualifier($2))) }
	;

declaration_specifiers_typed /* Vec<DeclarationSpecifier> — a type has been seen */
	: type_specifier                                                                        { spec!(self, vec![DeclarationSpecifier::Type($1)]) }
	| declaration_specifiers_untyped type_specifier                                         { spec!(self, push!($<mut>1, DeclarationSpecifier::Type($2))) }
	| declaration_specifiers_typed type_specifier_kw                                        { spec!(self, push!($<mut>1, DeclarationSpecifier::Type($2))) }
	| declaration_specifiers_typed storage_class_specifier                                  { spec!(self, push!($<mut>1, DeclarationSpecifier::Storage($2))) }
	| declaration_specifiers_typed type_qualifier                                           { spec!(self, push!($<mut>1, DeclarationSpecifier::Qualifier($2))) }
	;

init_declarator_list /* Vec<InitDeclaratorNode> */
	: init_declarator                                                                       { vec![$1] }
    | init_declarator_list ',' init_declarator                                              { push!($<mut>1, $3) }
	;

init_declarator /* InitDeclaratorNode */
	: declarator                                                                            { with_span!(self, InitDeclaratorNode::new, $1, None) }
	| declarator '=' initializer                                                            { with_span!(self, InitDeclaratorNode::new, $1, Some($3)) }
	;

storage_class_specifier /* Storage*/
	: TYPEDEF                                                                               { Storage::Typedef  }
	| EXTERN                                                                                { Storage::Extern }
	| STATIC                                                                                { Storage::Static }
	| AUTO                                                                                  { Storage::Auto }
	| REGISTER                                                                              { Storage::Register }
    ;

type_qualifier  /* Qualifier */
	: CONST                                                                                 { Qualifier::Const }
	| VOLATILE                                                                              { Qualifier::Volatile }
	;

type_specifier /* TypeSpecifier */
	: type_specifier_kw                                                                     { $1 }
	| TYPE_NAME                                                                             { TypeSpecifier::TypedefName($1) }
	;

type_specifier_kw /* TypeSpecifier */
	: VOID                                                                                  { TypeSpecifier::Void }
	| CHAR                                                                                  { TypeSpecifier::Char }
	| SHORT                                                                                 { TypeSpecifier::Short }
	| INT                                                                                   { TypeSpecifier::Int }
	| LONG                                                                                  { TypeSpecifier::Long }
	| FLOAT                                                                                 { TypeSpecifier::Float }
	| DOUBLE                                                                                { TypeSpecifier::Double }
	| SIGNED                                                                                { TypeSpecifier::Signed }
	| UNSIGNED                                                                              { TypeSpecifier::Unsigned }
	| struct_or_union_specifier                                                             { $1 }
	| enum_specifier                                                                        { TypeSpecifier::Enum($1) }
	;

initializer /* InitializerNode */
	: expression %prec PREC_NO_COMMA                                                        { with_span!(self, InitializerNode::new, Initializer::Single($1)) }
	| '{' initializer_list '}'                                                              { with_span!(self, InitializerNode::new, Initializer::List($2)) }
	| '{' initializer_list ',' '}'                                                          { with_span!(self, InitializerNode::new, Initializer::List($2)) }
	;

initializer_list /* Vec<Initializer> */
	: initializer                                                                           { vec![$1] }
    | initializer_list ',' initializer                                                      { push!($<mut>1, $3) }
	;

declarator /* DeclaratorNode */
	: pointer direct_declarator                                                             { node_span!(self, declarators, with_pointer, $1, $2) }
	| direct_declarator                                                                     { $1 }
	;

direct_declarator /* DeclaratorNode */
	: IDENTIFIER                                                                            { self.lexer.ctx.add_symbol($1.id); node_span!(self, declarators, ident, $1) }
	| '(' declarator ')'                                                                    { $2 }
	| direct_declarator '[' constant_expression ']'                                         { node_span!(self, declarators, array, $1, Some($3)) }
	| direct_declarator '[' ']'                                                             { node_span!(self, declarators, array, $1, None) }
	| direct_declarator '(' enter_scope parameter_type_list exit_scope ')'                  { node_span!(self, declarators, function, $1, $4) }
	| direct_declarator '(' enter_scope identifier_list exit_scope ')'                      { let a = with_span!(self, FunctionParametersNode::old_style, $4); node_span!(self, declarators, function, $1, a) }
	| direct_declarator '(' enter_scope exit_scope ')'                                      { let a = with_span!(self, FunctionParametersNode::empty); node_span!(self, declarators, function, $1, a) }
	;


pointer  /* Vec<Vec<Qualifier>> */
	: '*'                                                                                   { vec![vec![]] }
	| '*' type_qualifier_list                                                               { vec![$2] }
	| '*' pointer                                                                           { push!($<mut>2, vec![]) }
	| '*' type_qualifier_list pointer                                                       { push!($<mut>3, $2) }
	;

type_qualifier_list /* Vec<Qualifier> */
	: type_qualifier                                                                        { vec![$1] }
	| type_qualifier_list type_qualifier                                                    { push!($<mut>1, $2) }
	;

parameter_type_list /* FunctionParametersNode */
	: parameter_list                                                                        { with_span!(self, FunctionParametersNode::param_style, $1) }
	| parameter_list ',' ELLIPSIS                                                           { with_span!(self, FunctionParametersNode::variadic, $1) }
	;

parameter_list /* Vec<ParameterDeclaration> */
	: parameter_declaration                                                                 { vec![$1] }
	| parameter_list ',' parameter_declaration                                              { push!($<mut>1, $3) }
    ;

parameter_declaration /* ParameterDeclaration */
	: declaration_specifiers declarator                                                     { with_span!(self, ParameterDeclaration::new, $1, $2) }
	| declaration_specifiers abstract_declarator                                            { with_span!(self, ParameterDeclaration::new, $1, $2) }
	| declaration_specifiers                                                                { let a = node_span!(self, declarators, abstrct); with_span!(self, ParameterDeclaration::new, $1, a) }
	;

identifier_list /* Vec<Name> */
	: IDENTIFIER                                                                            { vec![$1] }
	| identifier_list ',' IDENTIFIER                                                        { push!($<mut>1, $3) }
	;

type_name /* Type */
	: specifier_qualifier_list                                                              { Type { specifiers: $1, declarator: node_span!(self, declarators, abstrct) } }
	| specifier_qualifier_list abstract_declarator                                          { Type { specifiers: $1, declarator: $2 } }
	;

specifier_qualifier_list /* Vec<DeclarationSpecifier> */
	: specifier_qualifier_list_untyped                                                      { $1 }
	| specifier_qualifier_list_typed                                                        { $1 }
	;

specifier_qualifier_list_untyped /* Vec<DeclarationSpecifier> — no type specifier yet */
	: type_qualifier                                                                        { spec!(self, vec![DeclarationSpecifier::Qualifier($1)]) }
	| specifier_qualifier_list_untyped type_qualifier                                       { spec!(self, push!($<mut>1, DeclarationSpecifier::Qualifier($2))) }
	;

specifier_qualifier_list_typed /* Vec<DeclarationSpecifier> — a type specifier has been seen */
	: type_specifier                                                                        { spec!(self, vec![DeclarationSpecifier::Type($1)]) }
	| specifier_qualifier_list_untyped type_specifier                                       { spec!(self, push!($<mut>1, DeclarationSpecifier::Type($2))) }
	| specifier_qualifier_list_typed type_specifier_kw                                      { spec!(self, push!($<mut>1, DeclarationSpecifier::Type($2))) }
	| specifier_qualifier_list_typed type_qualifier                                         { spec!(self, push!($<mut>1, DeclarationSpecifier::Qualifier($2))) }
	;

abstract_declarator /* DeclaratorNode */
	: pointer                                                                               { let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, with_pointer, $1, a) }
	| direct_abstract_declarator                                                            { $1 }
	| pointer direct_abstract_declarator                                                    { node_span!(self, declarators, with_pointer, $1, $2) }
	;

direct_abstract_declarator /* DeclaratorNode */
	: '(' abstract_declarator ')'                                                           { $2 }
	| '[' ']'                                                                               { let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, array, a, None) }
	| '[' constant_expression ']'                                                           { let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, array, a, Some($2)) }
	| direct_abstract_declarator '[' ']'                                                    { node_span!(self, declarators, array, $1, None) }
	| direct_abstract_declarator '[' constant_expression ']'                                { node_span!(self, declarators, array, $1, Some($3)) }
	| '(' ')'                                                                               { let a = node_span!(self, declarators, abstrct); let b = with_span!(self, FunctionParametersNode::empty); node_span!(self, declarators, function, a, b) }
	| '(' parameter_type_list ')'                                                           { let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, function, a, $2) }
	| direct_abstract_declarator '(' ')'                                                    { let a = with_span!(self, FunctionParametersNode::empty); node_span!(self, declarators, function, $1, a) }
	| direct_abstract_declarator '(' enter_scope parameter_type_list exit_scope ')'         { node_span!(self, declarators, function, $1, $4) }
	;

struct_or_union_specifier /* TypeSpecifier */
	: struct_or_union '{' enter_struct struct_declaration_list exit_struct '}'              { let s = self.span; self.lexer.ctx.struct_or_union($1, None, $4, s) }
	| struct_or_union IDENTIFIER '{' enter_struct struct_declaration_list exit_struct '}'   { let s = self.span; self.lexer.ctx.struct_or_union($1, Some($2), $5, s) }
	| struct_or_union IDENTIFIER                                                            { let s = self.span; self.lexer.ctx.struct_or_union($1, Some($2), vec![], s) }
	;

struct_or_union /* Tag */
	: STRUCT                                                                                { Tag::Struct }
	| UNION                                                                                 { Tag::Union }
	;

struct_declaration_list /* Vec<StructDeclaration> */
	: struct_declaration                                                                    { vec![$1] }
	| struct_declaration_list struct_declaration                                            { push!($<mut>1, $2) }
	;

struct_declaration /* StructDeclaration */
	: specifier_qualifier_list struct_declarator_list ';'                                   { with_span!(self, StructDeclaration::new, $1, $2)  }
	;

struct_declarator_list /* Vec<StructMemberDeclarator> */
	: struct_declarator                                                                     { vec![$1] }
	| struct_declarator_list ',' struct_declarator                                          { push!($<mut>1, $3) }
	;

struct_declarator /* StructMemberDeclarator */
	: declarator                                                                            { with_span!(self, StructMemberDeclarator::new, $1, None) }
	| ':' constant_expression                                                               { let d = node_span!(self, declarators, abstrct); with_span!(self, StructMemberDeclarator::new, d, Some($2)) }
	| declarator ':' constant_expression                                                    { with_span!(self, StructMemberDeclarator::new, $1, Some($3)) }
	;

enum_specifier /* EnumId */
	: ENUM '{' enumerator_list '}'                                                          { node_span!(self, enums, add, None, $3) }
	| ENUM IDENTIFIER '{' enumerator_list '}'                                               { node_span!(self, enums, add, Some($2), $4) }
	| ENUM IDENTIFIER                                                                       { node_span!(self, enums, add, Some($2), vec![]) }
	;

enumerator_list /* Vec<VariantId> */
	: enumerator                                                                            { vec![$1] }
	| enumerator_list ',' enumerator                                                        { push!($<mut>1, $3) }
	;

enumerator /* VariantId */
	: IDENTIFIER                                                                            { node_span!(self, variants, add, $1, None) }
	| IDENTIFIER '=' constant_expression                                                    { node_span!(self, variants, add, $1, Some($3)) }
	;

statement /* StatementNode */
	: labeled_statement                                                                     { node_span!(self, statements, labeled, $1) }
	| compound_statement                                                                    { node_span!(self, statements, compund, $1)}
	| expression_statement                                                                  { node_span!(self, statements, expression, $1) }
	| selection_statement                                                                   { node_span!(self, statements, selection, $1) }
	| iteration_statement                                                                   { node_span!(self, statements, iteration, $1) }
	| jump_statement                                                                        { node_span!(self, statements, jump, $1) }
	;

labeled_statement /* LabeledStatementNode */
	: IDENTIFIER ':' statement                                                              { with_span!(self, LabeledStatementNode::identifier, $1, $3) }
	| TYPE_NAME ':' statement                                                               { with_span!(self, LabeledStatementNode::identifier, $1, $3) }
	| CASE constant_expression ':' statement                                                { with_span!(self, LabeledStatementNode::case, $2, $4) }
	| DEFAULT ':' statement                                                                 { with_span!(self, LabeledStatementNode::default, $3) }
	;

compound_statement /* CompoundStatementNode */
	: '{' enter_scope '}' exit_scope                                                        { with_span!(self, CompoundStatementNode::new, vec![], vec![]) }
	| '{' enter_scope statement_list '}' exit_scope                                         { with_span!(self, CompoundStatementNode::new, vec![], $3) }
	| '{' enter_scope declaration_list '}' exit_scope                                       { with_span!(self, CompoundStatementNode::new, $3, vec![]) }
	| '{' enter_scope declaration_list statement_list '}' exit_scope                        { with_span!(self, CompoundStatementNode::new, $3, $4) }
	;

declaration_list /* Vec<DeclarationNode> */
	: declaration                                                                           { vec![$1] }
	| declaration_list declaration                                                          { push!($<mut>1, $2)}
	;

statement_list /* Vec<StatementNode> */
	: statement                                                                             { vec![$1] }
	| statement_list statement                                                              { push!($<mut>1, $2)}
	;

expression_statement /* ExpressionStatementNode */
	: ';'                                                                                   { with_span!(self, ExpressionStatementNode::new, None) }
	| expression ';'                                                                        { with_span!(self, ExpressionStatementNode::new, Some($1)) }
	;

selection_statement /* SelectionStatementNode */
	: IF '(' expression ')' statement %prec PREC_THEN                                       { with_span!(self, SelectionStatementNode::new_if, $3, $5, None) }
	| IF '(' expression ')' statement ELSE statement                                        { with_span!(self, SelectionStatementNode::new_if, $3, $5, Some($7))  }
	| SWITCH '(' expression ')' statement                                                   { with_span!(self, SelectionStatementNode::switch, $3, $5) } ;

iteration_statement /* IterationStatementNode */
	: WHILE '(' expression ')' statement                                                    { with_span!(self, IterationStatementNode::new_while, $3, $5) }
	| DO statement WHILE '(' expression ')' ';'                                             { with_span!(self, IterationStatementNode::new_do, $2, $5) }
	| FOR '(' expression_statement expression_statement ')' statement                       { with_span!(self, IterationStatementNode::new_for, $3, $4, None, $6) }
	| FOR '(' expression_statement expression_statement expression ')' statement            { with_span!(self, IterationStatementNode::new_for, $3, $4, Some($5), $7) }
	;

jump_statement /* JumpStatementNode */
	: GOTO IDENTIFIER ';'                                                                   { with_span!(self, JumpStatementNode::goto, $2) }
	| CONTINUE ';'                                                                          { with_span!(self, JumpStatementNode::new, JumpStatement::Continue) }
	| BREAK ';'                                                                             { with_span!(self, JumpStatementNode::new, JumpStatement::Break) }
	| RETURN ';'                                                                            { with_span!(self, JumpStatementNode::new_return, None) }
    | RETURN expression ';'                                                                 { with_span!(self, JumpStatementNode::new_return, Some($2)) }
	;

%%
