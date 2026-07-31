%no_main

%{
use crate::context::ContextAccess;
use crate::ast::{Qualifier, TypeNode, ExpressionNode, Name, DeclarationSpecifier};
use crate::parser::YYLex;
use crate::error::yyerror;
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
%type<TypeNode> type_name
/*
%type<Storage> storage_class_specifier
%type<TypeId> type_specifier
%type<Qualifier> type_qualifier
%type<Vec<Initializer>> initializer initializer_list
%type<Initializer> init_declarator_list
%type<DeclarationSpecifier> declaration_specifiers
%type<Vec<Qualifier>> specifier_qualifier_list
%type<TypeId> type_specifier
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
    : expression { let s = self.span; self.expressions().constant_expression($1, s) }
    ;

expression /* ExpressionId */
    : '(' expression ')'                            { $2 }
    | IDENTIFIER                                    { let s = self.span; self.expressions().identifier($1, s) }
    | CONSTANT                                      { let s = self.span; self.expressions().constant($1, s)}
    | STRING_LITERAL                                { let s = self.span; self.expressions().string_literal($1, s) }
    | expression '[' expression ']'                 { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression '(' ')'                            { let s = self.span; self.expressions().function_call($1, None, s) }
    | expression '(' expression ')'                 { let s = self.span; self.expressions().function_call($1, Some($3), s) }
    | expression '.' IDENTIFIER                     { let s = self.span; self.expressions().access($1, $2, $3, s) }
    | expression PTR_OP IDENTIFIER                  { let s = self.span; self.expressions().access($1, $2, $3, s) }
    | SIZEOF '(' expression ')'                     { let s = self.span; self.expressions().sizeof_expr($3, s) }
    | SIZEOF '(' type_name ')'                      { let s = self.span; self.expressions().sizeof_type($3, s) }
    | '(' type_name ')' expression %prec PREC_UNARY { let s = self.span; self.expressions().cast($2, $4, s) }
    | expression INC_OP                             { let s = self.span; self.expressions().unary(YYToken::POST_INC_OP, $1, s) }
    | expression DEC_OP                             { let s = self.span; self.expressions().unary(YYToken::POST_DEC_OP, $1, s) }
    | INC_OP expression                             { let s = self.span; self.expressions().unary($1, $2, s) }
    | DEC_OP expression                             { let s = self.span; self.expressions().unary($1, $2, s) }
    | '&' expression %prec PREC_UNARY               { let s = self.span; self.expressions().unary($1, $2, s) }
    | '*' expression %prec PREC_UNARY               { let s = self.span; self.expressions().unary($1, $2, s) }
    | '+' expression %prec PREC_UNARY               { let s = self.span; self.expressions().unary($1, $2, s) }
    | '-' expression %prec PREC_UNARY               { let s = self.span; self.expressions().unary($1, $2, s) }
    | '~' expression                                { let s = self.span; self.expressions().unary($1, $2, s) }
    | '!' expression                                { let s = self.span; self.expressions().unary($1, $2, s) }
    | expression '+' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression '-' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression '*' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression '/' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression '%' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression LEFT_OP expression                 { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression RIGHT_OP expression                { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression '<' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression '>' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression LE_OP expression                   { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression GE_OP expression                   { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression EQ_OP expression                   { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression NE_OP expression                   { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression '&' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression '^' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression '|' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression AND_OP expression                  { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression OR_OP expression                   { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression '=' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression MUL_ASSIGN expression              { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression DIV_ASSIGN expression              { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression MOD_ASSIGN expression              { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression ADD_ASSIGN expression              { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression SUB_ASSIGN expression              { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression LEFT_ASSIGN expression             { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression RIGHT_ASSIGN expression            { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression AND_ASSIGN expression              { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression XOR_ASSIGN expression              { let s = self.span; self.expressions().binary($1, $2, $3, s) }
	| expression OR_ASSIGN expression               { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression ',' expression                     { let s = self.span; self.expressions().binary($1, $2, $3, s) }
    | expression '?' expression ':' expression      { let s = self.span; self.expressions().ternary($1, $3, $5, s) }
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
//
// init_declarator_list /* Vec<(NameId, Option<ExpressionId>)> */
// 	: init_declarator                           { vec![$1] }
//     | init_declarator_list ',' init_declarator  { $1.push($3); $1 }
// 	;
//
// init_declarator /* (NameId, Option<ExpressionId>) */
// 	: declarator { ($1, None) }
// 	| declarator '=' initializer { ($1, Some($3)) }
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
// type_name
//     :
// 	;
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
// initializer /* Vec<ExpressionId> */
// 	: expression                        { vec![$1] }
// 	| '{' initializer_list '}'          { $2 }
// 	| '{' initializer_list ',' '}'      { $2 }
// 	;
//
// initializer_list /* Vec<ExpressionId> */
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
