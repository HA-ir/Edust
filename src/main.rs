use edust::compile_and_run;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: edustc <source-file>");
        std::process::exit(1);
    }
    
    let filename = &args[1];
    
    // Read source file
    let source = fs::read_to_string(filename)
        .unwrap_or_else(|e| {
            eprintln!("Error reading file {}: {}", filename, e);
            std::process::exit(1);
        });
    
    // Compile and run
    match compile_and_run(&source) {
        Ok(exit_code) => {
            println!("\nProgram exited with code: {}", exit_code);
        }
        Err(e) => {
            eprintln!("Compilation error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_program() {
        let source = r#"
            func main() {
                let x = 42;
                return x;
            }
        "#;
        
        let result = compile_and_run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
    
    #[test]
    fn test_arithmetic() {
        let source = r#"
            func main() {
                let a = 10;
                let b = 20;
                let c = a + b * 2;
                return c;
            }
        "#;
        
        let result = compile_and_run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50);
    }
    
    #[test]
    fn test_if_else() {
        let source = r#"
            func main() {
                let x = 5;
                if x > 3 {
                    return 1;
                } else {
                    return 0;
                }
            }
        "#;
        
        let result = compile_and_run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
    
    #[test]
    fn test_while_loop() {
        let source = r#"
            func main() {
                let i = 0;
                let sum = 0;
                while i < 5 {
                    sum = sum + i;
                    i = i + 1;
                }
                return sum;
            }
        "#;
        
        let result = compile_and_run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);
    }
    
    #[test]
    fn test_function_call() {
        let source = r#"
            func add(a, b) {
                return a + b;
            }
            
            func main() {
                let result = add(10, 20);
                return result;
            }
        "#;
        
        let result = compile_and_run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 30);
    }
    
    #[test]
    fn test_comparison_operators() {
        let source = r#"
            func main() {
                let a = 5;
                let b = 10;
                if a < b {
                    if a <= 5 {
                        if b > a {
                            if b >= 10 {
                                if a == 5 {
                                    if b != a {
                                        return 1;
                                    }
                                }
                            }
                        }
                    }
                }
                return 0;
            }
        "#;
        
        let result = compile_and_run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
    
    #[test]
    fn test_logical_operators() {
        let source = r#"
            func main() {
                let a = 1;
                let b = 0;
                if a && !b {
                    if a || b {
                        return 1;
                    }
                }
                return 0;
            }
        "#;
        
        let result = compile_and_run(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}