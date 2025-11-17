use crate::token::{Token, TokenType};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        
        loop {
            self.skip_whitespace();
            
            if self.is_at_end() {
                tokens.push(Token::new(TokenType::Eof, self.line, self.column));
                break;
            }
            
            let token = self.next_token()?;
            tokens.push(token);
        }
        
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_column = self.column;
        
        let ch = self.current_char();
        
        // Single-character tokens
        match ch {
            '(' => {
                self.advance();
                return Ok(Token::new(TokenType::LParen, start_line, start_column));
            }
            ')' => {
                self.advance();
                return Ok(Token::new(TokenType::RParen, start_line, start_column));
            }
            '{' => {
                self.advance();
                return Ok(Token::new(TokenType::LBrace, start_line, start_column));
            }
            '}' => {
                self.advance();
                return Ok(Token::new(TokenType::RBrace, start_line, start_column));
            }
            ',' => {
                self.advance();
                return Ok(Token::new(TokenType::Comma, start_line, start_column));
            }
            ';' => {
                self.advance();
                return Ok(Token::new(TokenType::Semicolon, start_line, start_column));
            }
            '+' => {
                self.advance();
                return Ok(Token::new(TokenType::Plus, start_line, start_column));
            }
            '-' => {
                self.advance();
                return Ok(Token::new(TokenType::Minus, start_line, start_column));
            }
            '*' => {
                self.advance();
                return Ok(Token::new(TokenType::Star, start_line, start_column));
            }
            '/' => {
                self.advance();
                return Ok(Token::new(TokenType::Slash, start_line, start_column));
            }
            '%' => {
                self.advance();
                return Ok(Token::new(TokenType::Percent, start_line, start_column));
            }
            _ => {}
        }
        
        // Two-character operators
        if ch == '=' {
            self.advance();
            if self.current_char() == '=' {
                self.advance();
                return Ok(Token::new(TokenType::Eq, start_line, start_column));
            }
            return Ok(Token::new(TokenType::Assign, start_line, start_column));
        }
        
        if ch == '!' {
            self.advance();
            if self.current_char() == '=' {
                self.advance();
                return Ok(Token::new(TokenType::Ne, start_line, start_column));
            }
            return Ok(Token::new(TokenType::Bang, start_line, start_column));
        }
        
        if ch == '<' {
            self.advance();
            if self.current_char() == '=' {
                self.advance();
                return Ok(Token::new(TokenType::Le, start_line, start_column));
            }
            return Ok(Token::new(TokenType::Lt, start_line, start_column));
        }
        
        if ch == '>' {
            self.advance();
            if self.current_char() == '=' {
                self.advance();
                return Ok(Token::new(TokenType::Ge, start_line, start_column));
            }
            return Ok(Token::new(TokenType::Gt, start_line, start_column));
        }
        
        if ch == '&' {
            self.advance();
            if self.current_char() == '&' {
                self.advance();
                return Ok(Token::new(TokenType::And, start_line, start_column));
            }
            return Err(format!("Unexpected character '&' at line {}, column {}", start_line, start_column));
        }
        
        if ch == '|' {
            self.advance();
            if self.current_char() == '|' {
                self.advance();
                return Ok(Token::new(TokenType::Or, start_line, start_column));
            }
            return Err(format!("Unexpected character '|' at line {}, column {}", start_line, start_column));
        }
        
        // Numbers
        if ch.is_ascii_digit() {
            return self.read_number(start_line, start_column);
        }
        
        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return self.read_identifier(start_line, start_column);
        }
        
        Err(format!("Unexpected character '{}' at line {}, column {}", ch, start_line, start_column))
    }
    
    fn read_number(&mut self, line: usize, column: usize) -> Result<Token, String> {
        let mut num_str = String::new();
        
        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            num_str.push(self.current_char());
            self.advance();
        }
        
        let value = num_str.parse::<i64>()
            .map_err(|_| format!("Invalid number at line {}, column {}", line, column))?;
        
        Ok(Token::new(TokenType::Number(value), line, column))
    }
    
    fn read_identifier(&mut self, line: usize, column: usize) -> Result<Token, String> {
        let mut ident = String::new();
        
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        let token_type = match ident.as_str() {
            "func" => TokenType::Func,
            "let" => TokenType::Let,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "return" => TokenType::Return,
            _ => TokenType::Ident(ident),
        };
        
        Ok(Token::new(token_type, line, column))
    }
    
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else if ch == '\n' {
                self.line += 1;
                self.column = 1;
                self.position += 1;
            } else {
                break;
            }
        }
    }
    
    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }
    
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
            self.column += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let input = "func main() { let x = 42; }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].typ, TokenType::Func));
        assert!(matches!(tokens[1].typ, TokenType::Ident(_)));
        assert!(matches!(tokens[2].typ, TokenType::LParen));
    }
    
    #[test]
    fn test_operators() {
        let input = "+ - * / % < <= > >= == != && || !";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].typ, TokenType::Plus));
        assert!(matches!(tokens[1].typ, TokenType::Minus));
        assert!(matches!(tokens[2].typ, TokenType::Star));
    }
}