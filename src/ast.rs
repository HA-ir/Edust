/// Abstract Syntax Tree node definitions for Edust

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl {
        name: String,
        value: Expr,
    },
    Assignment {
        name: String,
        value: Expr,
    },
    If {
        condition: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        condition: Expr,
        body: Block,
    },
    Return {
        value: Expr,
    },
    ExprStmt {
        expr: Expr,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Variable(String),
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    
    // Comparison
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    
    // Logical
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,   // -
    Not,   // !
}

impl Program {
    pub fn new() -> Self {
        Program {
            functions: Vec::new(),
        }
    }
    
    pub fn add_function(&mut self, func: Function) {
        self.functions.push(func);
    }
}

impl Block {
    pub fn new() -> Self {
        Block {
            statements: Vec::new(),
        }
    }
    
    pub fn add_statement(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }
}