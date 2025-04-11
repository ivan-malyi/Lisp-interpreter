use core::num;
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

/// Token types for a Lisp-like language.
/// 
/// Represents all possible token types that can be recognized
/// by the lexer when tokenizing a Lisp-like source code.
#[derive(Debug, PartialEq, Clone)]
pub enum TokenEnum {
    LeftParen,      // (
    RightParen,     // )
    Quote,          // '
    Symbol(String), // operators
    Number(f64),    // number literals
    String(String), // string literals
    Bool(bool),     // #t and #f for true and false
    EOF,            // end of file
    Error(String)   // any kind of error
}

impl fmt::Display for TokenEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenEnum::LeftParen           => write!(f, "("),
            TokenEnum::RightParen          => write!(f, ")"),
            TokenEnum::Quote               => write!(f, "'"),
            TokenEnum::Symbol(s)  => write!(f, "{}", s),
            TokenEnum::Number(n)     => write!(f, "{}", n),
            TokenEnum::String(s)  => write!(f, "{}", s),
            TokenEnum::Bool(b)      => write!(f, "#{}", if *b { "t" } else { "f" }),
            TokenEnum::EOF                 => write!(f, "EOF"),
            TokenEnum::Error(s)   => write!(f, "{s}")
        }
    }
}

pub struct Token {
    token: TokenEnum,
    line: usize,
    column: usize
}

impl Token {
    /// Creates a new Token instance with the given token type and position.
    ///
    /// # Arguments
    ///
    /// * `token` - The token type
    /// * `position` - A tuple of (line, column) representing the token's position
    ///
    /// # Returns
    ///
    /// A new Token instance
    fn init_instance(token: TokenEnum, position: (usize, usize)) -> Self {
        let (line, column) = position;
        Self { token, line, column}
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at line {}, column {}", self.token, self.line, self.column)
    }
}

/// A lexical analyzer (lexer) for a Lisp-like language.
///
/// Converts a source string into a sequence of tokens that can be
/// used by a parser to build an abstract syntax tree.
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    column: usize
}
 
impl<'a> Lexer<'a> {
    /// Creates a new Lexer instance for the given input string.
    ///
    /// # Arguments
    ///
    /// * `input` - The source code to tokenize
    ///
    /// # Returns
    ///
    /// A new Lexer instance
    /// 
    pub fn new(input: &'a str) -> Self {
        Self{
            input: input.chars().peekable(),
            line: 1,
            column: 0
        }
    }

    /// Generates an error message for an invalid character or other lexical error.
    ///
    /// # Arguments
    ///
    /// * `c` - The problematic character
    /// * `message` - The error message
    ///
    /// # Returns
    ///
    /// A formatted error string with position information
    /// 
    fn error_handle(&mut self, c: char, message: String) -> String {
        let (line, column) = self.get_position();

        format!("{} {} at line: {} clomun {}", message, c, line, column)
    }

    fn is_valid_character(c: &char) -> bool {
        c.is_alphanumeric() || 
        "()[]{}\"'`,;#|\\+-*/=<>!?_. \t\n\r".contains(*c)
    }
    
    /// Consumes and returns the next character from the input.
    ///
    /// Also updates the line and column counters.
    ///
    /// # Returns
    ///
    /// The next character from the input, or None if the input is exhausted
    /// 
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

    /// Peeks at the next character without consuming it.
    ///
    /// # Returns
    ///
    /// A reference to the next character, or None if the input is exhausted
    /// 
    fn peek_next(&mut self) -> Option<&char> {
        self.input.peek()
    }

    /// Skips whitespace and comments in the input.
    fn trim(&mut self) {
        while let Some(&c) = self.peek_next() {
            if c == ' ' || c == '\n' || c == '\t' || c == '\r' {
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

    /// Tokenizes a symbol starting with the given character.
    ///
    /// # Arguments
    ///
    /// * `first_char` - The first character of the symbol
    ///
    /// # Returns
    ///
    /// A Symbol token
    /// 
    fn tokenize_symbol(&mut self, first_char: char) -> TokenEnum {
        let mut string = String::new();
        string.push(first_char); 

        while let Some(&c) = self.peek_next() {
            if c.is_alphanumeric() || "+-*/=<>!_?".contains(c) {
                string.push(self.next_element().unwrap());
            } else {
                break;
            }
        }
        TokenEnum::Symbol(string)
    }

    /// Tokenizes a number starting with the given character.
    ///
    /// # Arguments
    ///
    /// * `first_char` - The first character of the number (digit or sign)
    ///
    /// # Returns
    ///
    /// A Number token or a Symbol token if the input is not a valid number
    /// 
    /// #TODO
    /// Validate number format
    fn tokenize_number(&mut self, first_char: char) -> TokenEnum {
        let mut number_str = String::new();
        let mut dot = false;

        number_str.push(first_char);

        if (first_char == '-' || first_char == '+') && 
           self.peek_next().map_or(false, |&c| !c.is_digit(10)) {
            return TokenEnum::Symbol(number_str);
        }

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

        match number_str.parse::<f64>() {
            Ok(number) => TokenEnum::Number(number),
            Err(_) => {
                TokenEnum::Symbol(number_str)
            }
        }
    }


    /// Tokenizes a string literal.
    ///
    /// # Returns
    ///
    /// A String token
    /// 
    fn tokenize_string(&mut self) -> TokenEnum {
        let mut string = String::new();

        self.next_element();

        while let Some(&c) = self.peek_next() {
            if c == '"' {
                self.next_element();
                break;
            } else if c == '\\'  {
                self.next_element();
                if let Some(control_flow) = self.next_element() {
                    match control_flow {
                        'n'  => string.push('\n'),
                        't'  => string.push('\t'),
                        'r'  => string.push('\r'),
                        '\\' => string.push('\\'),
                        '"'  => string.push('"'),
                        _    => string.push(control_flow)
                    }
                }
            } else {
                string.push(self.next_element().unwrap());
            }
        }

        TokenEnum::String(string)
    }


    /// Reads and returns the next token from the input.
    ///
    /// # Returns
    ///
    /// The next token from the input
    fn next_token(&mut self) -> TokenEnum {
        self.trim();

        if let Some(c) = self.next_element() {
            if !Lexer::is_valid_character(&c) {
                let error_msg = self.error_handle(c, "Invalid charecter".to_string());
                return TokenEnum::Error(error_msg);
            }

            match c {
                '('  => TokenEnum::LeftParen,
                ')'  => TokenEnum::RightParen,
                '\'' => TokenEnum::Quote,
                '"'  => {
                    self.column -= 1;
                    self.tokenize_string()
                },
                '#'  => {
                    if let Some(&b_char) = self.peek_next() {
                        match b_char {
                            't' => {
                                self.next_element();
                                TokenEnum::Bool(true)
                            }
                            'f' => {
                                self.next_element();
                                TokenEnum::Bool(false)
                            }
                            _   => self.tokenize_symbol('#')
                        }
                    } else {
                        TokenEnum::Symbol("#".to_string())
                    }
                },

                '0'..='9' => self.tokenize_number(c),
                '-' | '+' => {
                    if self.peek_next().map_or(false, |&next| next.is_digit(10)) {
                        self.tokenize_number(c)
                    } else {
                        self.tokenize_symbol(c)
                    }
                },
                _   => self.tokenize_symbol(c)
            }
        } else {
            TokenEnum::EOF
        }
    }


    /// Returns the current position in the input as a (line, column) tuple.
    ///
    /// # Returns
    ///
    /// A tuple of (line, column) representing the current position
    /// 
    pub fn get_position(&mut self) -> (usize, usize) {
        (self.line, self.column)
    }


    /// Tokenizes the entire input and returns a vector of tokens with position information.
    ///
    /// # Returns
    ///
    /// A vector of Token instances
    /// 
    /// # Examples
    ///
    /// Basic usage:
    /// 
    /// ```
    /// use lisp_lexer::Lexer;
    ///
    /// let input = r#"
    /// (define (factorial n)
    ///   (if (= n 0)
    ///       1
    ///       (* n (factorial (- n 1)))))
    /// (factorial 5)
    /// "#;
    ///
    /// let mut lexer = Lexer::new(input);
    /// let tokens = lexer.tokenize_all();
    /// 
    /// // Process or print tokens
    /// for token in &tokens {
    ///     println!("{}", token);
    /// }
    /// ```
    /// 
    /// # Returns
    ///
    /// A vector of `Token` instances representing the entire input program
    /// 
    pub fn tokenize_all(&mut self) -> Vec<Token> {
        let mut token_vector = Vec::new();

        loop {
            let mut token = self.next_token();
            let token_struct: Token = Token::init_instance(token.clone(), self.get_position());
            token_vector.push(token_struct);

            if token == TokenEnum::EOF {
                break;
            }
        }
        token_vector
    }

    
}