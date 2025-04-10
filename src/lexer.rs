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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Quote => write!(f, "'"),
            Token::Symbol(s) => write!(f, "{}", s),
            Token::Number(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "{}", s),
            Token::Bool(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            Token::EOF => write!(f, "EOF"),
        }
    }
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    column: usize
} 

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self{
            input: input.chars().peekable(),
            line: 1,
            column: 0
        }
    }

    fn next_element(&mut self) -> Option<char> {
        let next = self.input.next();

        if let Some(c) = next {
            self.column += 1;
            if c == '\n' {
                self.line +=1;
                self.column = 0;
            }
        }

        next
    }

    fn peek_next(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn trim(&mut self) {
        while let Some(&c) = self.peek_next() {
            if c == ' ' {
                self.next_element();
            } else if c == ';' {
                while let Some(c) = self.next_element() {
                    if c == '\n'{
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn tokenize_string(&mut self) -> Token {
        let mut string = String::new();
        self.next_element();

        while let Some(&c) = self.peek_next() {
            if c.is_alphanumeric() || "+-*?/=<>!_".contains(c) {
                string.push(self.next_element().unwrap());
            } else {
                break;
            }
        }

        Token::Symbol(string)
    }

    fn tokenize_number(&mut self) -> Token {
        let mut number: f64 = 0.0;
        let mut number_str = String::new();
        let mut dot = false;

        self.next_element();
        

        while let Some(&c) = self.peek_next() {
            if c.is_digit(10) {
                number_str.push(self.next_element().unwrap());
            } else if c == '.' && !dot {
                dot = true;
                number_str.push(self.next_element().unwrap());
            } else {
                break;
            }
        }

        number = number_str.parse::<f64>().unwrap();
        Token::Number(number)
    }

    fn tokenize_string(&mut self) -> Token {
        let mut string = String::new();

        self.next_element();

        while let Some(&c) = self.peek_next() {
            if c == '"' {
                self.next_element();
                break;
            } else if c == '\\'  {
                self.next_element();
                if let Some(control_flow) = self.next_element {
                    match control_flow {
                        'n' => string.push('\n'),
                        't' => string.push('\t'),
                        'r' => string.push('\r'),
                        '\\' => string.push('\\'),
                        '"' => string.push('"'),
                        _ => string.push(control_flow)
                    }
                }
            } else {
                string.push(self.next_element().unwrap());
            }
        }

        Token::String(string);
    }

}