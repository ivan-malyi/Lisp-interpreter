
#[allow(dead_code)]
// Определение типов токенов
pub enum TokenType {
    LeftParen,    // (
    RightParen,   // )
    Symbol,       // идентификаторы
    Number,       // числа
    String,       // строки
    Boolean,       // типа bool
    // другие типы...
}

#[allow(dead_code)]
// Структура токена
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,     // исходный текст
    pub line: usize,        // номер строки
    pub column: usize,      // номер колонки
}