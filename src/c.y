%no_main
%{
use crate::context::ContextAccess;
use crate::ast::{Qualifier, TypeNode, ExpressionNode, Name, DeclarationSpecifier, Initializer, Field};
use crate::ast::{DeclarationNode,InitDeclaratorNode, DeclaratorNode, InitializerNode, Storage};
use crate::parser::YYLex;
use crate::error::yyerror;

macro_rules! node {
    ($self:expr, $factory:ident, $method:ident $(, $arg:expr)* $(,)?) => {{
        let span = $self.span;
        $self.$factory().$method($($arg,)* span)
    }};
}
%}

%token<Name> IDENTIFIER STRING_LITERAL CONSTANT
%token TYPEDEF EXTERN STATIC AUTO REGISTER
%token CHAR SHORT INT LONG SIGNED UNSIGNED FLOAT DOUBLE CONST VOLATILE VOID
%token STRUCT UNION ENUM ELLIPSIS
%token CASE DEFAULT IF ELSE SWITCH WHILE DO FOR GOTO CONTINUE BREAK RETURN

%left ','
%right '=' SUB_ASSIGN LEFT_ASSIGN RIGHT_ASSIGN AND_ASSIGN MUL_ASSIGN
      DIV_ASSIGN MOD_ASSIGN ADD_ASSIGN XOR_ASSIGN OR_ASSIGN TYPE_NAME
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

%type<ExpressionNode> expression constant_expression

%type<DeclarationNode> declaration
%type<Vec<DeclarationSpecifier>> declaration_specifiers
%type<>

%type<InitDeclaratorNode> declarator_list
    %type<Vec<InitDeclaratorNode>> init_declarator_list
        %type<Storage> storage_class_specifier
        %type<TypeNode> type_name type_specifier
        %type<Qualifier> type_qualifier
    %type<InitializerNode> initializer
        %type<Vec<InitializerNode>> initializer_list

%type<DeclaratorNode> declarator


%type<Vec<Qualifier>> specifier_qualifier_list

/*
%type<Vec<Field>> struct_declaration_list
%type<Vec<Field>> struct_declaration
%type<Vec<TypeId>> struct_declarator_list
%type<NameId> struct_declarator
%type<StructId> struct_specifier
*/

%%

unit
    : constant_expression { self.lexer.ctx.print_ast(&$1); YYToken::unit }
    ;

constant_expression /* ExpressionId */
    : expression { node!(self, expressions, constant_expression, $1) }
    ;

expression /* ExpressionId */
    : '(' expression ')'                            { $2 }
    | IDENTIFIER                                    { node!(self, expressions, identifier, $1) }
    | CONSTANT                                      { node!(self, expressions, constant,$1)}
    | STRING_LITERAL                                { node!(self, expressions, string_literal,$1) }
    | expression '[' expression ']'                 { node!(self, expressions, binary, $1, $2, $3) }
    | expression '(' ')'                            { node!(self, expressions, function_call, $1, None) }
    | expression '(' expression ')'                 { node!(self, expressions, function_call, $1, Some($3)) }
    | expression '.' IDENTIFIER                     { node!(self, expressions, access, $1, $2, $3) }
    | expression PTR_OP IDENTIFIER                  { node!(self, expressions, access, $1, $2, $3) }
    | SIZEOF '(' expression ')'                     { node!(self, expressions, sizeof_expr, $3) }
    | SIZEOF '(' type_name ')'                      { node!(self, expressions, sizeof_type, $3) }
    | '(' type_name ')' expression %prec PREC_UNARY { node!(self, expressions, cast, $2, $4) }
    | expression INC_OP                             { node!(self, expressions, unary, YYToken::POST_INC_OP, $1) }
    | expression DEC_OP                             { node!(self, expressions, unary, YYToken::POST_DEC_OP, $1) }
    | INC_OP expression                             { node!(self, expressions, unary, $1, $2) }
    | DEC_OP expression                             { node!(self, expressions, unary, $1, $2) }
    | '&' expression %prec PREC_UNARY               { node!(self, expressions, unary, $1, $2) }
    | '*' expression %prec PREC_UNARY               { node!(self, expressions, unary, $1, $2) }
    | '+' expression %prec PREC_UNARY               { node!(self, expressions, unary, $1, $2) }
    | '-' expression %prec PREC_UNARY               { node!(self, expressions, unary, $1, $2) }
    | '~' expression                                { node!(self, expressions, unary, $1, $2) }
    | '!' expression                                { node!(self, expressions, unary, $1, $2) }
    | expression '+' expression                     { node!(self, expressions, binary, $1, $2, $3) }
    | expression '-' expression                     { node!(self, expressions, binary, $1, $2, $3) }
    | expression '*' expression                     { node!(self, expressions, binary, $1, $2, $3) }
    | expression '/' expression                     { node!(self, expressions, binary, $1, $2, $3) }
    | expression '%' expression                     { node!(self, expressions, binary, $1, $2, $3) }
    | expression LEFT_OP expression                 { node!(self, expressions, binary, $1, $2, $3) }
    | expression RIGHT_OP expression                { node!(self, expressions, binary, $1, $2, $3) }
    | expression '<' expression                     { node!(self, expressions, binary, $1, $2, $3) }
    | expression '>' expression                     { node!(self, expressions, binary, $1, $2, $3) }
    | expression LE_OP expression                   { node!(self, expressions, binary, $1, $2, $3) }
    | expression GE_OP expression                   { node!(self, expressions, binary, $1, $2, $3) }
	| expression EQ_OP expression                   { node!(self, expressions, binary, $1, $2, $3) }
	| expression NE_OP expression                   { node!(self, expressions, binary, $1, $2, $3) }
	| expression '&' expression                     { node!(self, expressions, binary, $1, $2, $3) }
	| expression '^' expression                     { node!(self, expressions, binary, $1, $2, $3) }
	| expression '|' expression                     { node!(self, expressions, binary, $1, $2, $3) }
	| expression AND_OP expression                  { node!(self, expressions, binary, $1, $2, $3) }
	| expression OR_OP expression                   { node!(self, expressions, binary, $1, $2, $3) }
	| expression '=' expression                     { node!(self, expressions, binary, $1, $2, $3) }
	| expression MUL_ASSIGN expression              { node!(self, expressions, binary, $1, $2, $3) }
	| expression DIV_ASSIGN expression              { node!(self, expressions, binary, $1, $2, $3) }
	| expression MOD_ASSIGN expression              { node!(self, expressions, binary, $1, $2, $3) }
	| expression ADD_ASSIGN expression              { node!(self, expressions, binary, $1, $2, $3) }
	| expression SUB_ASSIGN expression              { node!(self, expressions, binary, $1, $2, $3) }
	| expression LEFT_ASSIGN expression             { node!(self, expressions, binary, $1, $2, $3) }
	| expression RIGHT_ASSIGN expression            { node!(self, expressions, binary, $1, $2, $3) }
	| expression AND_ASSIGN expression              { node!(self, expressions, binary, $1, $2, $3) }
	| expression XOR_ASSIGN expression              { node!(self, expressions, binary, $1, $2, $3) }
	| expression OR_ASSIGN expression               { node!(self, expressions, binary, $1, $2, $3) }
    | expression ',' expression                     { node!(self, expressions, binary, $1, $2, $3) }
    | expression '?' expression ':' expression      { node!(self, expressions, ternary, $1, $3, $5) }
    ;

type_name
    :
	;
%%
// declaration /* Declaration */
// 	: declaration_specifiers ';'
// 	| declaration_specifiers init_declarator_list ';'
// 	;
//
// declaration_specifiers /* Vec<DeclarationSpecifier> */
// 	: storage_class_specifier                           { vec![DeclarationSpecifier::Storage($1)] }
// 	| storage_class_specifier declaration_specifiers    { $1.push($2); $1 }
// 	| type_specifier                                    { vec![DeclarationSpecifier::Qualifier($1)] }
// 	| type_specifier declaration_specifiers             { $1.push($2); $1 }
// 	| type_qualifier                                    { vec![DeclarationSpecifier::Type($1)] }
// 	| type_qualifier declaration_specifiers             { $1.push($2); $1 }
// 	;
//
// init_declarator_list /* Vec<Declarator> */
// 	: init_declarator                           { vec![$1] }
//     | init_declarator_list ',' init_declarator  { $1.push($3); $1 }
// 	;
//
// init_declarator /* InitDeclaratorNode */
// 	: declarator { let s = self.span(); InitDeclaratorNode::new($1, None, s) }
// 	| declarator '=' initializer { les s = self.span(); InitDeclaratorNode::new($1, Some($3), s) }
// 	;
//
// storage_class_specifier /* Storage*/
// 	: TYPEDEF           { Storage::Typedef  }
// 	| EXTERN            { Storage::Extern }
// 	| STATIC            { Storage::Static }
// 	| AUTO              { Storage::Auto }
// 	| REGISTER          { Storage::Register }
//     ;
//
// type_qualifier  /* Qualifier */
// 	: CONST             { Qualifier::Const }
// 	| VOLATILE          { Qualifier::Volatile }
// 	;
//
// type_specifier /* TypeId */
// 	: VOID              { self.types().void() }
// 	| CHAR              { self.types().char() }
// 	| SHORT             { self.types().short() }
// 	| INT               { self.types().int() }
// 	| LONG              { self.types().long() }
// 	| FLOAT             { self.types().float() }
// 	| DOUBLE            { self.types().double() }
// 	| SIGNED            { self.types().signed() }
// 	| UNSIGNED          { self.types().unsigned() }
// 	| struct_specifier  { 42.into() }
// 	| union_specifier   { 42.into() }
// 	| enum_specifier    { 42.into() }
// 	| TYPE_NAME         { 42.into() }
// 	;
//
// initializer /* InitializerNode */
// 	: expression                        { let s = self.span(); InitializerNode::new(Initializer::Single($1), span) }
// 	| '{' initializer_list '}'          { let s = self.span(); InitializerNode::new(Initializer::List($2), span) }
// 	| '{' initializer_list ',' '}'      { let s = self.span(); InitializerNode::new(Initializer::List($2), span) }
// 	;
//
// initializer_list /* Vec<InitializerNode> */
// 	: initializer                       { vec![$1] }
//     | initializer_list ',' initializer  { $1.push($3); $1 }
// 	;

// struct_specifier /* StructId */
// 	: STRUCT IDENTIFIER '{' struct_declaration_list '}'     { self.lexer.ctx.arenas.structs($2, $4, false) }
// 	| STRUCT '{' struct_declaration_list '}'                { self.lexer.ctx.arenas.structs(None, $3, false) }
// 	| STRUCT IDENTIFIER                                     { self.lexer.ctx.arenas.structs($2, Vec::new(), false) }
// 	;

// union_specifier /* UnionId */
// 	: STRUCT IDENTIFIER '{' struct_declaration_list '}'     { self.lexer.ctx.arenas.structs($2, $4, false) }
// 	| STRUCT '{' struct_declaration_list '}'                { self.lexer.ctx.arenas.structs(None, $3, false) }
// 	| STRUCT IDENTIFIER                                     { self.lexer.ctx.arenas.structs($2, Vec::new(), false) }
// 	;
//
// struct_declaration_list /* Vec<Field> */
// 	: struct_declaration                                    { vec![$1] }
// 	| struct_declaration_list struct_declaration            { $1.extend($2); $1 }
// 	;
//
// struct_declaration /* vec<Field> */
// 	: specifier_qualifier_list struct_declarator_list ';'   { $2.map(|f| Field::new($1.clone(), f)).collect::<Vec<_>>() }
// 	;
//
// specifier_qualifier_list /* Vec<Qualifiers> */
// 	: type_specifier specifier_qualifier_list
// 	| type_specifier
// 	| type_qualifier specifier_qualifier_list
// 	| type_qualifier
// 	;
//
// struct_declarator_list /* Vec<TypeId> */
// 	: struct_declarator                                     { vec![$1] }
// 	| struct_declarator_list ',' struct_declarator          { $1.push($3); $1 }
// 	;
//
// struct_declarator /* TypeId */
// 	: declarator                                            { $1 }
// 	/* | ':' constant_expression                               {  } */
// 	/* | declarator ':' constant_expression                    {  } */
// 	;
//
// enum_specifier /* EnumId */
// 	: ENUM '{' enumerator_list '}'              {}
// 	| ENUM IDENTIFIER '{' enumerator_list '}'   {}
// 	| ENUM IDENTIFIER                           {}
// 	;
//
// enumerator_list /* Vec<VariantId> */
// 	: enumerator
// 	| enumerator_list ',' enumerator
// 	;
//
// enumerator /* VariantId */
// 	: IDENTIFIER
// 	| IDENTIFIER '=' constant_expression
// 	;
//

//
// declarator /* TypeId */
// 	: pointer direct_declarator                     { self.lexer.ctx.types.pointer($2) }
// 	| direct_declarator                             { $1 }
// 	;
//
//
// direct_declarator /* TypeId */
// 	: IDENTIFIER                                    { self.lexer.ctx.types.void($1) }
// 	| '(' declarator ')'                            { $2 }
// 	| direct_declarator '[' constant_expression ']' {  }
// 	| direct_declarator '[' ']'                     {  }
// 	| direct_declarator '(' parameter_type_list ')' {  }
// 	| direct_declarator '(' identifier_list ')'     {  }
// 	| direct_declarator '(' ')'                     {  }
// 	;
//
// pointer /* */
// 	: '*'
// 	| '*' type_qualifier_list
// 	| '*' pointer
// 	| '*' type_qualifier_list pointer
// 	;
//
// type_qualifier_list /* */
// 	: type_qualifier
// 	| type_qualifier_list type_qualifier
// 	;
//
//
// parameter_type_list /* */
// 	: parameter_list
// 	| parameter_list ',' ELLIPSIS
// 	;
//
// parameter_list /* */
// 	: parameter_declaration
// 	| parameter_list ',' parameter_declaration
// 	;
//
// parameter_declaration /* */
// 	: declaration_specifiers declarator
// 	| declaration_specifiers abstract_declarator
// 	| declaration_specifiers
// 	;
//
// identifier_list /* */
// 	: IDENTIFIER
// 	| identifier_list ',' IDENTIFIER
// 	;
//
// type_name /* */
// 	: specifier_qualifier_list
// 	| specifier_qualifier_list abstract_declarator
// 	;
//
// abstract_declarator /* */
// 	: pointer
// 	| direct_abstract_declarator
// 	| pointer direct_abstract_declarator
// 	;
//
// direct_abstract_declarator /* */
// 	: '(' abstract_declarator ')'
// 	| '[' ']'
// 	| '[' constant_expression ']'
// 	| direct_abstract_declarator '[' ']'
// 	| direct_abstract_declarator '[' constant_expression ']'
// 	| '(' ')'
// 	| '(' parameter_type_list ')'
// 	| direct_abstract_declarator '(' ')'
// 	| direct_abstract_declarator '(' parameter_type_list ')'
// 	;
//

//
// statement /* */
// 	: labeled_statement
// 	| compound_statement
// 	| expression_statement
// 	| selection_statement
// 	| iteration_statement
// 	| jump_statement
// 	;
//
// labeled_statement /* */
// 	: IDENTIFIER ':' statement
// 	| CASE constant_expression ':' statement
// 	| DEFAULT ':' statement
// 	;
//
// compound_statement /* */
// 	: '{' '}'
// 	| '{' statement_list '}'
// 	| '{' declaration_list '}'
// 	| '{' declaration_list statement_list '}'
// 	;
//
// declaration_list /* */
// 	: declaration
// 	| declaration_list declaration
// 	;
//
// statement_list /* */
// 	: statement
// 	| statement_list statement
// 	;
//
// expression_statement /* */
// 	: ';'
// 	| expression ';'
// 	;
//
// selection_statement /* */
// 	: IF '(' expression ')' statement
// 	| IF '(' expression ')' statement ELSE statement
// 	| SWITCH '(' expression ')' statement
// 	;
//
// iteration_statement /* */
// 	: WHILE '(' expression ')' statement
// 	| DO statement WHILE '(' expression ')' ';'
// 	| FOR '(' expression_statement expression_statement ')' statement
// 	| FOR '(' expression_statement expression_statement expression ')' statement
// 	;
//
// jump_statement /* */
// 	: GOTO IDENTIFIER ';'
// 	| CONTINUE ';'
// 	| BREAK ';'
// 	| RETURN ';'
// 	| RETURN expression ';'
// 	;
//
// translation_unit /* */
// 	: external_declaration
// 	| translation_unit external_declaration
// 	;
//
// external_declaration /* */
// 	: function_definition
// 	| declaration
// 	;
//
// function_definition /* */
// 	: declaration_specifiers declarator declaration_list compound_statement
// 	| declaration_specifiers declarator compound_statement
// 	| declarator declaration_list compound_statement
// 	| declarator compound_statement
// 	;
//
