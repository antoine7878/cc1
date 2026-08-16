%no_main
%{
use crate::ast::{Qualifier, Type, ExpressionNode, Name, DeclarationSpecifier, Initializer, TypeSpecifier, ParameterDeclaration};
use crate::ast::{DeclarationNode, InitDeclaratorNode, DeclaratorNode, InitializerNode, Storage, FunctionParametersNode, Tag};
use crate::ast::{StructDeclaration, StructDeclarator, VariantId, EnumId, LabeledStatementNode, StatementNode, Labeled, CompoundStatementNode};
use crate::ast::{ExpressionStatementNode, SelectionStatementNode, IterationStatementNode, JumpStatementNode, JumpStatement};
use crate::ast::{ExternalDeclarationNode, FunctionDefinitionNode, TranslationUnitNode};

use crate::parser::{YYLex, Context};
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

%}

%token<Name> IDENTIFIER STRING_LITERAL CONSTANT TYPE_NAME
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
%nonassoc EQ_OP NE_OP
%nonassoc '<' '>' LE_OP GE_OP
%left LEFT_OP RIGHT_OP
%left '+' '-'
%left '*' '/' '%'
%right '!' '~' INC_OP DEC_OP POST_INC_OP POST_DEC_OP SIZEOF PREC_UNARY
%nonassoc '(' '[' '.' PTR_OP
%nonassoc PREC_THEN
%nonassoc ELSE

%type<Vec<Name>> identifier_list
%type<Type> type_name

%type<ExpressionNode> expression constant_expression

%type<DeclarationNode> declaration
%type<Vec<DeclarationNode>> declaration_list
%type<TypeSpecifier> type_specifier struct_or_union_specifier
%type<Storage> storage_class_specifier
%type<Qualifier> type_qualifier
%type<Vec<Qualifier>> type_qualifier_list
%type<Vec<DeclarationSpecifier>> declaration_specifiers
%type<InitDeclaratorNode> declarator_list init_declarator
%type<Vec<InitDeclaratorNode>> init_declarator_list
%type<InitializerNode> initializer
%type<Vec<InitializerNode>> initializer_list
%type<DeclaratorNode> declarator direct_declarator pointer direct_abstract_declarator abstract_declarator
%type<FunctionParametersNode> parameter_type_list
%type<Vec<ParameterDeclaration>> parameter_list
%type<ParameterDeclaration> parameter_declaration
%type<Vec<DeclarationSpecifier>> specifier_qualifier_list

%type<Tag> struct_or_union
%type<Vec<StructDeclaration>> struct_declaration_list
%type<StructDeclaration> struct_declaration
%type<Vec<StructDeclarator>> struct_declarator_list
%type<StructDeclarator> struct_declarator
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
%type<()> translation_unit

%%

translation_unit /* (TranslationUnitNode) */
	: external_declaration_list                                             { let ast = with_span!(self, TranslationUnitNode::new, $1); self.lexer.ctx.ast = ast; }
	;

external_declaration_list /* Vec<ExternalDeclarationNode> */
	: external_declaration                                                  { vec![$1] }
	| external_declaration_list external_declaration                        { push!($<mut>1, $2) }
	;

external_declaration /* ExternalDeclarationNode */
	: function_definition                                                   { with_span!(self, ExternalDeclarationNode::function, $1) }
	| declaration                                                           { with_span!(self, ExternalDeclarationNode::declaration, $1) }
	;

function_definition /* FunctionDefinitionNode */
	: declaration_specifiers declarator declaration_list compound_statement { with_span!(self, FunctionDefinitionNode::new, $1, $2, $3, $4) }
	| declaration_specifiers declarator compound_statement                  { with_span!(self, FunctionDefinitionNode::new, $1, $2, vec![], $3) }
	| declarator declaration_list compound_statement                        { with_span!(self, FunctionDefinitionNode::new, vec![], $1, $2, $3) }
	| declarator compound_statement                                         { with_span!(self, FunctionDefinitionNode::new, vec![], $1, vec![], $2) }
	;


constant_expression /* ExpressionId */
    : expression %prec PREC_NO_COMMA                                        { node_span!(self, expressions, constant_expression, $1) }
    ;

expression /* ExpressionId */
    : '(' expression ')'                                                    { $2 }
    | IDENTIFIER                                                            { node_span!(self, expressions, identifier, $1) }
    | CONSTANT                                                              { node_span!(self, expressions, constant,$1)}
    | STRING_LITERAL                                                        { node_span!(self, expressions, string_literal,$1) }
    | expression '[' expression ']'                                         { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '(' ')'                                                    { node_span!(self, expressions, function_call, $1, None) }
    | expression '(' expression ')'                                         { node_span!(self, expressions, function_call, $1, Some($3)) }
    | expression '.' IDENTIFIER                                             { node_span!(self, expressions, access, $1, $2, $3) }
    | expression PTR_OP IDENTIFIER                                          { node_span!(self, expressions, access, $1, $2, $3) }
    | SIZEOF '(' expression ')'                                             { node_span!(self, expressions, sizeof_expr, $3) }
    | SIZEOF '(' type_name ')'                                              { node_span!(self, expressions, sizeof_type, $3) }
    | '(' type_name ')' expression %prec PREC_UNARY                         { node_span!(self, expressions, cast, $2, $4) }
    | expression INC_OP                                                     { node_span!(self, expressions, unary, YYToken::POST_INC_OP, $1) }
    | expression DEC_OP                                                     { node_span!(self, expressions, unary, YYToken::POST_DEC_OP, $1) }
    | INC_OP expression                                                     { node_span!(self, expressions, unary, $1, $2) }
    | DEC_OP expression                                                     { node_span!(self, expressions, unary, $1, $2) }
    | '&' expression %prec PREC_UNARY                                       { node_span!(self, expressions, unary, $1, $2) }
    | '*' expression %prec PREC_UNARY                                       { node_span!(self, expressions, unary, $1, $2) }
    | '+' expression %prec PREC_UNARY                                       { node_span!(self, expressions, unary, $1, $2) }
    | '-' expression %prec PREC_UNARY                                       { node_span!(self, expressions, unary, $1, $2) }
    | '~' expression                                                        { node_span!(self, expressions, unary, $1, $2) }
    | '!' expression                                                        { node_span!(self, expressions, unary, $1, $2) }
    | expression '+' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '-' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '*' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '/' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '%' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression LEFT_OP expression                                         { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression RIGHT_OP expression                                        { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '<' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '>' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression LE_OP expression                                           { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression GE_OP expression                                           { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression EQ_OP expression                                           { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression NE_OP expression                                           { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression '&' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression '^' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression '|' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression AND_OP expression                                          { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression OR_OP expression                                           { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression '=' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression MUL_ASSIGN expression                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression DIV_ASSIGN expression                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression MOD_ASSIGN expression                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression ADD_ASSIGN expression                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression SUB_ASSIGN expression                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression LEFT_ASSIGN expression                                     { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression RIGHT_ASSIGN expression                                    { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression AND_ASSIGN expression                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression XOR_ASSIGN expression                                      { node_span!(self, expressions, binary, $1, $2, $3) }
	| expression OR_ASSIGN expression                                       { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression ',' expression                                             { node_span!(self, expressions, binary, $1, $2, $3) }
    | expression '?' expression ':' expression                              { node_span!(self, expressions, ternary, $1, $3, $5) }
    ;

declaration /* DeclarationNode */
	: declaration_specifiers ';'                                            { let node = with_span!(self, DeclarationNode::new, $1, vec![]); self.lexer.ctx.add_symbol(&node); node }
	| declaration_specifiers init_declarator_list ';'                       { let node = with_span!(self, DeclarationNode::new, $1, $2); self.lexer.ctx.add_symbol(&node); node }
	;

declaration_specifiers /* Vec<DeclarationSpecifier> */
	: storage_class_specifier                                               { vec![DeclarationSpecifier::Storage($1)] }
	| storage_class_specifier declaration_specifiers                        { push!($<mut>2, DeclarationSpecifier::Storage($1)) }
	| type_specifier                                                        { vec![DeclarationSpecifier::Type($1)] }
	| type_specifier declaration_specifiers                                 { push!($<mut>2, DeclarationSpecifier::Type($1)) }
	| type_qualifier                                                        { vec![DeclarationSpecifier::Qualifier($1)] }
	| type_qualifier declaration_specifiers                                 { push!($<mut>2, DeclarationSpecifier::Qualifier($1)) }
	;

init_declarator_list /* Vec<InitDeclaratorNode> */
	: init_declarator                                                       { vec![$1] }
    | init_declarator_list ',' init_declarator                              { push!($<mut>1, $3) }
	;

init_declarator /* InitDeclaratorNode */
	: declarator                                                            { with_span!(self, InitDeclaratorNode::new, $1, None) }
	| declarator '=' initializer                                            { with_span!(self, InitDeclaratorNode::new, $1, Some($3)) }
	;

storage_class_specifier /* Storage*/
	: TYPEDEF                                                               { Storage::Typedef  }
	| EXTERN                                                                { Storage::Extern }
	| STATIC                                                                { Storage::Static }
	| AUTO                                                                  { Storage::Auto }
	| REGISTER                                                              { Storage::Register }
    ;

type_qualifier  /* Qualifier */
	: CONST                                                                 { Qualifier::Const }
	| VOLATILE                                                              { Qualifier::Volatile }
	;

type_specifier /* TypeSpecifier */
	: VOID                                                                  { TypeSpecifier::Void }
	| CHAR                                                                  { TypeSpecifier::Char }
	| SHORT                                                                 { TypeSpecifier::Short }
	| INT                                                                   { TypeSpecifier::Int }
	| LONG                                                                  { TypeSpecifier::Long }
	| FLOAT                                                                 { TypeSpecifier::Float }
	| DOUBLE                                                                { TypeSpecifier::Double }
	| SIGNED                                                                { TypeSpecifier::Signed }
	| UNSIGNED                                                              { TypeSpecifier::Unsigned }
	| struct_or_union_specifier                                             { $1 }
	| enum_specifier                                                        { TypeSpecifier::Enum($1) }
	| TYPE_NAME                                                             { TypeSpecifier::TypedefName($1) }
	;

initializer /* InitializerNode */
	: expression %prec PREC_NO_COMMA                                        { with_span!(self, InitializerNode::new, Initializer::Single($1)) }
	| '{' initializer_list '}'                                              { with_span!(self, InitializerNode::new, Initializer::List($2)) }
	| '{' initializer_list ',' '}'                                          { with_span!(self, InitializerNode::new, Initializer::List($2)) }
	;

initializer_list /* Vec<Initializer> */
	: initializer                                                           { vec![$1] }
    | initializer_list ',' initializer                                      { push!($<mut>1, $3) }
	;

declarator /* DeclaratorNode */
	: pointer direct_declarator                                             { node_span!(self, declarators, with_pointer, $1, $2) }
	| direct_declarator                                                     { $1 }
	;

direct_declarator /* DeclaratorNode */
	: IDENTIFIER                                                            { node_span!(self, declarators, ident, $1) }
	| '(' declarator ')'                                                    { $2 }
	| direct_declarator '[' constant_expression ']'                         { node_span!(self, declarators, array, $1, Some($3)) }
	| direct_declarator '[' ']'                                             { node_span!(self, declarators, array, $1, None) }
	| direct_declarator '(' parameter_type_list ')'                         { node_span!(self, declarators, function, $1, $3) }
	| direct_declarator '(' identifier_list ')'                             { let a = with_span!(self, FunctionParametersNode::old_style, $3); node_span!(self, declarators, function, $1, a) }
	| direct_declarator '(' ')'                                             { let a = with_span!(self, FunctionParametersNode::empty); node_span!(self, declarators, function, $1, a) }
	;

pointer /* Declarator::Pointer */
	: '*'                                                                   { node_span!(self, declarators, pointer, vec![], None)     }
	| '*' type_qualifier_list                                               { node_span!(self, declarators, pointer, $2,     None)     }
	| '*' pointer                                                           { node_span!(self, declarators, pointer, vec![], Some($2)) }
	| '*' type_qualifier_list pointer                                       { node_span!(self, declarators, pointer, $2,     Some($3)) }
	;

type_qualifier_list /* Vec<Qualifier> */
	: type_qualifier                                                        { vec![$1] }
	| type_qualifier_list type_qualifier                                    { push!($<mut>1, $2) }
	;

parameter_type_list /* FunctionParametersNode */
	: parameter_list                                                        { with_span!(self, FunctionParametersNode::param_style, $1) }
	| parameter_list ',' ELLIPSIS                                           { with_span!(self, FunctionParametersNode::variadic, $1) }
	;

parameter_list /* Vec<ParameterDeclaration> */
	: parameter_declaration                                                 { vec![$1] }
	| parameter_list ',' parameter_declaration                              { push!($<mut>1, $3) }
    ;

parameter_declaration /* ParameterDeclaration */
	: declaration_specifiers declarator                                     { with_span!(self, ParameterDeclaration::new, $1, $2) }
	| declaration_specifiers abstract_declarator                            { with_span!(self, ParameterDeclaration::new, $1, $2) }
	| declaration_specifiers                                                { let a = node_span!(self, declarators, abstrct); with_span!(self, ParameterDeclaration::new, $1, a) }
	;

identifier_list /* Vec<Name> */
	: IDENTIFIER                                                            { vec![$1] }
	| identifier_list ',' IDENTIFIER                                        { push!($<mut>1, $3) }
	;

type_name /* Type */
	: specifier_qualifier_list                                      { Type { specifiers: $1, declarator: node_span!(self, declarators, abstrct) } }
	| specifier_qualifier_list abstract_declarator                 { Type { specifiers: $1, declarator: $2 } }
	;

specifier_qualifier_list /* Vec<DeclarationSpecifier> */
	: type_specifier specifier_qualifier_list                               { push!($<mut>2, DeclarationSpecifier::Type($1)) }
	| type_specifier                                                        { vec![DeclarationSpecifier::Type($1)] }
	| type_qualifier specifier_qualifier_list                               { push!($<mut>2, DeclarationSpecifier::Qualifier($1)) }
	| type_qualifier                                                        { vec![DeclarationSpecifier::Qualifier($1)] }
	;

abstract_declarator /* DeclaratorNode */
	: pointer                                                               { let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, with_pointer, $1, a) }
	| direct_abstract_declarator                                            { $1 }
	| pointer direct_abstract_declarator                                    { node_span!(self, declarators, with_pointer, $1, $2) }
	;

direct_abstract_declarator /* DeclaratorNode */
	: '(' abstract_declarator ')'                                           { $2 }
	| '[' ']'                                                               { let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, array, a, None) }
	| '[' constant_expression ']'                                           { let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, array, a, Some($2)) }
	| direct_abstract_declarator '[' ']'                                    { node_span!(self, declarators, array, $1, None) }
	| direct_abstract_declarator '[' constant_expression ']'                { node_span!(self, declarators, array, $1, Some($3)) }
	| '(' ')'                                                               { let a = node_span!(self, declarators, abstrct); let b = with_span!(self, FunctionParametersNode::empty); node_span!(self, declarators, function, a, b) }
	| '(' parameter_type_list ')'                                           { let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, function, a, $2) }
	| direct_abstract_declarator '(' ')'                                    { let a = with_span!(self, FunctionParametersNode::empty); node_span!(self, declarators, function, $1, a) }
	| direct_abstract_declarator '(' parameter_type_list ')'                { node_span!(self, declarators, function, $1, $3) }
	;

struct_or_union_specifier /* TypeSpecifier */
	: struct_or_union '{' struct_declaration_list '}'                       { let s = self.span; self.lexer.ctx.struct_or_union($1, None, $3, s) }
	| struct_or_union IDENTIFIER '{' struct_declaration_list '}'            { let s = self.span; self.lexer.ctx.struct_or_union($1, Some($2), $4, s) }
	| struct_or_union IDENTIFIER                                            { let s = self.span; self.lexer.ctx.struct_or_union($1, Some($2), vec![], s) }
	;

struct_or_union /* Tag */
	: STRUCT                                                                { Tag::Struct }
	| UNION                                                                 { Tag::Union }
	;

struct_declaration_list /* Vec<StructDeclaration> */
	: struct_declaration                                                    { vec![$1] }
	| struct_declaration_list struct_declaration                            { push!($<mut>1, $2) }
	;

struct_declaration /* StructDeclaration */
	: specifier_qualifier_list struct_declarator_list ';'                   { with_span!(self, StructDeclaration::new, $1, $2)  }
	;

struct_declarator_list /* Vec<StructDeclarator> */
	: struct_declarator                                                     { vec![$1] }
	| struct_declarator_list ',' struct_declarator                          { push!($<mut>1, $3) }
	;

struct_declarator /* StructDeclarator */
	: declarator                                                            { with_span!(self, StructDeclarator::new, $1, None) }
	| ':' constant_expression                                               { let d = node_span!(self, declarators, abstrct); with_span!(self, StructDeclarator::new, d, Some($2)) }
	| declarator ':' constant_expression                                    { with_span!(self, StructDeclarator::new, $1, Some($3)) }
	;

enum_specifier /* EnumId */
	: ENUM '{' enumerator_list '}'                                          { node_span!(self, enums, add, None, $3) }
	| ENUM IDENTIFIER '{' enumerator_list '}'                               { node_span!(self, enums, add, Some($2), $4) }
	| ENUM IDENTIFIER                                                       { node_span!(self, enums, add, Some($2), vec![]) }
	;

enumerator_list /* Vec<VariantId> */
	: enumerator                                                            { vec![$1] }
	| enumerator_list ',' enumerator                                        { push!($<mut>1, $3) }
	;

enumerator /* VariantId */
	: IDENTIFIER                                                            { node_span!(self, variants, add, $1, None) }
	| IDENTIFIER '=' constant_expression                                    { node_span!(self, variants, add, $1, Some($3)) }
	;

statement /* StatementNode */
	: labeled_statement                                                     { node_span!(self, statements, labeled, $1) }
	| compound_statement                                                    { node_span!(self, statements, compund, $1)}
	| expression_statement                                                  { node_span!(self, statements, expression, $1) }
	| selection_statement                                                   { node_span!(self, statements, selection, $1) }
	| iteration_statement                                                   { node_span!(self, statements, iteration, $1) }
	| jump_statement                                                        { node_span!(self, statements, jump, $1) }
	;

labeled_statement /* LabeledStatementNode */
	: IDENTIFIER ':' statement                                              { with_span!(self, LabeledStatementNode::identifier, $1, $3) }
	| CASE constant_expression ':' statement                                { with_span!(self, LabeledStatementNode::case, $2, $4) }
	| DEFAULT ':' statement                                                 { with_span!(self, LabeledStatementNode::default, $3) }
	;

compound_statement /* CompoundStatementNode */
	: '{' '}'                                                               { self.lexer.ctx.push_scope(); let node = with_span!(self, CompoundStatementNode::new, vec![], vec![]); self.lexer.ctx.pop_scope(); node }
	| '{' statement_list '}'                                                { self.lexer.ctx.push_scope(); let node = with_span!(self, CompoundStatementNode::new, vec![], $2); self.lexer.ctx.pop_scope(); node }
	| '{' declaration_list '}'                                              { self.lexer.ctx.push_scope(); let node = with_span!(self, CompoundStatementNode::new, $2, vec![]); self.lexer.ctx.pop_scope(); node }
	| '{' declaration_list statement_list '}'                               { self.lexer.ctx.push_scope(); let node = with_span!(self, CompoundStatementNode::new, $2, $3); self.lexer.ctx.pop_scope(); node }
	;

declaration_list /* Vec<DeclarationNode> */
	: declaration                                                           { vec![$1] }
	| declaration_list declaration                                          { push!($<mut>1, $2)}
	;

statement_list /* Vec<StatementNode> */
	: statement                                                             { vec![$1] }
	| statement_list statement                                              { push!($<mut>1, $2)}
	;

expression_statement /* ExpressionStatementNode */
	: ';'                                                                   { with_span!(self, ExpressionStatementNode::new, None) }
	| expression ';'                                                        { with_span!(self, ExpressionStatementNode::new, Some($1)) }
	;

selection_statement /* SelectionStatementNode */
	: IF '(' expression ')' statement %prec PREC_THEN                       { with_span!(self, SelectionStatementNode::new_if, $3, $5, None) }
	| IF '(' expression ')' statement ELSE statement                        { with_span!(self, SelectionStatementNode::new_if, $3, $5, Some($7))  }
	| SWITCH '(' expression ')' statement                                   { with_span!(self, SelectionStatementNode::switch, $3, $5) } ;

iteration_statement /* IterationStatementNode */
	: WHILE '(' expression ')' statement                                    { with_span!(self, IterationStatementNode::new_while, $3, $5) }
	| DO statement WHILE '(' expression ')' ';'                             { with_span!(self, IterationStatementNode::new_do, $2, $5) }
	| FOR '(' expression_statement expression_statement ')' statement               { with_span!(self, IterationStatementNode::new_for, $3, $4, None, $6) }
	| FOR '(' expression_statement expression_statement expression ')' statement    { with_span!(self, IterationStatementNode::new_for, $3, $4, Some($5), $7) }
	;

jump_statement /* JumpStatementNode */
	: GOTO IDENTIFIER ';'                                                   { with_span!(self, JumpStatementNode::new, JumpStatement::Goto) }
	| CONTINUE ';'                                                          { with_span!(self, JumpStatementNode::new, JumpStatement::Continue) }
	| BREAK ';'                                                             { with_span!(self, JumpStatementNode::new, JumpStatement::Break) }
	| RETURN ';'                                                            { with_span!(self, JumpStatementNode::new_return, None) }
    | RETURN expression ';'                                                 { with_span!(self, JumpStatementNode::new_return, Some($2)) }
	;

%%
