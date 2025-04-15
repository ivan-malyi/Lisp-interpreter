use crate::token::{self, Token};
use crate::token::TokenType;
use std::collections::HashMap;

// rsexp - это библиотека для парсинга S-выражений (основная структурная единица в Lisp).
use rsexp::{Sexp, Error as SexpError};
use std::str::FromStr;

// moka - это библиотека для кэширования с автоматическим контролем времени жизни элементов
use moka::sync::Cache;
use std::time::Duration;

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

// thiserror - библиотека для создания пользовательских типов ошибок.
use thiserror::Error;

// Определение структуры LispVal для представления Lisp-выражений
#[derive(Debug, Clone)]
pub enum LispVal {
    Symbol(String),
    Number(f64),
    String(String),
    Bool(bool),
    List(Vec<LispVal>),
}

// Конвертация из Sexp в LispVal
impl From<Sexp> for LispVal {
    fn from(sexp: Sexp) -> Self {
        match sexp {
            Sexp::Symbol(s) => LispVal::Symbol(s),
            Sexp::String(s) => LispVal::String(s),
            Sexp::Int(i) => LispVal::Number(i as f64),
            Sexp::Float(f) => LispVal::Number(f),
            Sexp::List(list) => {
                let lisp_list: Vec<LispVal> = list.into_iter()
                    .map(LispVal::from)
                    .collect();
                LispVal::List(lisp_list)
            },
            Sexp::Bool(b) => LispVal::Bool(b),
        }
    }
}

// Определение типов ошибок синтаксического анализа
#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("Unexpected closing parenthesis at line {line}, column {column}")]
    UnexpectedClosingParen { line: usize, column: usize },
    
    #[error("Unclosed parenthesis")]
    UnclosedParenthesis,
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Expected symbol after opening parenthesis")]
    ExpectedSymbolAfterOpenParen,
    
    #[error("Invalid number format: {0}")]
    InvalidNumber(String),
}

// Структура для хранения готовых векторов с токенами
#[derive(Clone)]
pub struct ReadyVec {
    vect_tokens: Vec<Token>,       // Исходные токены
    vect_line: usize,              // Номер строки
    indices: HashMap<String, usize>, // Быстрый доступ к токенам
    parsed_value: Option<LispVal>,  // Разобранное значение
}

impl ReadyVec {
    pub fn new(tokens: Vec<Token>, line: usize) -> Self {
        ReadyVec {
            vect_tokens: tokens,
            vect_line: line,
            indices: HashMap::new(),
            parsed_value: None,
        }
    }

    // Новый метод для создания ReadyVec из токенов и LispVal
    pub fn new_with_parsed(tokens: Vec<Token>, line: usize, parsed: LispVal) -> Self {
        let mut ready_vec = ReadyVec::new(tokens, line);
        ready_vec.parsed_value = Some(parsed);
        ready_vec
    }

    // Добавить индекс для токена с ключом
    pub fn add_token_index(&mut self, key: String, token_index: usize) {
        self.indices.insert(key, token_index);
    }

    // Получить токен по ключу
    pub fn get_token(&self, key: &str) -> Option<&Token> {
        self.indices.get(key).map(|&index| &self.vect_tokens[index])
    }

    // Получить изменяемую ссылку на токен по ключу
    pub fn get_token_mut(&mut self, key: &str) -> Option<&mut Token> {
        if let Some(&index) = self.indices.get(key) {
            Some(&mut self.vect_tokens[index])
        } else {
            None
        }
    }

    // Получить токен по индексу
    pub fn get_token_by_index(&self, index: usize) -> Option<&Token> {
        self.vect_tokens.get(index)
    }

    // Получить всю коллекцию токенов
    pub fn get_all_tokens(&self) -> &Vec<Token> {
        &self.vect_tokens
    }

    // Получить номер строки
    pub fn get_line(&self) -> usize {
        self.vect_line
    }

    // Количество токенов
    pub fn len(&self) -> usize {
        self.vect_tokens.len()
    }

    // Проверка на пустоту
    pub fn is_empty(&self) -> bool {
        self.vect_tokens.is_empty()
    }
    
    // Получить разобранное значение
    pub fn get_parsed_value(&self) -> Option<&LispVal> {
        self.parsed_value.as_ref()
    }
    
    // Установить разобранное значение
    pub fn set_parsed_value(&mut self, value: LispVal) {
        self.parsed_value = Some(value);
    }
}

pub struct Tree {
    tree: Vec<ReadyVec>,           // Список готовых векторов
    indices: HashMap<String, usize>, // Быстрый доступ к векторам
}

impl Tree {
    pub fn new() -> Self {
        Tree {
            tree: Vec::new(),
            indices: HashMap::new(),
        }
    }

    // Добавить вектор с ключом
    pub fn add_vec(&mut self, key: String, ready_vec: ReadyVec) {
        let index = self.tree.len();
        self.tree.push(ready_vec);
        self.indices.insert(key, index);
    }

    // Получить вектор по ключу
    pub fn get_vec(&self, key: &str) -> Option<&ReadyVec> {
        self.indices.get(key).map(|&index| &self.tree[index])
    }

    // Получить изменяемую ссылку на вектор по ключу
    pub fn get_vec_mut(&mut self, key: &str) -> Option<&mut ReadyVec> {
        if let Some(&index) = self.indices.get(key) {
            Some(&mut self.tree[index])
        } else {
            None
        }
    }

    // Получить вектор по индексу
    pub fn get_vec_by_index(&self, index: usize) -> Option<&ReadyVec> {
        self.tree.get(index)
    }

    // Получить изменяемую ссылку на вектор по индексу
    pub fn get_vec_by_index_mut(&mut self, index: usize) -> Option<&mut ReadyVec> {
        self.tree.get_mut(index)
    }

    // Получить токен из определенного вектора по ключам
    pub fn get_token(&self, vec_key: &str, token_key: &str) -> Option<&Token> {
        self.get_vec(vec_key).and_then(|vec| vec.get_token(token_key))
    }

    // Количество векторов
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    // Проверка на пустоту
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    // Итерирование по всем векторам
    pub fn iter(&self) -> impl Iterator<Item = &ReadyVec> {
        self.tree.iter()
    }
}

// Функции для работы с синтаксическим анализом

// Функция для преобразования вектора токенов в строку
fn tokens_to_string(tokens: &[Token]) -> String {
    tokens.iter()
        .map(|token| match token.token_type {
            TokenType::LeftParen => "(".to_string(),
            TokenType::RightParen => ")".to_string(),
            TokenType::Symbol => token.lexeme.clone(),
            TokenType::Number => token.lexeme.clone(),
            TokenType::String => format!("\"{}\"", token.lexeme),
            TokenType::Boolean => token.lexeme.clone(),
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// Функция для проверки синтаксиса с использованием rsexp
fn check_syntax_with_rsexp(tokens: &[Token]) -> Result<LispVal, SyntaxError> {
    let s_expr_str = tokens_to_string(tokens);
    
    match Sexp::from_str(&s_expr_str) {
        Ok(sexp) => Ok(LispVal::from(sexp)),
        Err(e) => Err(SyntaxError::ParseError(e.to_string()))
    }
}

// Проверка баланса скобок и других базовых синтаксических правил
fn is_vect_ready(tokens: &[Token]) -> Result<(), SyntaxError> {
    // 1. Проверка балансировки скобок
    let mut paren_counter = 0;
    
    for token in tokens {
        match token.token_type {
            TokenType::LeftParen => paren_counter += 1,
            TokenType::RightParen => {
                paren_counter -= 1;
                if paren_counter < 0 {
                    return Err(SyntaxError::UnexpectedClosingParen {
                        line: token.line,
                        column: token.column
                    });
                }
            },
            _ => {}
        }
    }
    
    if paren_counter > 0 {
        return Err(SyntaxError::UnclosedParenthesis);
    }
    
    // 2. Проверка структуры S-выражений
    // После открывающей скобки должен следовать символ (операнд)
    for i in 0..tokens.len() - 1 {
        if tokens[i].token_type == TokenType::LeftParen {
            if i + 1 < tokens.len() && tokens[i + 1].token_type != TokenType::Symbol {
                // Особый случай: вложенное выражение
                if tokens[i + 1].token_type == TokenType::LeftParen {
                    continue;
                }
                return Err(SyntaxError::ExpectedSymbolAfterOpenParen);
            }
        }
    }
    
    Ok(())
}

// Функция для хэширования вектора токенов
fn hash_token_vector(tokens: &[Token]) -> String {
    let mut hasher = DefaultHasher::new();
    
    for token in tokens {
        // Хэшируем информацию о каждом токене
        std::mem::discriminant(&token.token_type).hash(&mut hasher);
        token.lexeme.hash(&mut hasher);
        token.line.hash(&mut hasher);
        token.column.hash(&mut hasher);
    }
    
    format!("{:x}", hasher.finish())
}

// Функция для создания ReadyVec из вектора токенов
fn create_ready_vec(tokens: Vec<Token>, line: usize, parsed_val: Option<LispVal>) -> ReadyVec {
    let mut ready_vec = match parsed_val {
        Some(val) => ReadyVec::new_with_parsed(tokens.clone(), line, val),
        None => ReadyVec::new(tokens.clone(), line),
    };
    
    // Генерация индексов для быстрого доступа к токенам
    for (i, token) in tokens.iter().enumerate() {
        let key = format!("{}_{}", i, token.lexeme);
        ready_vec.add_token_index(key, i);
    }
    
    ready_vec
}

// Главная структура интерпретатора с использованием moka для кэширования
pub struct LispInterpreter {
    ready_code: Cache<String, ReadyVec>, // Кэш обработанных векторов
    current_ptr: usize,                 // Текущий указатель
    total_lines: usize,                 // Общее количество строк
}

impl LispInterpreter {
    pub fn new(capacity: u64) -> Self {
        LispInterpreter {
            ready_code: Cache::builder()
                .max_capacity(capacity)
                .time_to_live(Duration::from_secs(3600)) // 1 час времени жизни в кэше
                .build(),
            current_ptr: 0,
            total_lines: 0,
        }
    }
    
    // Установка общего количества строк
    pub fn set_total_lines(&mut self, lines: usize) {
        self.total_lines = lines;
    }
    
    // Попытка обработать вектор токенов
    pub fn process_vector(&mut self, tokens: Vec<Token>, line: usize) -> Result<(), SyntaxError> {
        // Проверяем синтаксис
        is_vect_ready(&tokens)?;
        
        // Парсим с помощью rsexp
        let parsed_val = check_syntax_with_rsexp(&tokens)?;
        
        // Создаем хеш для вектора
        let hash = hash_token_vector(&tokens);
        
        // Создаем ReadyVec с разобранным значением
        let ready_vec = create_ready_vec(tokens, line, Some(parsed_val));
        
        // Добавляем в кэш "Ready Code"
        self.ready_code.insert(hash, ready_vec);
        
        // Увеличиваем указатель
        self.current_ptr = line + 1;
        
        Ok(())
    }
    
    // Проверка, нужно ли пересобирать дерево или можно продолжить с текущего места
    pub fn should_rebuild(&self, new_code: &[Vec<Token>]) -> bool {
        if self.current_ptr == 0 || new_code.is_empty() {
            return true; // Если ничего не обработано или новый код пуст, нужна пересборка
        }
        
        // Количество совпадающих векторов
        let mut matches = 0;
        let max_check = std::cmp::min(self.current_ptr, new_code.len());
        
        for i in 0..max_check {
            let hash = hash_token_vector(&new_code[i]);
            if self.ready_code.contains_key(&hash) {
                matches += 1;
            } else {
                break; // При первом несовпадении прерываем проверку
            }
        }
        
        // Коэффициент совпадения
        let match_coefficient = matches as f64 / self.current_ptr as f64;
        
        // Если коэффициент меньше 0.5, требуется пересборка
        match_coefficient < 0.5
    }
    
    // Инвалидация кэша для строк после указанной позиции
    pub fn invalidate_from(&mut self, position: usize) {
        // Получаем все ключи из кэша
        let keys: Vec<String> = self.ready_code.iter()
            .filter_map(|entry| {
                let value = entry.value();
                if value.get_line() >= position {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();
        
        // Инвалидируем каждый ключ
        for key in keys {
            self.ready_code.invalidate(&key);
        }
        
        // Обновляем указатель
        if position < self.current_ptr {
            self.current_ptr = position;
        }
    }
    
    // Получение готового дерева
    pub fn get_tree(&self) -> Tree {
        let mut tree = Tree::new();
        
        // Сортированный список строк
        let mut line_numbers: Vec<usize> = self.ready_code.iter()
            .map(|entry| entry.value().get_line())
            .collect();
        line_numbers.sort();
        
        // Добавляем векторы в порядке строк
        for line in line_numbers {
            for entry in self.ready_code.iter() {
                let ready_vec = entry.value();
                if ready_vec.get_line() == line {
                    tree.add_vec(format!("line_{}", line), ready_vec.clone());
                    break;
                }
            }
        }
        
        tree
    }
    
    // Получение текущего прогресса обработки
    pub fn get_progress(&self) -> (usize, usize) {
        (self.current_ptr, self.total_lines)
    }
    
    // Сброс интерпретатора
    pub fn reset(&mut self) {
        self.ready_code.invalidate_all();
        self.current_ptr = 0;
    }
    
    // Проверка, все ли векторы обработаны
    pub fn is_complete(&self) -> bool {
        self.current_ptr >= self.total_lines && self.total_lines > 0
    }
}

// Рекурсивная функция для обхода дерева Lisp-выражений (для отладки)
pub fn traverse_lisp_val(expr: &LispVal, indent: usize) {
    // Добавляем отступ для визуализации уровня вложенности
    let indent_str = " ".repeat(indent);
    
    match expr {
        LispVal::Symbol(s) => println!("{}Symbol: {}", indent_str, s),
        LispVal::Number(n) => println!("{}Number: {}", indent_str, n),
        LispVal::String(s) => println!("{}String: \"{}\"", indent_str, s),
        LispVal::Bool(b) => println!("{}Boolean: {}", indent_str, b),
        LispVal::List(items) => {
            println!("{}List containing {} items:", indent_str, items.len());
            // Рекурсивно обходим каждый элемент списка с увеличенным отступом
            for item in items {
                traverse_lisp_val(item, indent + 2);
            }
        }
    }
}

// Пример использования интерпретатора
pub fn process_code_example(source_code: &str) -> Result<Tree, SyntaxError> {
    // Создаем интерпретатор
    let mut interpreter = LispInterpreter::new(10_000);
    
    // Получаем токены для всех строк (в реальном коде это сделает ваш лексический анализатор)
    let lines: Vec<&str> = source_code.lines().collect();
    interpreter.set_total_lines(lines.len());
    
    // Здесь бы был вызов лексического анализатора для получения токенов
    // Для примера создадим фиктивные токены
    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue; // Пропускаем пустые строки
        }
        
        // Здесь должен быть вызов вашего лексического анализатора
        // tokens = lexer.tokenize(line);
        // Для примера создаем фиктивные токены
        let tokens = vec![
            Token { 
                token_type: TokenType::LeftParen, 
                lexeme: "(".to_string(), 
                line: i, 
                column: 0 
            },
            Token { 
                token_type: TokenType::Symbol, 
                lexeme: "+".to_string(), 
                line: i, 
                column: 1 
            },
            Token { 
                token_type: TokenType::Number, 
                lexeme: "1".to_string(), 
                line: i, 
                column: 3 
            },
            Token { 
                token_type: TokenType::Number, 
                lexeme: "2".to_string(), 
                line: i, 
                column: 5 
            },
            Token { 
                token_type: TokenType::RightParen, 
                lexeme: ")".to_string(), 
                line: i, 
                column: 6 
            },
        ];
        
        // Обрабатываем вектор токенов
        if let Err(e) = interpreter.process_vector(tokens, i) {
            eprintln!("Error in line {}: {:?}", i + 1, e);
            return Err(e);
        }
    }
    
    // Получаем готовое дерево
    Ok(interpreter.get_tree())
}