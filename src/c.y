%no_main

%{
use crate::symbol::{NameId};
use crate::types::{Qualifiers, TypeId};
use crate::tag::{EnumId, StructId, UnionId, Field};
use crate::lexer::YYLex;
use crate::error::yyerror;
%}

%start translation_unit

%token<String> IDENTIFIER STRING_LITERAL
%token CONSTANT  SIZEOF
%token PTR_OP INC_OP DEC_OP LEFT_OP RIGHT_OP LE_OP GE_OP EQ_OP NE_OP
%token AND_OP OR_OP MUL_ASSIGN DIV_ASSIGN MOD_ASSIGN ADD_ASSIGN
%token SUB_ASSIGN LEFT_ASSIGN RIGHT_ASSIGN AND_ASSIGN
%token XOR_ASSIGN OR_ASSIGN TYPE_NAME

%token TYPEDEF EXTERN STATIC AUTO REGISTER
%token CHAR SHORT INT LONG SIGNED UNSIGNED FLOAT DOUBLE CONST VOLATILE VOID
%token STRUCT UNION ENUM ELLIPSIS

%token CASE DEFAULT IF ELSE SWITCH WHILE DO FOR GOTO CONTINUE BREAK RETURN

%type<Qualifiers> type_qualifier
%type<Vec<Qualifiers>> specifier_qualifier_list
%type<TypeId> type_specifier
%type<Vec<Field>> struct_declaration_list
%type<Vec<Field>> struct_declaration
// %type<Field> specifier_qualifier_list
%type<Vec<TypeId>> struct_declarator_list
%type<NameId> struct_declarator
%type<StructId> struct_specifier
%type<TypeId> direct_declarator

%%

primary_expression /* */
	: IDENTIFIER            {}
	| CONSTANT              {}
	| STRING_LITERAL        {}
	| '(' expression ')'    {}
	;

postfix_expression /* */
	: primary_expression
	| expression '[' expression ']'
	| expression '(' ')'
	| expression '(' argument_expression_list ')'
	| expression '.' IDENTIFIER
	| expression PTR_OP IDENTIFIER
	| expression INC_OP
	| expression DEC_OP
	;

argument_expression_list /* */
	: assignment_expression
	| argument_expression_list ',' assignment_expression
	;

unary_expression /* */
	: postfix_expression
	| INC_OP unary_expression
	| DEC_OP unary_expression
	| unary_operator cast_expression
	| SIZEOF unary_expression
	| SIZEOF '(' type_name ')'
	;

unary_operator /* */
	: '&'
	| '*'
	| '+'
	| '-'
	| '~'
	| '!'
	;

cast_expression /* */
	: unary_expression
	| '(' type_name ')' cast_expression
	;

multiplicative_expression /* */
	: cast_expression
	| multiplicative_expression '*' cast_expression
	| multiplicative_expression '/' cast_expression
	| multiplicative_expression '%' cast_expression
	;

additive_expression /* */
	: multiplicative_expression
	| additive_expression '+' multiplicative_expression
	| additive_expression '-' multiplicative_expression
	;

shift_expression /* */
	: additive_expression
	| shift_expression LEFT_OP additive_expression
	| shift_expression RIGHT_OP additive_expression
	;

relational_expression /* */
	: shift_expression
	| relational_expression '<' shift_expression
	| relational_expression '>' shift_expression
	| relational_expression LE_OP shift_expression
	| relational_expression GE_OP shift_expression
	;

equality_expression /* */
	: relational_expression
	| equality_expression EQ_OP relational_expression
	| equality_expression NE_OP relational_expression
	;

and_expression /* */
	: equality_expression
	| and_expression '&' equality_expression
	;

exclusive_or_expression /* */
	: and_expression
	| exclusive_or_expression '^' and_expression
	;

inclusive_or_expression /* */
	: exclusive_or_expression
	| inclusive_or_expression '|' exclusive_or_expression
	;

logical_and_expression /* */
	: inclusive_or_expression
	| logical_and_expression AND_OP inclusive_or_expression
	;

logical_or_expression /* */
	: logical_and_expression
	| logical_or_expression OR_OP logical_and_expression
	;

conditional_expression /* */
	: logical_or_expression
	| logical_or_expression '?' expression ':' conditional_expression
	;

assignment_expression /* */
	: conditional_expression
	| unary_expression assignment_operator assignment_expression
	;

assignment_operator /* */
	: '='
	| MUL_ASSIGN
	| DIV_ASSIGN
	| MOD_ASSIGN
	| ADD_ASSIGN
	| SUB_ASSIGN
	| LEFT_ASSIGN
	| RIGHT_ASSIGN
	| AND_ASSIGN
	| XOR_ASSIGN
	| OR_ASSIGN
	;

expression /* */
	: assignment_expression
	| expression ',' assignment_expression
	;

constant_expression /* */
	: conditional_expression
	;

declaration /* */
	: declaration_specifiers ';'
	| declaration_specifiers init_declarator_list ';'
	;

declaration_specifiers /* */
	: storage_class_specifier
	| storage_class_specifier declaration_specifiers
	| type_specifier
	| type_specifier declaration_specifiers
	| type_qualifier
	| type_qualifier declaration_specifiers
	;

init_declarator_list /* */
	: init_declarator
	| init_declarator_list ',' init_declarator
	;

init_declarator /* */
	: declarator
	| declarator '=' initializer
	;

storage_class_specifier /* */
	: TYPEDEF
	| EXTERN
	| STATIC
	| AUTO
	| REGISTER
	;

type_specifier /* TypeId */
	: VOID              { self.lexer.ctx.arenas.types.void() }
	| CHAR              { self.lexer.ctx.arenas.types.char() }
	| SHORT             { self.lexer.ctx.arenas.types.short() }
	| INT               { self.lexer.ctx.arenas.types.int() }
	| LONG              { self.lexer.ctx.arenas.types.long() }
	| FLOAT             { self.lexer.ctx.arenas.types.float() }
	| DOUBLE            { self.lexer.ctx.arenas.types.double() }
	| SIGNED            { self.lexer.ctx.arenas.types.signed() }
	| UNSIGNED          { self.lexer.ctx.arenas.types.unsigned() }
	| struct_specifier  { $1 }
	| union_specifier   { self.lexer.ctx.arenas.types.char() }
	| enum_specifier    { self.lexer.ctx.arenas.types.char() }
	| TYPE_NAME         { self.lexer.ctx.arenas.types.char() }
	;

struct_specifier /* StructId */
	: STRUCT IDENTIFIER '{' struct_declaration_list '}'     { self.lexer.ctx.arenas.structs($2, $4, false) }
	| STRUCT '{' struct_declaration_list '}'                { self.lexer.ctx.arenas.structs(None, $3, false) }
	| STRUCT IDENTIFIER                                     { self.lexer.ctx.arenas.structs($2, Vec::new(), false) }
	;

union_specifier /* UnionId */
	: STRUCT IDENTIFIER '{' struct_declaration_list '}'     { self.lexer.ctx.arenas.structs($2, $4, false) }
	| STRUCT '{' struct_declaration_list '}'                { self.lexer.ctx.arenas.structs(None, $3, false) }
	| STRUCT IDENTIFIER                                     { self.lexer.ctx.arenas.structs($2, Vec::new(), false) }
	;

struct_declaration_list /* Vec<Field> */
	: struct_declaration                                    { vec![$1] }
	| struct_declaration_list struct_declaration            { $1.extend($2); $1 }
	;

struct_declaration /* vec<Field> */
	: specifier_qualifier_list struct_declarator_list ';'   { $2.map(|f| Field::new($1.clone(), f)).collect::<Vec<_>>() }
	;

specifier_qualifier_list /* Vec<Qualifiers> */
	: type_specifier specifier_qualifier_list
	| type_specifier
	| type_qualifier specifier_qualifier_list
	| type_qualifier
	;

struct_declarator_list /* Vec<TypeId> */
	: struct_declarator                                     { vec![$1] }
	| struct_declarator_list ',' struct_declarator          { $1.push($3); $1 }
	;

struct_declarator /* TypeId */
	: declarator                                            { $1 }
	/* | ':' constant_expression                               {  } */
	/* | declarator ':' constant_expression                    {  } */
	;

enum_specifier /* EnumId */
	: ENUM '{' enumerator_list '}'              {}
	| ENUM IDENTIFIER '{' enumerator_list '}'   {}
	| ENUM IDENTIFIER                           {}
	;

enumerator_list /* Vec<VariantId> */
	: enumerator
	| enumerator_list ',' enumerator
	;

enumerator /* VariantId */
	: IDENTIFIER
	| IDENTIFIER '=' constant_expression
	;

type_qualifier  /* Qualifiers */
	: CONST { Qualifiers::Const }
	| VOLATILE { Qualifiers::Volatile }
	;

declarator /* TypeId */
	: pointer direct_declarator                     { self.lexer.ctx.types.pointer($2) }
	| direct_declarator                             { $1 }
	;


direct_declarator /* TypeId */
	: IDENTIFIER                                    { self.lexer.ctx.types.void($1) }
	| '(' declarator ')'                            { $2 }
	| direct_declarator '[' constant_expression ']' {  }
	| direct_declarator '[' ']'                     {  }
	| direct_declarator '(' parameter_type_list ')' {  }
	| direct_declarator '(' identifier_list ')'     {  }
	| direct_declarator '(' ')'                     {  }
	;

pointer /* */
	: '*'
	| '*' type_qualifier_list
	| '*' pointer
	| '*' type_qualifier_list pointer
	;

type_qualifier_list /* */
	: type_qualifier
	| type_qualifier_list type_qualifier
	;


parameter_type_list /* */
	: parameter_list
	| parameter_list ',' ELLIPSIS
	;

parameter_list /* */
	: parameter_declaration
	| parameter_list ',' parameter_declaration
	;

parameter_declaration /* */
	: declaration_specifiers declarator
	| declaration_specifiers abstract_declarator
	| declaration_specifiers
	;

identifier_list /* */
	: IDENTIFIER
	| identifier_list ',' IDENTIFIER
	;

type_name /* */
	: specifier_qualifier_list
	| specifier_qualifier_list abstract_declarator
	;

abstract_declarator /* */
	: pointer
	| direct_abstract_declarator
	| pointer direct_abstract_declarator
	;

direct_abstract_declarator /* */
	: '(' abstract_declarator ')'
	| '[' ']'
	| '[' constant_expression ']'
	| direct_abstract_declarator '[' ']'
	| direct_abstract_declarator '[' constant_expression ']'
	| '(' ')'
	| '(' parameter_type_list ')'
	| direct_abstract_declarator '(' ')'
	| direct_abstract_declarator '(' parameter_type_list ')'
	;

initializer /* */
	: assignment_expression
	| '{' initializer_list '}'
	| '{' initializer_list ',' '}'
	;

initializer_list /* */
	: initializer
	| initializer_list ',' initializer
	;

statement /* */
	: labeled_statement
	| compound_statement
	| expression_statement
	| selection_statement
	| iteration_statement
	| jump_statement
	;

labeled_statement /* */
	: IDENTIFIER ':' statement
	| CASE constant_expression ':' statement
	| DEFAULT ':' statement
	;

compound_statement /* */
	: '{' '}'
	| '{' statement_list '}'
	| '{' declaration_list '}'
	| '{' declaration_list statement_list '}'
	;

declaration_list /* */
	: declaration
	| declaration_list declaration
	;

statement_list /* */
	: statement
	| statement_list statement
	;

expression_statement /* */
	: ';'
	| expression ';'
	;

selection_statement /* */
	: IF '(' expression ')' statement
	| IF '(' expression ')' statement ELSE statement
	| SWITCH '(' expression ')' statement
	;

iteration_statement /* */
	: WHILE '(' expression ')' statement
	| DO statement WHILE '(' expression ')' ';'
	| FOR '(' expression_statement expression_statement ')' statement
	| FOR '(' expression_statement expression_statement expression ')' statement
	;

jump_statement /* */
	: GOTO IDENTIFIER ';'
	| CONTINUE ';'
	| BREAK ';'
	| RETURN ';'
	| RETURN expression ';'
	;

translation_unit /* */
	: external_declaration
	| translation_unit external_declaration
	;

external_declaration /* */
	: function_definition
	| declaration
	;

function_definition /* */
	: declaration_specifiers declarator declaration_list compound_statement
	| declaration_specifiers declarator compound_statement
	| declarator declaration_list compound_statement
	| declarator compound_statement
	;

%%
