pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod semantic;
pub mod token;

use codegen::CodeGenerator;
use lexer::Lexer;
use parser::Parser;
use semantic::SemanticAnalyzer;

/// Complete compilation pipeline for Edust
pub fn compile_and_run(source: &str) -> Result<i64, String> {
    // 1. Lexical analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {}", e))?;
    
    // 2. Parsing
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parser error: {}", e))?;
    
    // 3. Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast).map_err(|e| format!("Semantic error: {}", e))?;
    
    // 4. Code generation
    let mut codegen = CodeGenerator::new();
    let code_ptr = codegen.compile(&ast).map_err(|e| format!("Codegen error: {}", e))?;
    
    // 5. Execute
    let main_fn: fn() -> i64 = unsafe { std::mem::transmute(code_ptr) };
    let result = main_fn();
    
    Ok(result)
}

/// Compile without running (for testing/debugging)
pub fn compile_only(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {}", e))?;
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parser error: {}", e))?;
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast).map_err(|e| format!("Semantic error: {}", e))?;
    
    let mut codegen = CodeGenerator::new();
    let _code_ptr = codegen.compile(&ast).map_err(|e| format!("Codegen error: {}", e))?;
    
    Ok(())
}