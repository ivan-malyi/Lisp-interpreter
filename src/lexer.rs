use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

// Token types for Lisp-like language
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LeftParen,      // (
    RightParen,     // )
    Quote,          // '
    Symbol(String), // operators
    Number(f64),    // number literals
    String(String), // string literals
    Bool(bool),     // #t and #f for true and false
    EOF,            // end of file
}