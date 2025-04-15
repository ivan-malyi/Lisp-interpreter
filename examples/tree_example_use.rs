use std::error::Error;
use lisp_interpreter::token::{Token, TokenType};
use lisp_interpreter::syntax_parser::{LispInterpreter, traverse_lisp_val, Tree};

// Функция для преобразования строки Lisp-кода в токены (имитация лексического анализатора)
fn tokenize(code: &str, line: usize) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = code.chars().enumerate();
    
    while let Some((i, c)) = chars.next() {
        match c {
            '(' => tokens.push(Token {
                token_type: TokenType::LeftParen,
                lexeme: "(".to_string(),
                line,
                column: i,
            }),
            ')' => tokens.push(Token {
                token_type: TokenType::RightParen,
                lexeme: ")".to_string(),
                line,
                column: i,
            }),
            '+' | '-' | '*' | '/' | 'd' | 'e' | 'f' | 'i' | 'n' | 'a' | 'r' => {
                // Упрощенная обработка символов
                if c == 'd' && code.len() > i + 5 && &code[i..i+6] == "define" {
                    tokens.push(Token {
                        token_type: TokenType::Symbol,
                        lexeme: "define".to_string(),
                        line,
                        column: i,
                    });
                    // Пропускаем остальные символы "define"
                    for _ in 0..5 {
                        chars.next();
                    }
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Symbol,
                        lexeme: c.to_string(),
                        line,
                        column: i,
                    });
                }
            },
            '0'..='9' => {
                let mut number = c.to_string();
                while let Some((_, nc)) = chars.clone().next() {
                    if nc.is_digit(10) || nc == '.' {
                        number.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    token_type: TokenType::Number,
                    lexeme: number,
                    line,
                    column: i,
                });
            },
            '#' => {
                // Обработка булевых значений
                if let Some((_, next)) = chars.next() {
                    match next {
                        't' => tokens.push(Token {
                            token_type: TokenType::Boolean,
                            lexeme: "#t".to_string(),
                            line,
                            column: i,
                        }),
                        'f' => tokens.push(Token {
                            token_type: TokenType::Boolean,
                            lexeme: "#f".to_string(),
                            line,
                            column: i,
                        }),
                        _ => {} // Игнорируем неизвестные # последовательности
                    }
                }
            },
            '"' => {
                // Упрощенная обработка строк
                let mut string = String::new();
                while let Some((_, nc)) = chars.next() {
                    if nc == '"' {
                        break;
                    }
                    string.push(nc);
                }
                tokens.push(Token {
                    token_type: TokenType::String,
                    lexeme: string,
                    line,
                    column: i,
                });
            },
            ' ' | '\t' | '\r' => {}, // Пропускаем пробельные символы
            _ => {} // Игнорируем другие символы для простоты
        }
    }
    
    tokens
}

// Пример использования нашего синтаксического анализатора
fn main() -> Result<(), Box<dyn Error>> {
    println!("Lisp Syntax Parser Demo");
    println!("======================");
    
    // Создаем интерпретатор
    let mut interpreter = LispInterpreter::new(1000);
    
    // Пример кода Lisp
    let lisp_code = vec![
        "(+ 1 2)",
        "(define x 10)",
        "(+ x (* 3 4))",
        "(if #t (+ 1 2) (- 3 4))",
        "(define (square x) (* x x))",
        "(square 5)"
    ];
    
    // Устанавливаем общее количество строк
    interpreter.set_total_lines(lisp_code.len());
    
    println!("\nProcessing Lisp code:");
    for (i, line) in lisp_code.iter().enumerate() {
        println!("Line {}: {}", i + 1, line);
        let tokens = tokenize(line, i);
        
        println!("  Tokens: {:?}", tokens);
        
        match interpreter.process_vector(tokens, i) {
            Ok(()) => println!("  Parsed successfully"),
            Err(e) => {
                println!("  Error: {:?}", e);
                // Предположим, что мы исправили ошибку и хотим продолжить
                println!("  (Error would be fixed by user)");
            }
        }
        println!();
    }
    
    // Получаем построенное дерево
    let tree = interpreter.get_tree();
    
    // Вывод информации о дереве
    println!("\nTree Contents:");
    println!("Number of vectors in tree: {}", tree.len());
    
    // Печатаем содержимое каждого вектора в дереве
    for i in 0..tree.len() {
        if let Some(vec) = tree.get_vec_by_index(i) {
            println!("\nVector at line {}", vec.get_line() + 1);
            println!("Number of tokens: {}", vec.len());
            
            // Вывод разобранного значения, если оно есть
            if let Some(parsed) = vec.get_parsed_value() {
                println!("Parsed value structure:");
                traverse_lisp_val(parsed, 2);
            }
        }
    }
    
    // Демонстрация кэширования - изменяем код и проверяем решение о пересборке
    println!("\nTesting incremental parsing:");
    
    // Новая версия кода с изменением в одной строке
    let modified_code = vec![
        "(+ 1 2)",
        "(define x 20)", // Изменено значение с 10 на 20
        "(+ x (* 3 4))",
        "(if #t (+ 1 2) (- 3 4))",
        "(define (square x) (* x x))",
        "(square 5)"
    ];
    
    // Токенизируем измененный код
    let tokens_modified: Vec<Vec<Token>> = modified_code
        .iter()
        .enumerate()
        .map(|(i, line)| tokenize(line, i))
        .collect();
    
    // Проверяем, нужна ли пересборка
    let rebuild = interpreter.should_rebuild(&tokens_modified);
    println!("Should rebuild tree: {}", rebuild);
    
    // Если rebuild = false, то можно продолжить с места изменения
    if !rebuild {
        println!("Can continue incremental parsing");
        // Инвалидируем кэш начиная с измененной строки
        interpreter.invalidate_from(1); // Строка с индексом 1 изменилась
        
        // Перепарсим только измененные строки
        for i in 1..modified_code.len() {
            let tokens = tokenize(&modified_code[i], i);
            match interpreter.process_vector(tokens, i) {
                Ok(()) => println!("  Line {} re-parsed successfully", i + 1),
                Err(e) => println!("  Error re-parsing line {}: {:?}", i + 1, e),
            }
        }
    } else {
        println!("Need to rebuild the entire tree");
        // Сбрасываем интерпретатор и начинаем заново
        interpreter.reset();
        interpreter.set_total_lines(modified_code.len());
        
        for (i, line) in modified_code.iter().enumerate() {
            let tokens = tokenize(line, i);
            match interpreter.process_vector(tokens, i) {
                Ok(()) => println!("  Line {} parsed successfully", i + 1),
                Err(e) => println!("  Error parsing line {}: {:?}", i + 1, e),
            }
        }
    }
    
    // Получаем обновленное дерево
    let updated_tree = interpreter.get_tree();
    println!("\nUpdated Tree size: {}", updated_tree.len());
    
    // Проверяем, полностью ли обработан код
    println!("Parsing complete: {}", interpreter.is_complete());
    
    // Выводим прогресс обработки
    let (current, total) = interpreter.get_progress();
    println!("Progress: {}/{} lines processed", current, total);
    
    Ok(())
}