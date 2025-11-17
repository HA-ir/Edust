use crate::ast::*;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataContext, FuncId, Linkage, Module};
use std::collections::HashMap;

pub struct CodeGenerator {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    module: JITModule,
    
    // Function ID mappings
    functions: HashMap<String, FuncId>,
    
    // Variable mappings (stack slots) per function
    variables: HashMap<String, Variable>,
    variable_counter: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();
        
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        
        // Declare external C functions
        builder.symbol("print_int", crate::runtime::print_int as *const u8);
        
        let module = JITModule::new(builder);
        
        CodeGenerator {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
            functions: HashMap::new(),
            variables: HashMap::new(),
            variable_counter: 0,
        }
    }
    
    pub fn compile(&mut self, program: &Program) -> Result<*const u8, String> {
        // First pass: declare all functions
        for func in &program.functions {
            self.declare_function(&func.name, func.params.len())?;
        }
        
        // Second pass: compile all function bodies
        for func in &program.functions {
            self.compile_function(func)?;
        }
        
        // Finalize module
        self.module.finalize_definitions().map_err(|e| e.to_string())?;
        
        // Get pointer to main function
        let main_id = self.functions.get("main").ok_or("No main function")?;
        let code = self.module.get_finalized_function(*main_id);
        
        Ok(code)
    }
    
    fn declare_function(&mut self, name: &str, param_count: usize) -> Result<(), String> {
        // All functions return i64 and take i64 parameters
        self.ctx.func.signature.returns.push(AbiParam::new(types::I64));
        
        for _ in 0..param_count {
            self.ctx.func.signature.params.push(AbiParam::new(types::I64));
        }
        
        let func_id = self
            .module
            .declare_function(name, Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;
        
        self.functions.insert(name.to_string(), func_id);
        
        // Clear context for next function
        self.ctx.func.signature.params.clear();
        self.ctx.func.signature.returns.clear();
        
        Ok(())
    }
    
    fn compile_function(&mut self, func: &Function) -> Result<(), String> {
        // Reset variable tracking
        self.variables.clear();
        self.variable_counter = 0;
        
        // Setup function signature
        self.ctx.func.signature.returns.push(AbiParam::new(types::I64));
        for _ in 0..func.params.len() {
            self.ctx.func.signature.params.push(AbiParam::new(types::I64));
        }
        
        let func_id = *self.functions.get(&func.name).unwrap();
        
        // Build function
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        
        // Declare parameters as variables
        let params = builder.block_params(entry_block).to_vec();
        for (i, param_name) in func.params.iter().enumerate() {
            let var = Variable::new(self.variable_counter);
            self.variable_counter += 1;
            self.variables.insert(param_name.clone(), var);
            builder.declare_var(var, types::I64);
            builder.def_var(var, params[i]);
        }
        
        // Compile function body
        let return_val = self.compile_block(&mut builder, &func.body)?;
        
        // Default return 0 if no explicit return
        let final_return = return_val.unwrap_or_else(|| builder.ins().iconst(types::I64, 0));
        builder.ins().return_(&[final_return]);
        
        // Finalize function
        builder.finalize();
        
        // Define the function
        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| e.to_string())?;
        
        // Clear context
        self.module.clear_context(&mut self.ctx);
        
        Ok(())
    }
    
    fn compile_block(
        &mut self,
        builder: &mut FunctionBuilder,
        block: &Block,
    ) -> Result<Option<Value>, String> {
        let mut last_return = None;
        
        for stmt in &block.statements {
            if let Some(ret_val) = self.compile_statement(builder, stmt)? {
                last_return = Some(ret_val);
            }
        }
        
        Ok(last_return)
    }
    
    fn compile_statement(
        &mut self,
        builder: &mut FunctionBuilder,
        stmt: &Statement,
    ) -> Result<Option<Value>, String> {
        match stmt {
            Statement::VarDecl { name, value } => {
                let val = self.compile_expr(builder, value)?;
                
                let var = Variable::new(self.variable_counter);
                self.variable_counter += 1;
                self.variables.insert(name.clone(), var);
                
                builder.declare_var(var, types::I64);
                builder.def_var(var, val);
                
                Ok(None)
            }
            
            Statement::Assignment { name, value } => {
                let val = self.compile_expr(builder, value)?;
                let var = *self.variables.get(name).unwrap();
                builder.def_var(var, val);
                Ok(None)
            }
            
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond_val = self.compile_expr(builder, condition)?;
                
                let then_bb = builder.create_block();
                let else_bb = builder.create_block();
                let merge_bb = builder.create_block();
                
                builder.ins().brif(cond_val, then_bb, &[], else_bb, &[]);
                
                // Then block
                builder.switch_to_block(then_bb);
                builder.seal_block(then_bb);
                self.compile_block(builder, then_block)?;
                builder.ins().jump(merge_bb, &[]);
                
                // Else block
                builder.switch_to_block(else_bb);
                builder.seal_block(else_bb);
                if let Some(else_blk) = else_block {
                    self.compile_block(builder, else_blk)?;
                }
                builder.ins().jump(merge_bb, &[]);
                
                // Merge
                builder.switch_to_block(merge_bb);
                builder.seal_block(merge_bb);
                
                Ok(None)
            }
            
            Statement::While { condition, body } => {
                let header_bb = builder.create_block();
                let loop_body_bb = builder.create_block();
                let exit_bb = builder.create_block();
                
                builder.ins().jump(header_bb, &[]);
                
                // Loop header
                builder.switch_to_block(header_bb);
                let cond_val = self.compile_expr(builder, condition)?;
                builder.ins().brif(cond_val, loop_body_bb, &[], exit_bb, &[]);
                
                // Loop body
                builder.switch_to_block(loop_body_bb);
                builder.seal_block(loop_body_bb);
                self.compile_block(builder, body)?;
                builder.ins().jump(header_bb, &[]);
                
                // Seal header after back edge
                builder.seal_block(header_bb);
                
                // Exit
                builder.switch_to_block(exit_bb);
                builder.seal_block(exit_bb);
                
                Ok(None)
            }
            
            Statement::Return { value } => {
                let val = self.compile_expr(builder, value)?;
                Ok(Some(val))
            }
            
            Statement::ExprStmt { expr } => {
                self.compile_expr(builder, expr)?;
                Ok(None)
            }
        }
    }
    
    fn compile_expr(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &Expr,
    ) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(builder.ins().iconst(types::I64, *n)),
            
            Expr::Variable(name) => {
                let var = *self.variables.get(name).unwrap();
                Ok(builder.use_var(var))
            }
            
            Expr::Binary { op, left, right } => {
                let lhs = self.compile_expr(builder, left)?;
                let rhs = self.compile_expr(builder, right)?;
                
                let result = match op {
                    BinOp::Add => builder.ins().iadd(lhs, rhs),
                    BinOp::Sub => builder.ins().isub(lhs, rhs),
                    BinOp::Mul => builder.ins().imul(lhs, rhs),
                    BinOp::Div => builder.ins().sdiv(lhs, rhs),
                    BinOp::Mod => builder.ins().srem(lhs, rhs),
                    
                    BinOp::Lt => {
                        let cmp = builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs);
                        builder.ins().bint(types::I64, cmp)
                    }
                    BinOp::Le => {
                        let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs);
                        builder.ins().bint(types::I64, cmp)
                    }
                    BinOp::Gt => {
                        let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
                        builder.ins().bint(types::I64, cmp)
                    }
                    BinOp::Ge => {
                        let cmp = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
                        builder.ins().bint(types::I64, cmp)
                    }
                    BinOp::Eq => {
                        let cmp = builder.ins().icmp(IntCC::Equal, lhs, rhs);
                        builder.ins().bint(types::I64, cmp)
                    }
                    BinOp::Ne => {
                        let cmp = builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
                        builder.ins().bint(types::I64, cmp)
                    }
                    
                    BinOp::And => {
                        let lhs_bool = builder.ins().icmp_imm(IntCC::NotEqual, lhs, 0);
                        let rhs_bool = builder.ins().icmp_imm(IntCC::NotEqual, rhs, 0);
                        let result = builder.ins().band(lhs_bool, rhs_bool);
                        builder.ins().bint(types::I64, result)
                    }
                    BinOp::Or => {
                        let lhs_bool = builder.ins().icmp_imm(IntCC::NotEqual, lhs, 0);
                        let rhs_bool = builder.ins().icmp_imm(IntCC::NotEqual, rhs, 0);
                        let result = builder.ins().bor(lhs_bool, rhs_bool);
                        builder.ins().bint(types::I64, result)
                    }
                };
                
                Ok(result)
            }
            
            Expr::Unary { op, operand } => {
                let val = self.compile_expr(builder, operand)?;
                
                let result = match op {
                    UnaryOp::Neg => builder.ins().ineg(val),
                    UnaryOp::Not => {
                        let cmp = builder.ins().icmp_imm(IntCC::Equal, val, 0);
                        builder.ins().bint(types::I64, cmp)
                    }
                };
                
                Ok(result)
            }
            
            Expr::Call { name, args } => {
                // Handle builtin print
                if name == "print" {
                    return self.compile_print_call(builder, &args[0]);
                }
                
                // Regular function call
                let callee_id = *self.functions.get(name).unwrap();
                let local_callee = self.module.declare_func_in_func(callee_id, builder.func);
                
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.compile_expr(builder, arg)?);
                }
                
                let call = builder.ins().call(local_callee, &arg_values);
                Ok(builder.inst_results(call)[0])
            }
        }
    }
    
    fn compile_print_call(
        &mut self,
        builder: &mut FunctionBuilder,
        arg: &Expr,
    ) -> Result<Value, String> {
        let val = self.compile_expr(builder, arg)?;
        
        // Declare print_int external function
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I64));
        
        let print_func = self
            .module
            .declare_function("print_int", Linkage::Import, &sig)
            .map_err(|e| e.to_string())?;
        
        let local_print = self.module.declare_func_in_func(print_func, builder.func);
        
        let call = builder.ins().call(local_print, &[val]);
        Ok(builder.inst_results(call)[0])
    }
}