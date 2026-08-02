#![allow(unused_braces, mixed_script_confusables, unused)]
use crate::parser::Span;
use std::fmt;
use std::io::Read;

use crate::ast::{Qualifier, Type, ExpressionNode, Name, DeclarationSpecifier, Initializer, TypeSpecifier, ParameterDeclaration};
use crate::ast::{DeclarationNode, InitDeclaratorNode, DeclaratorNode, InitializerNode, Storage, FunctionParametersNode};
use crate::parser::YYLex;
use crate::error::yyerror;

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


#[allow(non_camel_case_types, mixed_script_confusables)]
            #[derive(Debug, Clone, PartialEq)]
            pub enum YYToken {
                Empty,
yyeof,
error,
accept,
IDENTIFIER(Name),
STRING_LITERAL(Name),
CONSTANT(Name),
TYPE_NAME(Name),
TYPEDEF,
EXTERN,
STATIC,
AUTO,
REGISTER,
CHAR,
SHORT,
INT,
LONG,
SIGNED,
UNSIGNED,
FLOAT,
DOUBLE,
CONST,
VOLATILE,
VOID,
STRUCT,
UNION,
ENUM,
ELLIPSIS,
CASE,
DEFAULT,
IF,
ELSE,
SWITCH,
WHILE,
DO,
FOR,
GOTO,
CONTINUE,
BREAK,
RETURN,
SUB_ASSIGN,
LEFT_ASSIGN,
RIGHT_ASSIGN,
AND_ASSIGN,
MUL_ASSIGN,
DIV_ASSIGN,
MOD_ASSIGN,
ADD_ASSIGN,
XOR_ASSIGN,
OR_ASSIGN,
OR_OP,
AND_OP,
EQ_OP,
NE_OP,
LE_OP,
GE_OP,
LEFT_OP,
RIGHT_OP,
INC_OP,
DEC_OP,
POST_INC_OP,
POST_DEC_OP,
SIZEOF,
PREC_UNARY,
PTR_OP,
expression(ExpressionNode),
constant_expression(ExpressionNode),
type_name(Type),
declaration(DeclarationNode),
type_specifier(TypeSpecifier),
storage_class_specifier(Storage),
type_qualifier(Qualifier),
type_qualifier_list(Vec<Qualifier>),
declaration_specifiers(Vec<DeclarationSpecifier>),
declarator_list(InitDeclaratorNode),
init_declarator(InitDeclaratorNode),
init_declarator_list(Vec<InitDeclaratorNode>),
initializer(InitializerNode),
initializer_list(Vec<InitializerNode>),
declarator(DeclaratorNode),
direct_declarator(DeclaratorNode),
pointer(DeclaratorNode),
direct_abstract_declarator(DeclaratorNode),
abstract_declarator(DeclaratorNode),
identifier_list(Vec<Name>),
parameter_type_list(FunctionParametersNode),
parameter_list(Vec<ParameterDeclaration>),
parameter_declaration(ParameterDeclaration),
specifier_qualifier_list(Vec<DeclarationSpecifier>),
unit,
decalration,
struct_specifier,
union_specifier,
enum_specifier,
Char(char),
}
impl YYToken {
                fn index(&self) -> usize {
                    match self {
YYToken::yyeof => 0,
YYToken::error => 1,
YYToken::accept => 2,
YYToken::IDENTIFIER(_) => 3,
YYToken::STRING_LITERAL(_) => 4,
YYToken::CONSTANT(_) => 5,
YYToken::TYPE_NAME(_) => 6,
YYToken::TYPEDEF => 7,
YYToken::EXTERN => 8,
YYToken::STATIC => 9,
YYToken::AUTO => 10,
YYToken::REGISTER => 11,
YYToken::CHAR => 12,
YYToken::SHORT => 13,
YYToken::INT => 14,
YYToken::LONG => 15,
YYToken::SIGNED => 16,
YYToken::UNSIGNED => 17,
YYToken::FLOAT => 18,
YYToken::DOUBLE => 19,
YYToken::CONST => 20,
YYToken::VOLATILE => 21,
YYToken::VOID => 22,
YYToken::STRUCT => 23,
YYToken::UNION => 24,
YYToken::ENUM => 25,
YYToken::ELLIPSIS => 26,
YYToken::CASE => 27,
YYToken::DEFAULT => 28,
YYToken::IF => 29,
YYToken::ELSE => 30,
YYToken::SWITCH => 31,
YYToken::WHILE => 32,
YYToken::DO => 33,
YYToken::FOR => 34,
YYToken::GOTO => 35,
YYToken::CONTINUE => 36,
YYToken::BREAK => 37,
YYToken::RETURN => 38,
YYToken::Char (',') => 39,
YYToken::Char ('=') => 40,
YYToken::SUB_ASSIGN => 41,
YYToken::LEFT_ASSIGN => 42,
YYToken::RIGHT_ASSIGN => 43,
YYToken::AND_ASSIGN => 44,
YYToken::MUL_ASSIGN => 45,
YYToken::DIV_ASSIGN => 46,
YYToken::MOD_ASSIGN => 47,
YYToken::ADD_ASSIGN => 48,
YYToken::XOR_ASSIGN => 49,
YYToken::OR_ASSIGN => 50,
YYToken::Char ('?') => 51,
YYToken::Char (':') => 52,
YYToken::OR_OP => 53,
YYToken::AND_OP => 54,
YYToken::Char ('|') => 55,
YYToken::Char ('^') => 56,
YYToken::Char ('&') => 57,
YYToken::EQ_OP => 58,
YYToken::NE_OP => 59,
YYToken::Char ('<') => 60,
YYToken::Char ('>') => 61,
YYToken::LE_OP => 62,
YYToken::GE_OP => 63,
YYToken::LEFT_OP => 64,
YYToken::RIGHT_OP => 65,
YYToken::Char ('+') => 66,
YYToken::Char ('-') => 67,
YYToken::Char ('*') => 68,
YYToken::Char ('/') => 69,
YYToken::Char ('%') => 70,
YYToken::Char ('!') => 71,
YYToken::Char ('~') => 72,
YYToken::INC_OP => 73,
YYToken::DEC_OP => 74,
YYToken::POST_INC_OP => 75,
YYToken::POST_DEC_OP => 76,
YYToken::SIZEOF => 77,
YYToken::PREC_UNARY => 78,
YYToken::Char ('(') => 79,
YYToken::Char ('[') => 80,
YYToken::Char ('.') => 81,
YYToken::PTR_OP => 82,
YYToken::expression(_) => 83,
YYToken::constant_expression(_) => 84,
YYToken::type_name(_) => 85,
YYToken::declaration(_) => 86,
YYToken::type_specifier(_) => 87,
YYToken::storage_class_specifier(_) => 88,
YYToken::type_qualifier(_) => 89,
YYToken::type_qualifier_list(_) => 90,
YYToken::declaration_specifiers(_) => 91,
YYToken::declarator_list(_) => 92,
YYToken::init_declarator(_) => 93,
YYToken::init_declarator_list(_) => 94,
YYToken::initializer(_) => 95,
YYToken::initializer_list(_) => 96,
YYToken::declarator(_) => 97,
YYToken::direct_declarator(_) => 98,
YYToken::pointer(_) => 99,
YYToken::direct_abstract_declarator(_) => 100,
YYToken::abstract_declarator(_) => 101,
YYToken::identifier_list(_) => 102,
YYToken::parameter_type_list(_) => 103,
YYToken::parameter_list(_) => 104,
YYToken::parameter_declaration(_) => 105,
YYToken::specifier_qualifier_list(_) => 106,
YYToken::unit => 107,
YYToken::decalration => 108,
YYToken::Char (')') => 109,
YYToken::Char (']') => 110,
YYToken::Char (';') => 111,
YYToken::struct_specifier => 112,
YYToken::union_specifier => 113,
YYToken::enum_specifier => 114,
YYToken::Char ('{') => 115,
YYToken::Char ('}') => 116,
_ => unreachable!()
                    }
                }
            }
#[allow(non_snake_case, mixed_script_confusables)]
impl YYToken {
fn into_IDENTIFIER (self) -> Name { match self {
                        YYToken::IDENTIFIER(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_STRING_LITERAL (self) -> Name { match self {
                        YYToken::STRING_LITERAL(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_CONSTANT (self) -> Name { match self {
                        YYToken::CONSTANT(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_TYPE_NAME (self) -> Name { match self {
                        YYToken::TYPE_NAME(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_expression (self) -> ExpressionNode { match self {
                        YYToken::expression(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_constant_expression (self) -> ExpressionNode { match self {
                        YYToken::constant_expression(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_type_name (self) -> Type { match self {
                        YYToken::type_name(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_declaration (self) -> DeclarationNode { match self {
                        YYToken::declaration(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_type_specifier (self) -> TypeSpecifier { match self {
                        YYToken::type_specifier(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_storage_class_specifier (self) -> Storage { match self {
                        YYToken::storage_class_specifier(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_type_qualifier (self) -> Qualifier { match self {
                        YYToken::type_qualifier(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_type_qualifier_list (self) -> Vec<Qualifier> { match self {
                        YYToken::type_qualifier_list(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_declaration_specifiers (self) -> Vec<DeclarationSpecifier> { match self {
                        YYToken::declaration_specifiers(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_declarator_list (self) -> InitDeclaratorNode { match self {
                        YYToken::declarator_list(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_init_declarator (self) -> InitDeclaratorNode { match self {
                        YYToken::init_declarator(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_init_declarator_list (self) -> Vec<InitDeclaratorNode> { match self {
                        YYToken::init_declarator_list(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_initializer (self) -> InitializerNode { match self {
                        YYToken::initializer(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_initializer_list (self) -> Vec<InitializerNode> { match self {
                        YYToken::initializer_list(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_declarator (self) -> DeclaratorNode { match self {
                        YYToken::declarator(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_direct_declarator (self) -> DeclaratorNode { match self {
                        YYToken::direct_declarator(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_pointer (self) -> DeclaratorNode { match self {
                        YYToken::pointer(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_direct_abstract_declarator (self) -> DeclaratorNode { match self {
                        YYToken::direct_abstract_declarator(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_abstract_declarator (self) -> DeclaratorNode { match self {
                        YYToken::abstract_declarator(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_identifier_list (self) -> Vec<Name> { match self {
                        YYToken::identifier_list(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_parameter_type_list (self) -> FunctionParametersNode { match self {
                        YYToken::parameter_type_list(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_parameter_list (self) -> Vec<ParameterDeclaration> { match self {
                        YYToken::parameter_list(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_parameter_declaration (self) -> ParameterDeclaration { match self {
                        YYToken::parameter_declaration(s) => s,
                        _ => panic!("wrong type")
                    }
                }

fn into_specifier_qualifier_list (self) -> Vec<DeclarationSpecifier> { match self {
                        YYToken::specifier_qualifier_list(s) => s,
                        _ => panic!("wrong type")
                    }
                }

}

pub struct Yacc<R: Read> {
    state_stack: Vec<usize>,
    value_stack: Vec<YYToken>,
    span_stack: Vec<Span>,
    lookahead: Option<YYToken>,
    lookahead_id: usize,
    lookahead_span: Span,
    act: isize,
    continue_parse: bool,
    is_recovering: bool,
    ret: i32,
    token_since_error: usize,
    pub span: Span,
    pub yydebug: bool,
    pub lexer: YYLex<R>,
}


impl<R: Read> Yacc<R> {

const YY_ERROR_TOKEN_ID: usize = 1;
const YY_EOF_TOKEN_ID: usize = 0;
const YY_GOTO_TABLE: [[isize; 117]; 3] =[[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-2,-3,0,0,0,0,0,0,0,0,],
[1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
];
const YY_RLEN_TABLE: [usize; 135] = [1,1,1,3,1,1,1,4,3,4,3,3,4,4,4,2,2,2,2,2,2,2,2,2,2,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,5,2,3,1,2,1,2,1,2,1,3,1,3,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,3,4,1,3,2,1,1,3,4,3,4,4,3,1,2,2,3,1,2,1,3,1,3,2,2,1,1,3,1,2,2,1,2,1,1,1,2,3,2,3,3,4,2,3,3,4,];
const YY_PRODUCT_TABLE: [usize; 135] = [2,107,84,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,86,86,91,91,91,91,91,91,94,94,93,93,88,88,88,88,88,89,89,87,87,87,87,87,87,87,87,87,87,87,87,87,95,95,95,96,96,97,97,98,98,98,98,98,98,98,99,99,99,99,90,90,103,103,104,104,105,105,105,102,102,85,85,106,106,106,106,101,101,101,100,100,100,100,100,100,100,100,100,];
const YY_ACTION_TABLE: [isize; 135] = [-1,0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,-1,-1,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,];
const YY_DEFAULT_ACT: [isize; 3] = [0,0,2,];
const YY_DEFAULT_REDUCE_ACT: [isize; 3] = [0,0,2,];
const YY_PRODUCTION_LINE: [usize; 135] = [0,97,101,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154,155,156,157,161,162,166,167,168,169,170,171,175,176,180,181,185,186,187,188,189,193,194,198,199,200,201,202,203,204,205,206,207,208,209,210,214,215,216,220,221,225,226,230,231,232,233,234,235,236,240,241,242,243,247,248,252,253,257,258,263,264,265,269,270,274,275,279,280,281,282,286,287,288,292,293,294,295,296,297,298,299,300,];
const YY_TERMINAL_TABLE: [bool; 117] = [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,true,false,false,false,true,true,true,false,false,];

    pub fn new(lexer: YYLex<R>) -> Self {
        Self {
            state_stack: Vec::new(),
            value_stack: Vec::new(),
            span_stack: Vec::new(),
            span: Span::default(),
            lookahead: None,
            lookahead_id: 0,
            lookahead_span: Span::default(),
            yydebug: false,
            act: 0,
            continue_parse: true,
            is_recovering: false,
            ret: 0,
            lexer,
            token_since_error: 0,
        }
    }

    fn next_act(&mut self) -> isize {
        let &state_id = self.state_stack.last().unwrap();
        let mut dflt = Self::YY_DEFAULT_ACT[state_id];
        if dflt != 0 {
            return dflt;
        }
        if self.lookahead.is_none() {
            self.read_token();
        }
        dflt = Self::YY_GOTO_TABLE[state_id][self.lookahead_id];
        if dflt != 0 {
            return dflt;
        }
        Self::YY_DEFAULT_REDUCE_ACT[state_id]
    }
    pub fn yyparse(&mut self) -> i32 {

        self.push_statcks(0, YYToken::yyeof, Span::default());
        self.read_token();

        while self.continue_parse {
            let state_id = self.state_stack.last().unwrap();
            self.act = self.next_act();
            match self.act {
                1 => break,
                a if a > 0 => self.reduce(),
                a if a < 0 => self.shift(),
                _ => self.error(),
            }
        }
        self.ret
    }

    fn error(&mut self) {
        if !self.is_recovering {
            self.unwind()
        }
        if matches!(self.lookahead, Some(YYToken::yyeof)) {
            self.yyabort();
        }
    }

    fn unwind(&mut self) {
        yyerror("syntax error", self);
        let start = self.span_stack.last().cloned().unwrap_or_default().start;
        while let Some(i) = self.state_stack.last()
            && Self::YY_GOTO_TABLE[*i][Self::YY_ERROR_TOKEN_ID] == 0
        {
            self.state_stack.pop();
            self.span_stack.pop();
            let val = self.value_stack.pop().unwrap();
        }
        let end = self.span_stack.last().cloned().unwrap_or_default().end;

        let Some(i) = self.state_stack.last() else {
            self.yyabort();
            return;
        };
        self.is_recovering = true;
        let act = Self::YY_GOTO_TABLE[*i][Self::YY_ERROR_TOKEN_ID] + 1;


        self.push_statcks((-act) as usize, YYToken::error, Span::new(start, end));
        self.token_since_error = 0;

        let top = (-act) as usize;
        self.act = Self::YY_GOTO_TABLE[top][self.lookahead_id];
        while self.act == 0 {
            self.yyclearin();
            self.read_token();
            if matches!(self.lookahead, Some(YYToken::yyeof)) {
                return;
            }
            self.act = Self::YY_GOTO_TABLE[top][self.lookahead_id];
        }
    }

    pub fn yyerrok(&mut self) {
        self.is_recovering = false;
    }

    pub fn yyaccept(&mut self) {
        self.continue_parse = false;
        self.ret = 0;
    }

    pub fn yyabort(&mut self) {
        self.continue_parse = false;
        self.ret = 3;
    }

    pub fn yyrecovering(&mut self) -> bool {
        self.is_recovering
    }

    pub fn yyclearin(&mut self) {
        self.lookahead = None;
    }

    /* /* DEBUGGING */ */
    /* fn token_class(token: &Option<YYToken>) -> &'static str { */
    /*     if token == &Some(YYToken::Empty) { */
    /*         return "nterm"; */
    /*     } */
    /*     let id = token.as_ref().unwrap().index(); */
    /*     if Self::YY_TERMINAL_TABLE[id] { "nterm" } else { "token" } */
    /* } */
    /* /* DEBUGGING */ */

    fn push_statcks(&mut self, state: usize, value: YYToken, span: Span) {
        self.state_stack.push(state);
        self.value_stack.push(value);
        self.span_stack.push(span);
    }

    fn read_token(&mut self) {
        let c = self.lexer.yylex();
        self.lookahead_id = c.index();
        self.lookahead_span = self.lexer.span;
        self.lookahead = Some(c);
    }

    fn shift(&mut self) {
        self.act += 1;

        if self.token_since_error >= 2 {
            self.is_recovering = false;
        } else {
            self.token_since_error += 1;
        }

        if self.lookahead.is_none() {
            self.read_token();
        }

        let token = self.lookahead.take().unwrap_or(YYToken::yyeof);
        self.push_statcks(-self.act as usize, token, self.lexer.span);
    }

    fn reduce(&mut self) {
        self.act -= 1;

        let len = Self::YY_RLEN_TABLE[self.act as usize];
        let product = Self::YY_PRODUCT_TABLE[self.act as usize];

        let yyval = self.action();
        if yyval == YYToken::error {
            self.error();
            return;
        }

        let &top = self.state_stack.last().unwrap();
        self.push_statcks(-(Self::YY_GOTO_TABLE[top][product] + 1) as usize, yyval, self.span);
    }

    #[allow(unused_braces, clippy::let_and_return)]
    fn action(&mut self) -> YYToken {
        let len = self.value_stack.len();
        let idx = len - Self::YY_RLEN_TABLE[self.act as usize];

        self.span.start = self.span_stack[idx].start;
        self.span.end = self.span_stack[len - 1].end;


        let ret = self.do_action(idx);
        self.value_stack.truncate(idx);
        self.state_stack.truncate(idx);
        self.span_stack.truncate(idx);

        ret
    }

    fn do_action(&mut self, idx: usize) -> YYToken {
        match Self::YY_ACTION_TABLE[self.act as usize] {
0 => {
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty);
{ self.lexer.ctx.print_ast(&__yy1 ); YYToken::unit }}
1 => YYToken::constant_expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
{ node_span!(self, expressions, constant_expression, __yy1 ) }}),
2 => YYToken::expression({
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_expression();
{ __yy2 }}),
3 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_IDENTIFIER();
{ node_span!(self, expressions, identifier, __yy1 ) }}),
4 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_CONSTANT();
{ node_span!(self, expressions, constant,__yy1 )}}),
5 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_STRING_LITERAL();
{ node_span!(self, expressions, string_literal,__yy1 ) }}),
6 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
7 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
{ node_span!(self, expressions, function_call, __yy1 , None) }}),
8 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, function_call, __yy1 , Some(__yy3 )) }}),
9 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_IDENTIFIER();
{ node_span!(self, expressions, access, __yy1 , __yy2 , __yy3 ) }}),
10 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_IDENTIFIER();
{ node_span!(self, expressions, access, __yy1 , __yy2 , __yy3 ) }}),
11 => YYToken::expression({
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, sizeof_expr, __yy3 ) }}),
12 => YYToken::expression({
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_type_name();
{ node_span!(self, expressions, sizeof_type, __yy3 ) }}),
13 => YYToken::expression({
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_type_name();
let  __yy4 = std::mem::replace(&mut self.value_stack[idx + 3], YYToken::Empty).into_expression();
{ node_span!(self, expressions, cast, __yy2 , __yy4 ) }}),
14 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
{ node_span!(self, expressions, unary, YYToken::POST_INC_OP, __yy1 ) }}),
15 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
{ node_span!(self, expressions, unary, YYToken::POST_DEC_OP, __yy1 ) }}),
16 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty);
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_expression();
{ node_span!(self, expressions, unary, __yy1 , __yy2 ) }}),
17 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty);
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_expression();
{ node_span!(self, expressions, unary, __yy1 , __yy2 ) }}),
18 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty);
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_expression();
{ node_span!(self, expressions, unary, __yy1 , __yy2 ) }}),
19 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty);
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_expression();
{ node_span!(self, expressions, unary, __yy1 , __yy2 ) }}),
20 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty);
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_expression();
{ node_span!(self, expressions, unary, __yy1 , __yy2 ) }}),
21 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty);
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_expression();
{ node_span!(self, expressions, unary, __yy1 , __yy2 ) }}),
22 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty);
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_expression();
{ node_span!(self, expressions, unary, __yy1 , __yy2 ) }}),
23 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty);
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_expression();
{ node_span!(self, expressions, unary, __yy1 , __yy2 ) }}),
24 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
25 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
26 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
27 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
28 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
29 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
30 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
31 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
32 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
33 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
34 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
35 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
36 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
37 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
38 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
39 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
40 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
41 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
42 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
43 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
44 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
45 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
46 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
47 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
48 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
49 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
50 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
51 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
52 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
53 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
{ node_span!(self, expressions, binary, __yy1 , __yy2 , __yy3 ) }}),
54 => YYToken::expression({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_expression();
let  __yy5 = std::mem::replace(&mut self.value_stack[idx + 4], YYToken::Empty).into_expression();
{ node_span!(self, expressions, ternary, __yy1 , __yy3 , __yy5 ) }}),
55 => YYToken::declaration({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_declaration_specifiers();
{ with_span!(self, DeclarationNode::new, __yy1 , vec![]) }}),
56 => YYToken::declaration({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_declaration_specifiers();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_init_declarator_list();
{ with_span!(self, DeclarationNode::new, __yy1 , __yy2 ) }}),
57 => YYToken::declaration_specifiers({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_storage_class_specifier();
{ vec![DeclarationSpecifier::Storage(__yy1 )] }}),
58 => YYToken::declaration_specifiers({
let mut __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_declaration_specifiers();
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_storage_class_specifier();
{ push!(__yy2 , DeclarationSpecifier::Storage(__yy1 )) }}),
59 => YYToken::declaration_specifiers({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_type_specifier();
{ vec![DeclarationSpecifier::Type(__yy1 )] }}),
60 => YYToken::declaration_specifiers({
let mut __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_declaration_specifiers();
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_type_specifier();
{ push!(__yy2 , DeclarationSpecifier::Type(__yy1 )) }}),
61 => YYToken::declaration_specifiers({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_type_qualifier();
{ vec![DeclarationSpecifier::Qualifier(__yy1 )] }}),
62 => YYToken::declaration_specifiers({
let mut __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_declaration_specifiers();
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_type_qualifier();
{ push!(__yy2 , DeclarationSpecifier::Qualifier(__yy1 )) }}),
63 => YYToken::init_declarator_list({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_init_declarator();
{ vec![__yy1 ] }}),
64 => YYToken::init_declarator_list({
let mut __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_init_declarator_list();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_init_declarator();
{ push!(__yy1 , __yy3 ) }}),
65 => YYToken::init_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_declarator();
{ with_span!(self, InitDeclaratorNode::new, __yy1 , None) }}),
66 => YYToken::init_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_declarator();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_initializer();
{ with_span!(self, InitDeclaratorNode::new, __yy1 , Some(__yy3 )) }}),
67 => YYToken::storage_class_specifier({
{ Storage::Typedef  }}),
68 => YYToken::storage_class_specifier({
{ Storage::Extern }}),
69 => YYToken::storage_class_specifier({
{ Storage::Static }}),
70 => YYToken::storage_class_specifier({
{ Storage::Auto }}),
71 => YYToken::storage_class_specifier({
{ Storage::Register }}),
72 => YYToken::type_qualifier({
{ Qualifier::Const }}),
73 => YYToken::type_qualifier({
{ Qualifier::Volatile }}),
74 => YYToken::type_specifier({
{ TypeSpecifier::Void }}),
75 => YYToken::type_specifier({
{ TypeSpecifier::Char }}),
76 => YYToken::type_specifier({
{ TypeSpecifier::Short }}),
77 => YYToken::type_specifier({
{ TypeSpecifier::Int }}),
78 => YYToken::type_specifier({
{ TypeSpecifier::Long }}),
79 => YYToken::type_specifier({
{ TypeSpecifier::Float }}),
80 => YYToken::type_specifier({
{ TypeSpecifier::Double }}),
81 => YYToken::type_specifier({
{ TypeSpecifier::Signed }}),
82 => YYToken::type_specifier({
{ TypeSpecifier::Unsigned }}),
83 => YYToken::type_specifier({
{ TypeSpecifier::Struct(42.into()) }}),
84 => YYToken::type_specifier({
{ TypeSpecifier::Union(42.into()) }}),
85 => YYToken::type_specifier({
{ TypeSpecifier::Enum(42.into()) }}),
86 => YYToken::type_specifier({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_TYPE_NAME();
{ TypeSpecifier::TypedefName(__yy1 ) }}),
87 => YYToken::initializer({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_expression();
{ with_span!(self, InitializerNode::new, Initializer::Single(__yy1 )) }}),
88 => YYToken::initializer({
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_initializer_list();
{ with_span!(self, InitializerNode::new, Initializer::List(__yy2 )) }}),
89 => YYToken::initializer({
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_initializer_list();
{ with_span!(self, InitializerNode::new, Initializer::List(__yy2 )) }}),
90 => YYToken::initializer_list({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_initializer();
{ vec![__yy1 ] }}),
91 => YYToken::initializer_list({
let mut __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_initializer_list();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_initializer();
{ push!(__yy1 , __yy3 ) }}),
92 => YYToken::declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_pointer();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_direct_declarator();
{ node_span!(self, declarators, with_pointer, __yy1 , __yy2 ) }}),
93 => YYToken::declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_direct_declarator();
{ __yy1 }}),
94 => YYToken::direct_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_IDENTIFIER();
{ node_span!(self, declarators, ident, __yy1 ) }}),
95 => YYToken::direct_declarator({
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_declarator();
{ __yy2 }}),
96 => YYToken::direct_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_direct_declarator();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_constant_expression();
{ node_span!(self, declarators, array, __yy1 , Some(__yy3 )) }}),
97 => YYToken::direct_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_direct_declarator();
{ node_span!(self, declarators, array, __yy1 , None) }}),
98 => YYToken::direct_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_direct_declarator();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_parameter_type_list();
{ node_span!(self, declarators, function, __yy1 , __yy3 ) }}),
99 => YYToken::direct_declarator({
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_identifier_list();
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_direct_declarator();
{ let a = with_span!(self, FunctionParametersNode::old_style, __yy3 ); node_span!(self, declarators, function, __yy1 , a) }}),
100 => YYToken::direct_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_direct_declarator();
{ let a = with_span!(self, FunctionParametersNode::empty); node_span!(self, declarators, function, __yy1 , a) }}),
101 => YYToken::pointer({
{ node_span!(self, declarators, pointer, vec![], None)     }}),
102 => YYToken::pointer({
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_type_qualifier_list();
{ node_span!(self, declarators, pointer, __yy2 ,     None)     }}),
103 => YYToken::pointer({
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_pointer();
{ node_span!(self, declarators, pointer, vec![], Some(__yy2 )) }}),
104 => YYToken::pointer({
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_type_qualifier_list();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_pointer();
{ node_span!(self, declarators, pointer, __yy2 ,     Some(__yy3 )) }}),
105 => YYToken::type_qualifier_list({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_type_qualifier();
{ vec![__yy1 ] }}),
106 => YYToken::type_qualifier_list({
let mut __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_type_qualifier_list();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_type_qualifier();
{ push!(__yy1 , __yy2 ) }}),
107 => YYToken::parameter_type_list({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_parameter_list();
{ with_span!(self, FunctionParametersNode::param_style, __yy1 ) }}),
108 => YYToken::parameter_type_list({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_parameter_list();
{ with_span!(self, FunctionParametersNode::variadic, __yy1 ) }}),
109 => YYToken::parameter_list({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_parameter_declaration();
{ vec![__yy1 ] }}),
110 => YYToken::parameter_list({
let mut __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_parameter_list();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_parameter_declaration();
{ push!(__yy1 , __yy3 ) }}),
111 => YYToken::parameter_declaration({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_declaration_specifiers();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_declarator();
{ with_span!(self, ParameterDeclaration::new, __yy1 , __yy2 ) }}),
112 => YYToken::parameter_declaration({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_declaration_specifiers();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_abstract_declarator();
{ with_span!(self, ParameterDeclaration::new, __yy1 , __yy2 ) }}),
113 => YYToken::parameter_declaration({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_declaration_specifiers();
{ let a = node_span!(self, declarators, abstrct); with_span!(self, ParameterDeclaration::new, __yy1 , a) }}),
114 => YYToken::identifier_list({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_IDENTIFIER();
{ vec![__yy1 ] }}),
115 => YYToken::identifier_list({
let mut __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_identifier_list();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_IDENTIFIER();
{ push!(__yy1 , __yy3 ) }}),
116 => YYToken::specifier_qualifier_list({
let mut __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_specifier_qualifier_list();
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_type_specifier();
{ push!(__yy2 , DeclarationSpecifier::Type(__yy1 )) }}),
117 => YYToken::specifier_qualifier_list({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_type_specifier();
{ vec![DeclarationSpecifier::Type(__yy1 )] }}),
118 => YYToken::specifier_qualifier_list({
let mut __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_specifier_qualifier_list();
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_type_qualifier();
{ push!(__yy2 , DeclarationSpecifier::Qualifier(__yy1 )) }}),
119 => YYToken::specifier_qualifier_list({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_type_qualifier();
{ vec![DeclarationSpecifier::Qualifier(__yy1 )] }}),
120 => YYToken::abstract_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_pointer();
{ let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, with_pointer, __yy1 , a) }}),
121 => YYToken::abstract_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_direct_abstract_declarator();
{ __yy1 }}),
122 => YYToken::abstract_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_pointer();
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_direct_abstract_declarator();
{ node_span!(self, declarators, with_pointer, __yy1 , __yy2 ) }}),
123 => YYToken::direct_abstract_declarator({
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_abstract_declarator();
{ __yy2 }}),
124 => YYToken::direct_abstract_declarator({
{ let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, array, a, None) }}),
125 => YYToken::direct_abstract_declarator({
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_constant_expression();
{ let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, array, a, Some(__yy2 )) }}),
126 => YYToken::direct_abstract_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_direct_abstract_declarator();
{ node_span!(self, declarators, array, __yy1 , None) }}),
127 => YYToken::direct_abstract_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_direct_abstract_declarator();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_constant_expression();
{ node_span!(self, declarators, array, __yy1 , Some(__yy3 )) }}),
128 => YYToken::direct_abstract_declarator({
{ let a = node_span!(self, declarators, abstrct); let b = with_span!(self, FunctionParametersNode::empty); node_span!(self, declarators, function, a, b) }}),
129 => YYToken::direct_abstract_declarator({
let  __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty).into_parameter_type_list();
{ let a = node_span!(self, declarators, abstrct); node_span!(self, declarators, function, a, __yy2 ) }}),
130 => YYToken::direct_abstract_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_direct_abstract_declarator();
{ let a = with_span!(self, FunctionParametersNode::empty); node_span!(self, declarators, function, __yy1 , a) }}),
131 => YYToken::direct_abstract_declarator({
let  __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_direct_abstract_declarator();
let  __yy3 = std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty).into_parameter_type_list();
{ node_span!(self, declarators, function, __yy1 , __yy3 ) }}),
            -1 => std::mem::replace(&mut self.value_stack[idx], YYToken::Empty),
            _ => unreachable!(),
        }
    }
}



// struct_specifier /* StructId */
// 	: STRUCT IDENTIFIER '{' struct_declaration_list '}'     { self.lexer.ctx.arenas.structs($2, $4, false) } | STRUCT '{' struct_declaration_list '}'                { self.lexer.ctx.arenas.structs(None, $3, false) }
// 	| STRUCT IDENTIFIER                                     { self.lexer.ctx.arenas.structs($2, Vec::new(), false) }
// 	;

// union_specifier /* UnionId */
// 	: STRUCT IDENTIFIER '{' struct_declaration_list '}'     { self.lexer.ctx.arenas.structs($2, $4, false) }
// 	| STRUCT '{' struct_declaration_list '}'                { self.lexer.ctx.arenas.structs(None, $3, false) }
// 	| STRUCT IDENTIFIER                                     { self.lexer.ctx.arenas.structs($2, Vec::new(), false) }
// 	;
//
// struct_declaration_list /*  */
// 	: struct_declaration                                    { vec![$1] }
// 	| struct_declaration_list struct_declaration            { $1.extend($2); $1 }
// 	;
//
// struct_declaration /*  */
// 	: specifier_qualifier_list struct_declarator_list ';'   {  }
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
