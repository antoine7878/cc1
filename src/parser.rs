#![allow(unused_braces, mixed_script_confusables, unused)]
use std::fmt;
use std::io::Read;

use crate::context::ContextAccess;
use crate::ast::NodeId;
use crate::symbol::NameId;
use crate::types::{Qualifiers, TypeId};
use crate::tag::{EnumId, StructId, UnionId, Field};
use crate::lexer::YYLex;
use crate::error::yyerror;

#[allow(non_camel_case_types, mixed_script_confusables)]
            #[derive(Debug, Clone, PartialEq)]
            pub enum YYToken {
                Empty,
yyeof,
error,
accept,
IDENTIFIER(NameId),
STRING_LITERAL(NameId),
CONSTANT(NameId),
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
POST_INC_OP,
POST_DEC_OP,
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
TYPE_NAME,
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
SIZEOF,
PREC_UNARY,
PTR_OP,
expression(NodeId),
constant_expression(NodeId),
type_name(TypeId),
unary_op,
binary_op,
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
YYToken::TYPEDEF => 6,
YYToken::EXTERN => 7,
YYToken::STATIC => 8,
YYToken::AUTO => 9,
YYToken::REGISTER => 10,
YYToken::CHAR => 11,
YYToken::SHORT => 12,
YYToken::INT => 13,
YYToken::LONG => 14,
YYToken::SIGNED => 15,
YYToken::UNSIGNED => 16,
YYToken::FLOAT => 17,
YYToken::DOUBLE => 18,
YYToken::CONST => 19,
YYToken::VOLATILE => 20,
YYToken::VOID => 21,
YYToken::STRUCT => 22,
YYToken::UNION => 23,
YYToken::ENUM => 24,
YYToken::ELLIPSIS => 25,
YYToken::POST_INC_OP => 26,
YYToken::POST_DEC_OP => 27,
YYToken::CASE => 28,
YYToken::DEFAULT => 29,
YYToken::IF => 30,
YYToken::ELSE => 31,
YYToken::SWITCH => 32,
YYToken::WHILE => 33,
YYToken::DO => 34,
YYToken::FOR => 35,
YYToken::GOTO => 36,
YYToken::CONTINUE => 37,
YYToken::BREAK => 38,
YYToken::RETURN => 39,
YYToken::Char (',') => 40,
YYToken::Char ('=') => 41,
YYToken::SUB_ASSIGN => 42,
YYToken::LEFT_ASSIGN => 43,
YYToken::RIGHT_ASSIGN => 44,
YYToken::AND_ASSIGN => 45,
YYToken::MUL_ASSIGN => 46,
YYToken::DIV_ASSIGN => 47,
YYToken::MOD_ASSIGN => 48,
YYToken::ADD_ASSIGN => 49,
YYToken::XOR_ASSIGN => 50,
YYToken::OR_ASSIGN => 51,
YYToken::TYPE_NAME => 52,
YYToken::Char ('?') => 53,
YYToken::Char (':') => 54,
YYToken::OR_OP => 55,
YYToken::AND_OP => 56,
YYToken::Char ('|') => 57,
YYToken::Char ('^') => 58,
YYToken::Char ('&') => 59,
YYToken::EQ_OP => 60,
YYToken::NE_OP => 61,
YYToken::Char ('<') => 62,
YYToken::Char ('>') => 63,
YYToken::LE_OP => 64,
YYToken::GE_OP => 65,
YYToken::LEFT_OP => 66,
YYToken::RIGHT_OP => 67,
YYToken::Char ('+') => 68,
YYToken::Char ('-') => 69,
YYToken::Char ('*') => 70,
YYToken::Char ('/') => 71,
YYToken::Char ('%') => 72,
YYToken::Char ('!') => 73,
YYToken::Char ('~') => 74,
YYToken::INC_OP => 75,
YYToken::DEC_OP => 76,
YYToken::SIZEOF => 77,
YYToken::PREC_UNARY => 78,
YYToken::Char ('(') => 79,
YYToken::Char ('[') => 80,
YYToken::Char ('.') => 81,
YYToken::PTR_OP => 82,
YYToken::expression(_) => 83,
YYToken::constant_expression(_) => 84,
YYToken::type_name(_) => 85,
YYToken::Char (')') => 86,
YYToken::Char (']') => 87,
YYToken::unary_op => 88,
YYToken::binary_op => 89,
_ => unreachable!()
                    }
                }
            }
impl From<YYToken> for NameId {
                fn from(token: YYToken) -> NameId {
                    match token {
YYToken::IDENTIFIER(s)|YYToken::STRING_LITERAL(s)|YYToken::CONSTANT(s) => s,
                    _ => panic!("wrong type"),
                    }
                }
            }
impl From<YYToken> for NodeId {
                fn from(token: YYToken) -> NodeId {
                    match token {
YYToken::expression(s)|YYToken::constant_expression(s) => s,
                    _ => panic!("wrong type"),
                    }
                }
            }
impl From<YYToken> for TypeId {
                fn from(token: YYToken) -> TypeId {
                    match token {
YYToken::type_name(s) => s,
                    _ => panic!("wrong type"),
                    }
                }
            }

pub struct Yacc<R: Read> {
    state_stack: Vec<usize>,
    value_stack: Vec<YYToken>,
    lookahead: Option<YYToken>,
    lookahead_id: usize,
    act: isize,
    pub yydebug: bool,
    continue_parse: bool,
    is_recovering: bool,
    pub lexer: YYLex<R>,
    ret: i32,
    token_since_error: usize,
}

/* DEBUGGING */
macro_rules! yylog {
    ($self:expr, $($arg:tt)*) => {
        if $self.yydebug {
            eprintln!($($arg)*);
        }
    };
}
/* DEBUGGING */

impl<R: Read> Yacc<R> {

const YY_ERROR_TOKEN_ID: usize = 1;
const YY_EOF_TOKEN_ID: usize = 0;
const YY_GOTO_TABLE: [[isize; 90]; 77] =[[0,0,0,-2,-3,-4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-5,0,0,0,0,0,0,0,0,-6,-7,-8,0,0,-9,-10,-11,-12,-13,0,-14,0,0,0,-15,-16,0,0,0,-17,0,],
[3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,3,3,3,3,3,3,3,3,3,3,3,0,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,0,0,3,3,0,0,3,3,3,3,0,0,0,3,3,0,0,],
[5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5,5,5,5,5,5,5,5,5,5,5,5,0,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,0,0,5,5,0,0,5,5,5,5,0,0,0,5,5,0,0,],
[4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,4,4,4,4,4,4,4,4,4,4,4,0,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,0,0,4,4,0,0,4,4,4,4,0,0,0,4,4,0,0,],
[0,0,0,22,22,22,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,22,0,0,0,0,0,0,0,0,22,22,22,0,0,22,22,22,22,22,0,22,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,24,24,24,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,24,0,0,0,0,0,0,0,0,24,24,24,0,0,24,24,24,24,24,0,24,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,25,25,25,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,25,0,0,0,0,0,0,0,0,25,25,25,0,0,25,25,25,25,25,0,25,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,23,23,23,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,23,0,0,0,0,0,0,0,0,23,23,23,0,0,23,23,23,23,23,0,23,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,27,27,27,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,27,0,0,0,0,0,0,0,0,27,27,27,0,0,27,27,27,27,27,0,27,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,26,26,26,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,26,0,0,0,0,0,0,0,0,26,26,26,0,0,26,26,26,26,26,0,26,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,20,20,20,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,20,0,0,0,0,0,0,0,0,20,20,20,0,0,20,20,20,20,20,0,20,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,21,21,21,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,21,0,0,0,0,0,0,0,0,21,21,21,0,0,21,21,21,21,21,0,21,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-18,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,-2,-3,-4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-5,0,0,0,0,0,0,0,0,-6,-7,-8,0,0,-9,-10,-11,-12,-13,0,-14,0,0,0,-19,0,-20,58,0,-17,0,],
[2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-21,-22,-23,-24,-25,-26,-27,-28,-29,-30,-31,-32,0,-33,0,-34,-35,-36,-37,-38,-39,-40,-41,-42,-43,-44,-45,-46,-47,-48,-49,-50,-51,0,0,-52,-53,0,0,-54,-55,-56,-57,0,0,0,0,0,0,-58,],
[1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,-2,-3,-4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-5,0,0,0,0,0,0,0,0,-6,-7,-8,0,0,-9,-10,-11,-12,-13,0,-14,0,0,0,-59,0,0,0,0,-17,0,],
[0,0,0,-2,-3,-4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-5,0,0,0,0,0,0,0,0,-6,-7,-8,0,0,-9,-10,-11,-12,-13,0,-14,0,0,0,-60,0,-61,58,0,-17,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-21,-22,-23,-24,-25,-26,-27,-28,-29,-30,-31,-32,0,-33,0,-34,-35,-36,-37,-38,-39,-40,-41,-42,-43,-44,-45,-46,-47,-48,-49,-50,-51,0,0,-52,-53,0,0,-54,-55,-56,-57,0,0,0,-62,0,0,-58,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-63,0,0,0,],
[0,0,0,57,57,57,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,57,0,0,0,0,0,0,0,0,57,57,57,0,0,57,57,57,57,57,0,57,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,46,46,46,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,46,0,0,0,0,0,0,0,0,46,46,46,0,0,46,46,46,46,46,0,46,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,51,51,51,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,51,0,0,0,0,0,0,0,0,51,51,51,0,0,51,51,51,51,51,0,51,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,52,52,52,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,52,0,0,0,0,0,0,0,0,52,52,52,0,0,52,52,52,52,52,0,52,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,53,53,53,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,53,0,0,0,0,0,0,0,0,53,53,53,0,0,53,53,53,53,53,0,53,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,54,54,54,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,54,0,0,0,0,0,0,0,0,54,54,54,0,0,54,54,54,54,54,0,54,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,47,47,47,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,47,0,0,0,0,0,0,0,0,47,47,47,0,0,47,47,47,47,47,0,47,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,48,48,48,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,48,0,0,0,0,0,0,0,0,48,48,48,0,0,48,48,48,48,48,0,48,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,49,49,49,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,49,0,0,0,0,0,0,0,0,49,49,49,0,0,49,49,49,49,49,0,49,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,50,50,50,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,50,0,0,0,0,0,0,0,0,50,50,50,0,0,50,50,50,50,50,0,50,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,55,55,55,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,55,0,0,0,0,0,0,0,0,55,55,55,0,0,55,55,55,55,55,0,55,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,56,56,56,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,56,0,0,0,0,0,0,0,0,56,56,56,0,0,56,56,56,56,56,0,56,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,-2,-3,-4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-5,0,0,0,0,0,0,0,0,-6,-7,-8,0,0,-9,-10,-11,-12,-13,0,-14,0,0,0,-64,0,0,0,0,-17,0,],
[0,0,0,45,45,45,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,45,0,0,0,0,0,0,0,0,45,45,45,0,0,45,45,45,45,45,0,45,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,44,44,44,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,44,0,0,0,0,0,0,0,0,44,44,44,0,0,44,44,44,44,44,0,44,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,43,43,43,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,43,0,0,0,0,0,0,0,0,43,43,43,0,0,43,43,43,43,43,0,43,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,42,42,42,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,42,0,0,0,0,0,0,0,0,42,42,42,0,0,42,42,42,42,42,0,42,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,41,41,41,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,41,0,0,0,0,0,0,0,0,41,41,41,0,0,41,41,41,41,41,0,41,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,39,39,39,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,39,0,0,0,0,0,0,0,0,39,39,39,0,0,39,39,39,39,39,0,39,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,40,40,40,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,40,0,0,0,0,0,0,0,0,40,40,40,0,0,40,40,40,40,40,0,40,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,35,35,35,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,35,0,0,0,0,0,0,0,0,35,35,35,0,0,35,35,35,35,35,0,35,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,36,36,36,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,36,0,0,0,0,0,0,0,0,36,36,36,0,0,36,36,36,36,36,0,36,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,37,37,37,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,37,0,0,0,0,0,0,0,0,37,37,37,0,0,37,37,37,37,37,0,37,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,38,38,38,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,38,0,0,0,0,0,0,0,0,38,38,38,0,0,38,38,38,38,38,0,38,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,33,33,33,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,33,0,0,0,0,0,0,0,0,33,33,33,0,0,33,33,33,33,33,0,33,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,34,34,34,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,34,0,0,0,0,0,0,0,0,34,34,34,0,0,34,34,34,34,34,0,34,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,28,28,28,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,28,0,0,0,0,0,0,0,0,28,28,28,0,0,28,28,28,28,28,0,28,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,29,29,29,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,29,0,0,0,0,0,0,0,0,29,29,29,0,0,29,29,29,29,29,0,29,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,30,30,30,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,30,0,0,0,0,0,0,0,0,30,30,30,0,0,30,30,30,30,30,0,30,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,31,31,31,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,31,0,0,0,0,0,0,0,0,31,31,31,0,0,31,31,31,31,31,0,31,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,32,32,32,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,32,0,0,0,0,0,0,0,0,32,32,32,0,0,32,32,32,32,32,0,32,0,0,0,0,0,0,0,0,0,0,],
[15,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,15,15,15,15,15,15,15,15,15,15,15,15,0,15,15,15,15,15,15,15,15,15,15,15,15,15,15,15,15,15,15,15,15,0,0,15,15,0,0,15,15,15,15,0,0,0,15,15,0,0,],
[16,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,16,16,16,16,16,16,16,16,16,16,16,16,0,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,0,0,16,16,0,0,16,16,16,16,0,0,0,16,16,0,0,],
[0,0,0,-2,-3,-4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-5,0,0,0,0,0,0,0,0,-6,-7,-8,0,0,-9,-10,-11,-12,-13,0,-14,0,0,0,-65,0,0,-66,0,-17,0,],
[0,0,0,-2,-3,-4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-5,0,0,0,0,0,0,0,0,-6,-7,-8,0,0,-9,-10,-11,-12,-13,0,-14,0,0,0,-67,0,0,0,0,-17,0,],
[0,0,0,-68,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,-69,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,-2,-3,-4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-5,0,0,0,0,0,0,0,0,-6,-7,-8,0,0,-9,-10,-11,-12,-13,0,-14,0,0,0,-70,0,0,0,0,-17,0,],
[17,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,17,17,17,17,17,17,17,17,17,17,17,17,0,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,17,0,0,-52,-53,0,0,-54,-55,-56,-57,0,0,0,17,17,0,-58,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-21,-22,-23,-24,-25,-26,-27,-28,-29,-30,-31,-32,0,-33,0,-34,-35,-36,-37,-38,-39,-40,-41,-42,-43,-44,-45,-46,-47,-48,-49,-50,-51,0,0,-52,-53,0,0,-54,-55,-56,-57,0,0,0,-71,0,0,-58,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-72,0,0,0,],
[6,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,6,6,6,6,6,6,6,6,6,6,6,6,0,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,0,0,6,6,0,0,6,6,6,6,0,0,0,6,6,0,0,],
[0,0,0,-2,-3,-4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-5,0,0,0,0,0,0,0,0,-6,-7,-8,0,0,-9,-10,-11,-12,-13,0,-14,0,0,0,-73,0,0,0,0,-17,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-21,-22,-23,-24,-25,-26,-27,-28,-29,-30,-31,-32,0,-33,-74,-34,-35,-36,-37,-38,-39,-40,-41,-42,-43,-44,-45,-46,-47,-48,-49,-50,-51,0,0,-52,-53,0,0,-54,-55,-56,-57,0,0,0,0,0,0,-58,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-21,-22,-23,-24,-25,-26,-27,-28,-29,-30,-31,-32,0,-33,0,-34,-35,-36,-37,-38,-39,-40,-41,-42,-43,-44,-45,-46,-47,-48,-49,-50,-51,0,0,-52,-53,0,0,-54,-55,-56,-57,0,0,0,-75,0,0,-58,],
[8,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,8,8,8,8,8,8,8,8,8,8,8,8,0,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,8,0,0,8,8,0,0,8,8,8,8,0,0,0,8,8,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-21,-22,-23,-24,-25,-26,-27,-28,-29,-30,-31,-32,0,-33,0,-34,-35,-36,-37,-38,-39,-40,-41,-42,-43,-44,-45,-46,-47,-48,-49,-50,-51,0,0,-52,-53,0,0,-54,-55,-56,-57,0,0,0,0,-76,0,-58,],
[10,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,10,10,10,10,10,10,10,10,10,10,10,10,0,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,10,0,0,10,10,0,0,10,10,10,10,0,0,0,10,10,0,0,],
[11,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,11,11,11,11,11,11,11,11,11,11,11,11,0,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,0,0,11,11,0,0,11,11,11,11,0,0,0,11,11,0,0,],
[18,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-21,-22,-23,-24,-25,-26,-27,-28,-29,-30,-31,-32,0,-33,18,-34,-35,-36,-37,-38,-39,-40,-41,-42,-43,-44,-45,-46,-47,-48,-49,-50,-51,0,0,-52,-53,0,0,-54,-55,-56,-57,0,0,0,18,18,0,-58,],
[12,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,12,12,12,12,12,12,12,12,12,12,12,12,0,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,0,0,12,12,0,0,12,12,12,12,0,0,0,12,12,0,0,],
[13,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,13,13,13,13,13,13,13,13,13,13,13,13,0,13,13,13,13,13,13,13,13,13,13,13,13,13,13,13,13,13,13,13,13,0,0,13,13,0,0,13,13,13,13,0,0,0,13,13,0,0,],
[14,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,14,14,14,14,14,14,14,14,14,14,14,14,0,14,14,14,14,14,14,14,14,14,14,14,14,14,14,14,14,14,14,14,14,0,0,-52,-53,0,0,-54,-55,-56,-57,0,0,0,14,14,0,-58,],
[0,0,0,-2,-3,-4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,-5,0,0,0,0,0,0,0,0,-6,-7,-8,0,0,-9,-10,-11,-12,-13,0,-14,0,0,0,-77,0,0,0,0,-17,0,],
[9,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,9,9,9,9,9,9,9,9,9,9,9,9,0,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,0,0,9,9,0,0,9,9,9,9,0,0,0,9,9,0,0,],
[7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7,7,7,7,7,7,7,7,7,7,7,7,0,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,0,0,7,7,0,0,7,7,7,7,0,0,0,7,7,0,0,],
[19,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,19,19,19,19,19,19,19,19,19,19,19,19,0,-33,19,-34,-35,-36,-37,-38,-39,-40,-41,-42,-43,-44,-45,-46,-47,-48,-49,-50,-51,0,0,-52,-53,0,0,-54,-55,-56,-57,0,0,0,19,19,0,-58,],
];
const YY_RLEN_TABLE: [usize; 58] = [1,1,1,1,1,3,4,3,4,3,3,4,4,4,2,2,2,3,5,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,0,];
const YY_PRODUCT_TABLE: [usize; 58] = [2,84,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,83,88,88,88,88,88,88,88,88,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,89,85,];
const YY_ACTION_TABLE: [isize; 58] = [-1,0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,];
const YY_DEFAULT_ACT: [isize; 77] = [0,3,5,4,22,24,25,23,27,26,20,21,0,0,0,0,0,0,0,0,57,46,51,52,53,54,47,48,49,50,55,56,0,45,44,43,42,41,39,40,35,36,37,38,33,34,28,29,30,31,32,15,16,0,0,0,0,0,0,0,0,6,0,0,0,8,0,10,11,0,12,13,0,0,9,7,0,];
const YY_DEFAULT_REDUCE_ACT: [isize; 77] = [0,3,5,4,22,24,25,23,27,26,20,21,0,58,2,0,0,58,0,0,57,46,51,52,53,54,47,48,49,50,55,56,0,45,44,43,42,41,39,40,35,36,37,38,33,34,28,29,30,31,32,15,16,0,0,0,0,0,17,0,0,6,0,0,0,8,0,10,11,18,12,13,14,0,9,7,19,];
const YY_PRODUCTION_LINE: [usize; 58] = [0,47,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,71,72,73,74,75,76,77,78,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,115,];
const YY_TERMINAL_TABLE: [bool; 90] = [false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,true,true,true,false,false,true,true,];

    pub fn new(lexer: YYLex<R>) -> Self {
        Self {
            state_stack: Vec::new(),
            value_stack: Vec::new(),
            lookahead: None,
            lookahead_id: 0,
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
        /* DEBUGGING */
        yylog!(self, "Starting parse");
        /* DEBUGGING */

        self.push_statcks(0, YYToken::yyeof);
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
        /* DEBUGGING */
        for token in self.value_stack[1..].iter().rev() {
            yylog!(self, "Cleanup: popping {:?}", token);
        }
        /* DEBUGGING */
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
        while let Some(i) = self.state_stack.last()
            && Self::YY_GOTO_TABLE[*i][Self::YY_ERROR_TOKEN_ID] == 0
        {
            self.state_stack.pop();
            let val = self.value_stack.pop().unwrap();
            /* DEBUGGING */
            yylog!(self, "Error: popping {:?}", val);
            yylog!(self, "Stack now {:?}", self.state_stack);
            /* DEBUGGING */
        }

        let Some(i) = self.state_stack.last() else {
            self.yyabort();
            return;
        };
        self.is_recovering = true;
        let act = Self::YY_GOTO_TABLE[*i][Self::YY_ERROR_TOKEN_ID] + 1;
        /* DEBUGGING */
        yylog!(self, "Shifting token {:?}", YYToken::error);
        /* DEBUGGING */
        self.push_statcks((-act) as usize, YYToken::error);
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

    fn push_statcks(&mut self, state: usize, value: YYToken) {
        self.state_stack.push(state);
        self.value_stack.push(value);
        /* DEBUGGING */
        yylog!(self, "Entering state {}", state);
        yylog!(self, "Stack now {:?}", self.state_stack)
        /* DEBUGGING */
    }

    fn read_token(&mut self) {
        let c = self.lexer.yylex();
        self.lookahead_id = c.index();
        self.lookahead = Some(c);
        /* DEBUGGING */
        yylog!(self, "Reading a token");
        match &self.lookahead {
            None => yylog!(self, "Now at end of input."),
            Some(l) => yylog!(self, "Next token is token {:?}", l),
        }
        /* DEBUGGING */
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

        /* DEBUGGING */
        yylog!(
            self,
            "Shifting token {:?}",
            self.lookahead.as_ref().unwrap_or(&YYToken::yyeof)
        );
        /* DEBUGGING */
        let token = self.lookahead.take().unwrap_or(YYToken::yyeof);
        self.push_statcks(-self.act as usize, token);
    }

    fn reduce(&mut self) {
        self.act -= 1;

        /* DEBUGGING */
        yylog!(
            self,
            "Reducing stack by rule {} (line {}):",
            self.act,
            Self::YY_PRODUCTION_LINE[self.act as usize]
        );
        /* DEBUGGING */
        let len = Self::YY_RLEN_TABLE[self.act as usize];
        let product = Self::YY_PRODUCT_TABLE[self.act as usize];

        let yyval = self.action();
        if yyval == YYToken::error {
            self.error();
            return;
        }

        let &top = self.state_stack.last().unwrap();
        self.push_statcks(-(Self::YY_GOTO_TABLE[top][product] + 1) as usize, yyval);
    }

    #[allow(unused_braces, clippy::let_and_return)]
    fn action(&mut self) -> YYToken {
        let idx = self.value_stack.len() - Self::YY_RLEN_TABLE[self.act as usize];

        /* DEBUGGING */
        for (i, tok) in self.value_stack[idx..].iter().enumerate().rev() {
            yylog!(self, "   ${} = {:?}", i + 1, tok);
        }
        /* DEBUGGING */

        let ret = self.do_action(idx);
        self.value_stack.truncate(idx);
        self.state_stack.truncate(idx);

        /* DEBUGGING */
        yylog!(self, "-> $$ = {:?}", ret);
        /* DEBUGGING */
        ret
    }

    fn do_action(&mut self, idx: usize) -> YYToken {
        match Self::YY_ACTION_TABLE[self.act as usize] {
0 => YYToken::constant_expression({
let __yy1 = NodeId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
{ self.nodes().constant_expression(__yy1 ) }}),
1 => YYToken::expression({
let __yy1 = NameId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
{ self.nodes().identifier(__yy1 ) }}),
2 => YYToken::expression({
let __yy1 = NameId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
{ self.nodes().constant(__yy1 )}}),
3 => YYToken::expression({
let __yy1 = NameId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
{ self.nodes().string_literal(__yy1 ) }}),
4 => YYToken::expression({
let __yy2 = NodeId::from(std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty));
{ __yy2 }}),
5 => YYToken::expression({
let __yy1 = NodeId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
let __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let __yy3 = NodeId::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty));
{ self.nodes().binary(__yy1 , __yy2 , __yy3 ) }}),
6 => YYToken::expression({
let __yy1 = NodeId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
{ self.nodes().function_call(__yy1 , None) }}),
7 => YYToken::expression({
let __yy1 = NodeId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
let __yy3 = NodeId::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty));
{ self.nodes().function_call(__yy1 , Some(__yy3 )) }}),
8 => YYToken::expression({
let __yy1 = NodeId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
let __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let __yy3 = NameId::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty));
{ self.nodes().access(__yy1 , __yy2 , __yy3 ) }}),
9 => YYToken::expression({
let __yy1 = NodeId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
let __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let __yy3 = NameId::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty));
{ self.nodes().access(__yy1 , __yy2 , __yy3 ) }}),
10 => YYToken::expression({
let __yy3 = NodeId::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty));
{ self.nodes().sizeof_expr(__yy3 ) }}),
11 => YYToken::expression({
let __yy3 = TypeId::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty));
{ self.nodes().sizeof_type(__yy3 ) }}),
12 => YYToken::expression({
let __yy2 = TypeId::from(std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty));
let __yy4 = NodeId::from(std::mem::replace(&mut self.value_stack[idx + 3], YYToken::Empty));
{ self.nodes().cast(__yy2 , __yy4 ) }}),
13 => YYToken::expression({
let __yy1 = NodeId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
{ self.nodes().unary(YYToken::POST_INC_OP, __yy1 ) }}),
14 => YYToken::expression({
let __yy1 = NodeId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
{ self.nodes().unary(YYToken::POST_DEC_OP, __yy1 ) }}),
15 => YYToken::expression({
let __yy1 = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty);
let __yy2 = NodeId::from(std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty));
{ self.nodes().unary(__yy1 , __yy2 ) }}),
16 => YYToken::expression({
let __yy1 = NodeId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
let __yy2 = std::mem::replace(&mut self.value_stack[idx + 1], YYToken::Empty);
let __yy3 = NodeId::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty));
{ self.nodes().binary(__yy1 , __yy2 , __yy3 ) }}),
17 => YYToken::expression({
let __yy1 = NodeId::from(std::mem::replace(&mut self.value_stack[idx], YYToken::Empty));
let __yy3 = NodeId::from(std::mem::replace(&mut self.value_stack[idx + 2], YYToken::Empty));
let __yy5 = NodeId::from(std::mem::replace(&mut self.value_stack[idx + 4], YYToken::Empty));
{ self.nodes().ternary(__yy1 , __yy3 , __yy5 ) }}),
            -1 => std::mem::replace(&mut self.value_stack[idx], YYToken::Empty),
            _ => unreachable!(),
        }
    }
}




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
