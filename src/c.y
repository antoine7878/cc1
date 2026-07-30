%no_main

%{
use crate::context::ContextAccess;
use crate::ast::NodeId;
use crate::symbol::NameId;
use crate::types::{Qualifiers, TypeId};
use crate::tag::{EnumId, StructId, UnionId, Field};
use crate::lexer::YYLex;
use crate::error::yyerror;
%}

%token<NameId> IDENTIFIER STRING_LITERAL CONSTANT
%token TYPEDEF EXTERN STATIC AUTO REGISTER
%token CHAR SHORT INT LONG SIGNED UNSIGNED FLOAT DOUBLE CONST VOLATILE VOID
%token STRUCT UNION ENUM ELLIPSIS
%token 

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

%type<NodeId> expression constant_expression
%type<TypeId> type_name

%%

constant_expression /* NodeId */
    : expression { self.nodes().constant_expression($1) }
    ;

expression /* NodeId */
    : IDENTIFIER                                    { self.nodes().identifier($1) }
    | CONSTANT                                      { self.nodes().constant($1)}
    | STRING_LITERAL                                { self.nodes().string_literal($1) }
    | '(' expression ')'                            { $2 }
    | expression '[' expression ']'                 { self.nodes().binary($1, $2, $3) }
    | expression '(' ')'                            { self.nodes().function_call($1, None) }
    | expression '(' expression ')'                 { self.nodes().function_call($1, Some($3)) }
    | expression '.' IDENTIFIER                     { self.nodes().access($1, $2, $3) }
    | expression PTR_OP IDENTIFIER                  { self.nodes().access($1, $2, $3) }
    | SIZEOF '(' expression ')'                     { self.nodes().sizeof_expr($3) }
    | SIZEOF '(' type_name ')'                      { self.nodes().sizeof_type($3) }
    | '(' type_name ')' expression %prec PREC_UNARY { self.nodes().cast($2, $4) }

    | expression INC_OP                             { self.nodes().unary(YYToken::POST_INC_OP, $1) }
    | expression DEC_OP                             { self.nodes().unary(YYToken::POST_DEC_OP, $1) }
    | INC_OP expression                             { self.nodes().unary($1, $2) }
    | DEC_OP expression                             { self.nodes().unary($1, $2) }
    | '&' expression %prec PREC_UNARY               { self.nodes().unary($1, $2) }
    | '*' expression %prec PREC_UNARY               { self.nodes().unary($1, $2) }
    | '+' expression %prec PREC_UNARY               { self.nodes().unary($1, $2) }
    | '-' expression %prec PREC_UNARY               { self.nodes().unary($1, $2) }
    | '~' expression                                { self.nodes().unary($1, $2) }
    | '!' expression                                { self.nodes().unary($1, $2) }

    | expression '+' expression                     { self.nodes().binary($1, $2, $3) }
    | expression '-' expression                     { self.nodes().binary($1, $2, $3) }
    | expression '*' expression                     { self.nodes().binary($1, $2, $3) }
    | expression '/' expression                     { self.nodes().binary($1, $2, $3) }
    | expression '%' expression                     { self.nodes().binary($1, $2, $3) }
    | expression LEFT_OP expression                 { self.nodes().binary($1, $2, $3) }
    | expression RIGHT_OP expression                { self.nodes().binary($1, $2, $3) }
    | expression '<' expression                     { self.nodes().binary($1, $2, $3) }
    | expression '>' expression                     { self.nodes().binary($1, $2, $3) }
    | expression LE_OP expression                   { self.nodes().binary($1, $2, $3) }
    | expression GE_OP expression                   { self.nodes().binary($1, $2, $3) }
	| expression EQ_OP expression                   { self.nodes().binary($1, $2, $3) }
	| expression NE_OP expression                   { self.nodes().binary($1, $2, $3) }
	| expression '&' expression                     { self.nodes().binary($1, $2, $3) }
	| expression '^' expression                     { self.nodes().binary($1, $2, $3) }
	| expression '|' expression                     { self.nodes().binary($1, $2, $3) }
	| expression AND_OP expression                  { self.nodes().binary($1, $2, $3) }
	| expression OR_OP expression                   { self.nodes().binary($1, $2, $3) }
	| expression '=' expression                     { self.nodes().binary($1, $2, $3) }
	| expression MUL_ASSIGN expression              { self.nodes().binary($1, $2, $3) }
	| expression DIV_ASSIGN expression              { self.nodes().binary($1, $2, $3) }
	| expression MOD_ASSIGN expression              { self.nodes().binary($1, $2, $3) }
	| expression ADD_ASSIGN expression              { self.nodes().binary($1, $2, $3) }
	| expression SUB_ASSIGN expression              { self.nodes().binary($1, $2, $3) }
	| expression LEFT_ASSIGN expression             { self.nodes().binary($1, $2, $3) }
	| expression RIGHT_ASSIGN expression            { self.nodes().binary($1, $2, $3) }
	| expression AND_ASSIGN expression              { self.nodes().binary($1, $2, $3) }
	| expression XOR_ASSIGN expression              { self.nodes().binary($1, $2, $3) }
	| expression OR_ASSIGN expression               { self.nodes().binary($1, $2, $3) }
    | expression ',' expression                     { self.nodes().binary($1, $2, $3) }

    | expression '?' expression ':' expression      { self.nodes().ternary($1, $3, $5) }
    ;

type_name
    :
	;

%%


/*
%type<Qualifiers> type_qualifier
%type<Vec<Qualifiers>> specifier_qualifier_list
%type<TypeId> type_specifier
%type<Vec<Field>> struct_declaration_list
%type<Vec<Field>> struct_declaration
%type<Vec<TypeId>> struct_declarator_list
%type<NameId> struct_declarator
%type<StructId> struct_specifier
*/

// declaration /* */
// 	: declaration_specifiers ';'
// 	| declaration_specifiers init_declarator_list ';'
// 	;
//
// declaration_specifiers /* */
// 	: storage_class_specifier
// 	| storage_class_specifier declaration_specifiers
// 	| type_specifier
// 	| type_specifier declaration_specifiers
// 	| type_qualifier
// 	| type_qualifier declaration_specifiers
// 	;
//
// init_declarator_list /* */
// 	: init_declarator
// 	| init_declarator_list ',' init_declarator
// 	;
//
// init_declarator /* */
// 	: declarator
// 	| declarator '=' initializer
// 	;
//
// storage_class_specifier /* */
// 	: TYPEDEF
// 	| EXTERN
// 	| STATIC
// 	| AUTO
// 	| REGISTER
// 	;
//
// type_specifier /* TypeId */
// 	: VOID              { self.lexer.ctx.arenas.types.void() }
// 	| CHAR              { self.lexer.ctx.arenas.types.char() }
// 	| SHORT             { self.lexer.ctx.arenas.types.short() }
// 	| INT               { self.lexer.ctx.arenas.types.int() }
// 	| LONG              { self.lexer.ctx.arenas.types.long() }
// 	| FLOAT             { self.lexer.ctx.arenas.types.float() }
// 	| DOUBLE            { self.lexer.ctx.arenas.types.double() }
// 	| SIGNED            { self.lexer.ctx.arenas.types.signed() }
// 	| UNSIGNED          { self.lexer.ctx.arenas.types.unsigned() }
// 	| struct_specifier  { $1 }
// 	| union_specifier   { self.lexer.ctx.arenas.types.char() }
// 	| enum_specifier    { self.lexer.ctx.arenas.types.char() }
// 	| TYPE_NAME         { self.lexer.ctx.arenas.types.char() }
// 	;
//
// struct_specifier /* StructId */
// 	: STRUCT IDENTIFIER '{' struct_declaration_list '}'     { self.lexer.ctx.arenas.structs($2, $4, false) }
// 	| STRUCT '{' struct_declaration_list '}'                { self.lexer.ctx.arenas.structs(None, $3, false) }
// 	| STRUCT IDENTIFIER                                     { self.lexer.ctx.arenas.structs($2, Vec::new(), false) }
// 	;
//
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
// type_qualifier  /* Qualifiers */
// 	: CONST { Qualifiers::Const }
// 	| VOLATILE { Qualifiers::Volatile }
// 	;
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
// initializer /* */
// 	: assignment_expression
// 	| '{' initializer_list '}'
// 	| '{' initializer_list ',' '}'
// 	;
//
// initializer_list /* */
// 	: initializer
// 	| initializer_list ',' initializer
// 	;
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
