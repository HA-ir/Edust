/// Token types for the Edust language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number(i64),
    Ident(String),
    
    // Keywords
    Func,
    Let,
    If,
    Else,
    While,
    Return,
    
    // Operators
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    
    // Comparison
    Lt,         // <
    Le,         // <=
    Gt,         // >
    Ge,         // >=
    Eq,         // ==
    Ne,         // !=
    
    // Logical
    And,        // &&
    Or,         // ||
    Bang,       // !
    
    // Assignment
    Assign,     // =
    
    // Delimiters
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    Comma,      // ,
    Semicolon,  // ;
    
    // Special
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(typ: TokenType, line: usize, column: usize) -> Self {
        Token { typ, line, column }
    }
}