use crate::ast::*;
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Result<Program, String> {
        let mut program = Program::new();
        
        while !self.is_at_end() {
            let func = self.parse_function()?;
            program.add_function(func);
        }
        
        Ok(program)
    }
    
    // Function = "func" Ident "(" [ ParamList ] ")" Block
    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(TokenType::Func)?;
        
        let name = match &self.current_token().typ {
            TokenType::Ident(s) => s.clone(),
            _ => return Err(self.error("Expected function name")),
        };
        self.advance();
        
        self.expect(TokenType::LParen)?;
        
        let params = self.parse_param_list()?;
        
        self.expect(TokenType::RParen)?;
        
        let body = self.parse_block()?;
        
        Ok(Function { name, params, body })
    }
    
    // ParamList = Ident { "," Ident }
    fn parse_param_list(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        
        if let TokenType::Ident(name) = &self.current_token().typ {
            params.push(name.clone());
            self.advance();
            
            while self.check(&TokenType::Comma) {
                self.advance(); // consume comma
                
                if let TokenType::Ident(name) = &self.current_token().typ {
                    params.push(name.clone());
                    self.advance();
                } else {
                    return Err(self.error("Expected parameter name"));
                }
            }
        }
        
        Ok(params)
    }
    
    // Block = "{" { Statement } "}"
    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(TokenType::LBrace)?;
        
        let mut block = Block::new();
        
        while !self.check(&TokenType::RBrace) && !self.is_at_end() {
            let stmt = self.parse_statement()?;
            block.add_statement(stmt);
        }
        
        self.expect(TokenType::RBrace)?;
        
        Ok(block)
    }
    
    // Statement = VarDecl | Assignment | If | While | Return | Expr ";"
    fn parse_statement(&mut self) -> Result<Statement, String> {
        // VarDecl: "let" Ident "=" Expr ";"
        if self.check(&TokenType::Let) {
            self.advance();
            
            let name = match &self.current_token().typ {
                TokenType::Ident(s) => s.clone(),
                _ => return Err(self.error("Expected variable name")),
            };
            self.advance();
            
            self.expect(TokenType::Assign)?;
            
            let value = self.parse_expr()?;
            
            self.expect(TokenType::Semicolon)?;
            
            return Ok(Statement::VarDecl { name, value });
        }
        
        // If: "if" Expr Block [ "else" Block ]
        if self.check(&TokenType::If) {
            self.advance();
            
            let condition = self.parse_expr()?;
            let then_block = self.parse_block()?;
            
            let else_block = if self.check(&TokenType::Else) {
                self.advance();
                Some(self.parse_block()?)
            } else {
                None
            };
            
            return Ok(Statement::If {
                condition,
                then_block,
                else_block,
            });
        }
        
        // While: "while" Expr Block
        if self.check(&TokenType::While) {
            self.advance();
            
            let condition = self.parse_expr()?;
            let body = self.parse_block()?;
            
            return Ok(Statement::While { condition, body });
        }
        
        // Return: "return" Expr ";"
        if self.check(&TokenType::Return) {
            self.advance();
            
            let value = self.parse_expr()?;
            
            self.expect(TokenType::Semicolon)?;
            
            return Ok(Statement::Return { value });
        }
        
        // Assignment or ExprStmt
        // Look ahead to distinguish assignment from expression statement
        if let TokenType::Ident(name) = &self.current_token().typ {
            let name_clone = name.clone();
            self.advance();
            
            if self.check(&TokenType::Assign) {
                self.advance();
                let value = self.parse_expr()?;
                self.expect(TokenType::Semicolon)?;
                
                return Ok(Statement::Assignment {
                    name: name_clone,
                    value,
                });
            } else {
                // Backtrack - it's an expression statement
                self.current -= 1;
            }
        }
        
        // ExprStmt: Expr ";"
        let expr = self.parse_expr()?;
        self.expect(TokenType::Semicolon)?;
        
        Ok(Statement::ExprStmt { expr })
    }
    
    // Expression parsing using precedence climbing
    
    // Expr = LogicOr
    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_logic_or()
    }
    
    // LogicOr = LogicAnd { "||" LogicAnd }
    fn parse_logic_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_logic_and()?;
        
        while self.check(&TokenType::Or) {
            self.advance();
            let right = self.parse_logic_and()?;
            left = Expr::Binary {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    // LogicAnd = Equality { "&&" Equality }
    fn parse_logic_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        
        while self.check(&TokenType::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::Binary {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    // Equality = Relational { ("==" | "!=") Relational }
    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_relational()?;
        
        while self.check(&TokenType::Eq) || self.check(&TokenType::Ne) {
            let op = if self.check(&TokenType::Eq) {
                BinOp::Eq
            } else {
                BinOp::Ne
            };
            self.advance();
            
            let right = self.parse_relational()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    // Relational = Add { ("<" | "<=" | ">" | ">=") Add }
    fn parse_relational(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_add()?;
        
        while self.check(&TokenType::Lt)
            || self.check(&TokenType::Le)
            || self.check(&TokenType::Gt)
            || self.check(&TokenType::Ge)
        {
            let op = match &self.current_token().typ {
                TokenType::Lt => BinOp::Lt,
                TokenType::Le => BinOp::Le,
                TokenType::Gt => BinOp::Gt,
                TokenType::Ge => BinOp::Ge,
                _ => unreachable!(),
            };
            self.advance();
            
            let right = self.parse_add()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    // Add = Mul { ("+" | "-") Mul }
    fn parse_add(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_mul()?;
        
        while self.check(&TokenType::Plus) || self.check(&TokenType::Minus) {
            let op = if self.check(&TokenType::Plus) {
                BinOp::Add
            } else {
                BinOp::Sub
            };
            self.advance();
            
            let right = self.parse_mul()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    // Mul = Unary { ("*" | "/" | "%") Unary }
    fn parse_mul(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        
        while self.check(&TokenType::Star)
            || self.check(&TokenType::Slash)
            || self.check(&TokenType::Percent)
        {
            let op = match &self.current_token().typ {
                TokenType::Star => BinOp::Mul,
                TokenType::Slash => BinOp::Div,
                TokenType::Percent => BinOp::Mod,
                _ => unreachable!(),
            };
            self.advance();
            
            let right = self.parse_unary()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    // Unary = ("!" | "-") Unary | Primary
    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.check(&TokenType::Bang) || self.check(&TokenType::Minus) {
            let op = if self.check(&TokenType::Bang) {
                UnaryOp::Not
            } else {
                UnaryOp::Neg
            };
            self.advance();
            
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary {
                op,
                operand: Box::new(operand),
            });
        }
        
        self.parse_primary()
    }
    
    // Primary = Number | Ident | "(" Expr ")" | FunctionCall
    fn parse_primary(&mut self) -> Result<Expr, String> {
        // Number
        if let TokenType::Number(n) = self.current_token().typ {
            self.advance();
            return Ok(Expr::Number(n));
        }
        
        // Identifier or FunctionCall
        if let TokenType::Ident(name) = &self.current_token().typ {
            let name_clone = name.clone();
            self.advance();
            
            // Check for function call
            if self.check(&TokenType::LParen) {
                self.advance(); // consume '('
                
                let args = self.parse_arg_list()?;
                
                self.expect(TokenType::RParen)?;
                
                return Ok(Expr::Call {
                    name: name_clone,
                    args,
                });
            }
            
            return Ok(Expr::Variable(name_clone));
        }
        
        // Parenthesized expression
        if self.check(&TokenType::LParen) {
            self.advance();
            let expr = self.parse_expr()?;
            self.expect(TokenType::RParen)?;
            return Ok(expr);
        }
        
        Err(self.error("Expected expression"))
    }
    
    // ArgList = Expr { "," Expr }
    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        
        if !self.check(&TokenType::RParen) {
            args.push(self.parse_expr()?);
            
            while self.check(&TokenType::Comma) {
                self.advance();
                args.push(self.parse_expr()?);
            }
        }
        
        Ok(args)
    }
    
    // Helper methods
    
    fn current_token(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn check(&self, typ: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.current_token().typ) == std::mem::discriminant(typ)
    }
    
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.current_token().typ, TokenType::Eof)
    }
    
    fn expect(&mut self, typ: TokenType) -> Result<(), String> {
        if self.check(&typ) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(&format!("Expected {:?}", typ)))
        }
    }
    
    fn error(&self, msg: &str) -> String {
        let token = self.current_token();
        format!(
            "{} at line {}, column {}",
            msg, token.line, token.column
        )
    }
}