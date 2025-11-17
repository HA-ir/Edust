use crate::ast::*;
use std::collections::HashMap;

/// Semantic analyzer performs:
/// - Function signature collection
/// - Variable scope checking
/// - Type checking (basic - all integers for MVP)
pub struct SemanticAnalyzer {
    functions: HashMap<String, FunctionSignature>,
    scopes: Vec<HashMap<String, VarInfo>>,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub param_count: usize,
}

#[derive(Debug, Clone)]
struct VarInfo {
    name: String,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            functions: HashMap::new(),
            scopes: vec![HashMap::new()],
        }
    }
    
    pub fn analyze(&mut self, program: &Program) -> Result<(), String> {
        // First pass: collect all function signatures
        for func in &program.functions {
            if self.functions.contains_key(&func.name) {
                return Err(format!("Duplicate function definition: {}", func.name));
            }
            
            self.functions.insert(
                func.name.clone(),
                FunctionSignature {
                    name: func.name.clone(),
                    param_count: func.params.len(),
                },
            );
        }
        
        // Check for main function
        if !self.functions.contains_key("main") {
            return Err("No main function found".to_string());
        }
        
        if self.functions.get("main").unwrap().param_count != 0 {
            return Err("main function must have no parameters".to_string());
        }
        
        // Second pass: analyze each function body
        for func in &program.functions {
            self.analyze_function(func)?;
        }
        
        Ok(())
    }
    
    fn analyze_function(&mut self, func: &Function) -> Result<(), String> {
        // Create new scope for function
        self.enter_scope();
        
        // Add parameters to scope
        for param in &func.params {
            if self.current_scope().contains_key(param) {
                return Err(format!("Duplicate parameter name: {}", param));
            }
            self.declare_variable(param.clone());
        }
        
        // Analyze function body
        self.analyze_block(&func.body)?;
        
        // Exit function scope
        self.exit_scope();
        
        Ok(())
    }
    
    fn analyze_block(&mut self, block: &Block) -> Result<(), String> {
        for stmt in &block.statements {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }
    
    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::VarDecl { name, value } => {
                self.analyze_expr(value)?;
                
                if self.current_scope().contains_key(name) {
                    return Err(format!("Variable already declared in this scope: {}", name));
                }
                
                self.declare_variable(name.clone());
            }
            
            Statement::Assignment { name, value } => {
                self.analyze_expr(value)?;
                
                if !self.is_variable_declared(name) {
                    return Err(format!("Undefined variable: {}", name));
                }
            }
            
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.analyze_expr(condition)?;
                
                self.enter_scope();
                self.analyze_block(then_block)?;
                self.exit_scope();
                
                if let Some(else_blk) = else_block {
                    self.enter_scope();
                    self.analyze_block(else_blk)?;
                    self.exit_scope();
                }
            }
            
            Statement::While { condition, body } => {
                self.analyze_expr(condition)?;
                
                self.enter_scope();
                self.analyze_block(body)?;
                self.exit_scope();
            }
            
            Statement::Return { value } => {
                self.analyze_expr(value)?;
            }
            
            Statement::ExprStmt { expr } => {
                self.analyze_expr(expr)?;
            }
        }
        
        Ok(())
    }
    
    fn analyze_expr(&self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Number(_) => Ok(()),
            
            Expr::Variable(name) => {
                if !self.is_variable_declared(name) {
                    return Err(format!("Undefined variable: {}", name));
                }
                Ok(())
            }
            
            Expr::Binary { left, right, .. } => {
                self.analyze_expr(left)?;
                self.analyze_expr(right)?;
                Ok(())
            }
            
            Expr::Unary { operand, .. } => {
                self.analyze_expr(operand)?;
                Ok(())
            }
            
            Expr::Call { name, args } => {
                // Check if it's the builtin print function
                if name == "print" {
                    if args.len() != 1 {
                        return Err("print() requires exactly 1 argument".to_string());
                    }
                    self.analyze_expr(&args[0])?;
                    return Ok(());
                }
                
                // Check if function exists
                let sig = self
                    .functions
                    .get(name)
                    .ok_or_else(|| format!("Undefined function: {}", name))?;
                
                // Check argument count
                if args.len() != sig.param_count {
                    return Err(format!(
                        "Function {} expects {} arguments, got {}",
                        name,
                        sig.param_count,
                        args.len()
                    ));
                }
                
                // Analyze all arguments
                for arg in args {
                    self.analyze_expr(arg)?;
                }
                
                Ok(())
            }
        }
    }
    
    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    
    fn exit_scope(&mut self) {
        self.scopes.pop();
    }
    
    fn current_scope(&mut self) -> &mut HashMap<String, VarInfo> {
        self.scopes.last_mut().unwrap()
    }
    
    fn declare_variable(&mut self, name: String) {
        self.current_scope().insert(name.clone(), VarInfo { name });
    }
    
    fn is_variable_declared(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        false
    }
}