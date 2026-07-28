#![allow(unused_braces, mixed_script_confusables, unused)]
use std::io::{Read, stdin};
use std::str::from_utf8;

#[allow(unused)]
const INITIAL: usize = 0;
#[allow(unused)]
const INITIAL_BOL: usize = 1;

const READ_LEN: usize = 4096;

#[derive(Debug, PartialEq)]
enum LexerState {
    Error,
    Running,
    Eof,
}

#[allow(non_camel_case_types, mixed_script_confusables)]
#[derive(Debug, Clone, PartialEq)]
pub enum YYToken {
    Operator(char),
    Number(i32),
    yyeof,
}

pub trait YYLexer {
    fn yylex(&mut self) -> YYToken;
}

#[derive(Debug)]
struct AcceptData {
    state: usize,
    buf_pos: usize,
}

pub struct YYLex<R: Read, C> {
    state: LexerState,
    action: isize,
    current_state: usize,
    start_condition: usize,
    buffer: Vec<u8>,
    buffer_position: usize,
    run_position: usize,
    trailing_end_pos: usize,
    trailing_action: isize,
    bol: bool,
    more: bool,
    accept_stack: Vec<AcceptData>,
    yycontinue: Box<dyn FnMut() -> Option<R>>,

    pub yyin: R,
    pub yytext: String,
    pub line_no: isize,
    pub col_no: isize,
    pub ctx: C,
}

#[allow(unused)]
impl Default for YYLex<std::io::Stdin, u32> {
    fn default() -> Self {
        Self::new(stdin(), || None, 1)
    }
}

impl<R: Read> YYLex<R, u32> {
    pub fn with_reader(yyin: R) -> YYLex<R, u32> {
        Self::new(yyin, || None, 1)
    }
}

#[allow(unused)]
impl<R: Read, C> YYLex<R, C> {
    pub fn new<F>(yyin: R, yycontinue: F, ctx: C) -> Self
    where
        F: FnMut() -> Option<R> + 'static,
    {
        Self {
            state: LexerState::Running,
            action: 0,
            current_state: 1,
            start_condition: 0,
            buffer: Vec::new(),
            buffer_position: 0,
            run_position: 0,
            trailing_end_pos: 0,
            trailing_action: -1,
            bol: true,
            more: false,
            accept_stack: Vec::new(),
            yycontinue: Box::new(yycontinue),
            yyin,
            yytext: String::new(),
            line_no: 0,
            col_no: 0,
            ctx,
        }
    }

    const YY_CHAR_EQ: [isize; 256] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
        71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
        94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
        113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130,
        131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148,
        149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166,
        167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184,
        185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202,
        203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220,
        221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238,
        239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
    ];
    const YY_BASE: [isize; 2304] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 5, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 3, 3, 3, 3, 6, 3, 3, 3, 3, 6, 6,
        3, 3, 3, 3, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 5, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 4, 3, 3, 3, 3, 6, 3, 3, 3, 3, 6, 6, 3, 3, 3, 3, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5,
        5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8,
        8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    const YY_START: [isize; 2] = [1, 2];
    const YY_TRAILLING: [isize; 9] = [-1, -1, -1, -1, -1, -1, -1, -1, -1];
    const YY_ACCEPT: [isize; 9] = [-1, 1, 1, 3, 2, 2, 0, 1, 1];
    const YY_NEXT_ACCEPT: [isize; 36] = [
        -1, -1, -1, -1, -1, 2, -1, -1, -1, 2, -1, -1, -1, -1, -1, -1, -1, -1, 3, -1, -1, -1, -1,
        -1, 3, -1, -1, -1, -1, 3, -1, -1, -1, -1, -1, -1,
    ];
    const YY_CLASS_COUNT: usize = 256;
    const YY_RULE_COUNT: usize = 4;

    fn begin(&mut self, condition: usize) {
        self.start_condition = condition;
    }

    fn reject(&mut self) {
        let next_action = Self::YY_NEXT_ACCEPT
            [self.stack_top().state * Self::YY_RULE_COUNT + self.action as usize];
        self.action = if next_action >= 0 {
            next_action
        } else {
            self.accept_stack.pop();
            Self::YY_ACCEPT[self.stack_top().state]
        };
    }

    fn keep_bytes_to_process(&mut self) -> usize {
        let bytes_to_keep = self.buffer.len() - self.buffer_position;
        let len = self.buffer.len();
        if bytes_to_keep > 0 {
            self.buffer.copy_within((len - bytes_to_keep)..len, 0);
        }
        self.shift_all_positions(self.buffer_position);
        bytes_to_keep
    }

    fn shift_all_positions(&mut self, offset: usize) {
        self.accept_stack
            .iter_mut()
            .for_each(|s| s.buf_pos -= offset);
        self.run_position -= offset;
        self.trailing_end_pos = self.trailing_end_pos.saturating_sub(offset);
        self.buffer_position -= offset;
    }

    fn load_buffer(&mut self) {
        let mut bytes_to_keep = self.keep_bytes_to_process();

        let mut read_try_len = self.buffer.len() - bytes_to_keep;
        if read_try_len == 0 {
            self.buffer.extend([0; READ_LEN]);
            read_try_len = self.buffer.len() - bytes_to_keep;
        }
        loop {
            let read_len = self.yyin.read(&mut self.buffer[bytes_to_keep..]).unwrap();
            if read_len == read_try_len {
                break;
            }
            match (self.yycontinue)() {
                Some(yyin) => {
                    self.yyin = yyin;
                    read_try_len -= read_len;
                    bytes_to_keep += read_len;
                    continue;
                }
                None => {
                    self.buffer[bytes_to_keep + read_len] = 0;
                    self.state = LexerState::Eof;
                    break;
                }
            }
        }

        if let Some(first_zero) = self.buffer.iter().position(|&b| b == 0) {
            self.buffer.truncate(first_zero);
        }
    }

    fn try_accept(&mut self) {
        if Self::YY_TRAILLING[self.current_state] >= 0 {
            self.trailing_action = Self::YY_TRAILLING[self.current_state];
            self.trailing_end_pos = self.run_position;
        }
        if Self::YY_ACCEPT[self.current_state] < 0 {
            return;
        }
        self.accept_stack.push(AcceptData {
            state: self.current_state,
            buf_pos: self.run_position,
        });
    }

    fn count(&mut self) {
        for c in self.yytext.chars() {
            if c == '\n' {
                self.line_no += 1;
                self.col_no = 0;
            } else if c == '\t' {
                self.col_no += 4 - (self.col_no % 4);
            } else {
                self.col_no += 1;
            }
        }
    }

    fn build_yytext(&mut self) {
        self.yytext = from_utf8(&self.buffer[self.buffer_position..self.run_position])
            .unwrap()
            .to_string();
        self.count();
    }

    pub fn yymore(&mut self) {
        self.more = true;
    }

    pub fn yyless(&mut self, n: usize) {
        self.run_position -= self.yytext.len() - n;
        self.build_yytext();
    }

    pub fn input(&mut self) -> u8 {
        if self.run_position >= self.buffer.len() {
            self.load_buffer();
        }
        if self.run_position >= self.buffer.len() {
            return 0;
        }
        let ret = *self
            .buffer
            .get(self.run_position)
            .expect("run_position out of bounds");
        self.run_position += 1;
        ret
    }

    pub fn unput(&mut self, c: u8) {
        self.run_position -= 1;
        self.buffer[self.run_position] = c;
        self.yytext.pop();
    }

    fn stack_top(&self) -> &AcceptData {
        match self.accept_stack.last() {
            Some(t) => t,
            None => unreachable!(),
        }
    }

    fn stack_top_mut(&mut self) -> &mut AcceptData {
        match self.accept_stack.last_mut() {
            Some(t) => t,
            None => unreachable!(),
        }
    }

    fn prepare_run(&mut self) -> Option<()> {
        if self.state == LexerState::Error {
            return None;
        }

        if self.more {
            self.more = false;
        } else {
            self.buffer_position = self.run_position;
            self.bol = self.buffer_position == 0 || self.buffer[self.buffer_position - 1] == b'\n';
        }

        if self.run_position >= self.buffer.len() && self.state == LexerState::Eof {
            return None;
        }

        self.trailing_end_pos = 0;
        self.trailing_action = -1;

        self.current_state = Self::YY_START[self.start_condition + self.bol as usize] as usize;
        self.accept_stack = vec![AcceptData {
            state: 0,
            buf_pos: self.buffer_position,
        }];
        Some(())
    }

    fn run_dfa(&mut self) {
        loop {
            if self.run_position == self.buffer.len() {
                self.load_buffer();
            }

            if self.run_position == self.buffer.len() {
                break;
            }

            let char_class: usize =
                Self::YY_CHAR_EQ[self.buffer[self.run_position] as usize] as usize;
            self.current_state =
                Self::YY_BASE[self.current_state * Self::YY_CLASS_COUNT + char_class] as usize;
            if self.current_state == 0 {
                break;
            }

            self.try_accept();
            self.run_position += 1;
        }
    }

    fn prepare_action(&mut self) {
        let top = self.stack_top();
        self.action = Self::YY_ACCEPT[top.state];
        if self.trailing_action >= 0 && self.trailing_action == self.action {
            self.stack_top_mut().buf_pos = self.trailing_end_pos;
        }
        self.run_position = std::cmp::min(self.stack_top().buf_pos + 1, self.buffer.len());
        self.build_yytext();
    }

    fn yylex(&mut self) -> YYToken {
        loop {
            if self.prepare_run().is_none() {
                return YYToken::yyeof;
            }
            self.run_dfa();
            self.prepare_action();
            match self.action {
                0 => {
                    self.ctx = 3_u32;
                    return YYToken::Operator(self.yytext.chars().next().unwrap());
                }

                1 => {
                    return YYToken::Number(self.yytext.parse::<i32>().unwrap());
                }

                2 => {}

                3 => {
                    println!("Unknown token: {}", self.yytext);
                }
                -1 => print!("{}", self.yytext),
                _ => panic!("wrong action\n"),
            }
        }
    }
}

fn main() {
    use std::env;
    use std::io::{Cursor, Read};
    let args: Vec<String> = env::args().collect();
    let mut reader = Cursor::new(args[1].clone());
    let mut lexer: YYLex<_, u32> = YYLex::new(reader, || None, 1_u32);
    loop {
        match lexer.yylex() {
            YYToken::Operator(op) => println!("Operator: {}", op),
            YYToken::Number(n) => println!("Number: {}", n),
            YYToken::yyeof => break,
            _ => (),
        }
    }
    println!("final: {}", lexer.ctx);
}
