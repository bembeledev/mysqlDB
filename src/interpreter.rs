use crate::ast::{BinaryOperator, Expression, Program, SpannedExpression, SpannedStatement, Statement, UnaryOperator};
use crate::core::symbol_table::SymbolTable;
use crate::core::types::{SuperType, SuperValue};

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Line {}] Runtime Error: {}", self.line, self.message)
    }
}

pub struct Interpreter {
    pub globals: SymbolTable,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = SymbolTable::new();
        // Inject built-in functions
        let _ = globals.define(
            "print".to_string(),
            SuperType::Any,
            SuperValue::NativeFunction("print".to_string()),
            false
        );
        let _ = globals.define(
            "println".to_string(),
            SuperType::Any,
            SuperValue::NativeFunction("println".to_string()),
            false
        );

        Interpreter { globals }
    }

    pub fn eval_program(&mut self, program: Program) -> Result<SuperValue, RuntimeError> {
        let mut result = SuperValue::Void;

        // Extract globals to avoid borrow checker issues
        let mut current_env = std::mem::replace(&mut self.globals, SymbolTable::new());

        for statement in program.statements {
            match Self::eval_statement_static(&statement, &mut current_env) {
                Ok(val) => { result = val; },
                Err(e) => {
                    // Restore globals before returning error
                    self.globals = current_env;
                    return Err(e);
                }
            }
        }

        // Restore globals
        self.globals = current_env;

        Ok(result)
    }

    pub fn eval_statement_static(spanned_stmt: &SpannedStatement, env: &mut SymbolTable) -> Result<SuperValue, RuntimeError> {
        let line = spanned_stmt.line;
        let make_err = |msg: String| RuntimeError { line, message: msg };

        match &spanned_stmt.stmt {
            Statement::VariableDeclaration { name, is_mutable, type_annotation, initializer } => {
                let value = Self::eval_expression_static(initializer, env)?;
                env.define(name.clone(), type_annotation.clone(), value, *is_mutable).map_err(make_err)?;
                Ok(SuperValue::Void)
            }
            Statement::Assignment { name, value } => {
                let eval_val = Self::eval_expression_static(value, env)?;
                env.assign(name, eval_val).map_err(make_err)?;
                Ok(SuperValue::Void)
            }
            Statement::PropertyAssignment { object, property, value } => {
                let mut obj_val = Self::eval_expression_static(object, env)?;
                let eval_val = Self::eval_expression_static(value, env)?;

                if let SuperValue::Object(ref mut map) = obj_val {
                     map.insert(property.clone(), eval_val);
                     // If the object was evaluated from an identifier, we need to reassign it back to save state
                     if let Expression::Identifier(name) = &object.expr {
                         env.assign(name, SuperValue::Object(map.clone())).map_err(make_err)?;
                     } else if let Expression::PropertyAccess { object: parent_obj, property: parent_prop } = &object.expr {
                         env.assign(parent_prop, SuperValue::Object(map.clone())).ok();
                     }
                } else {
                     return Err(make_err("Property assignment is only supported on objects".to_string()));
                }
                Ok(SuperValue::Void)
            }
            Statement::ExpressionStatement(expr) => {
                Self::eval_expression_static(expr, env)
            }
            Statement::Block(statements) => {
                let mut new_env = env.clone().spawn_child();
                let mut result = SuperValue::Void;
                for stmt in statements {
                    if let Statement::Return(val) = &stmt.stmt {
                        result = if let Some(expr) = val {
                            Self::eval_expression_static(expr, &mut new_env)?
                        } else {
                            SuperValue::Void
                        };
                        break;
                    }
                    result = Self::eval_statement_static(stmt, &mut new_env)?;
                }
                let _ = new_env.kill_child().map_err(make_err)?; // Destroy the local scope
                Ok(result)
            }
            Statement::If { condition, consequence, alternative } => {
                let cond_val = Self::eval_expression_static(condition, env)?;
                if let SuperValue::Bool(b) = cond_val {
                    if b {
                        Self::eval_statement_static(consequence, env)
                    } else if let Some(alt) = alternative {
                        Self::eval_statement_static(alt, env)
                    } else {
                        Ok(SuperValue::Void)
                    }
                } else {
                    Err(make_err("If condition must be a boolean".to_string()))
                }
            }
            Statement::Loop { init, condition, increment, iterable, loop_var, body } => {
                let mut loop_env = env.clone().spawn_child();

                if let Some(iterable_expr) = iterable {
                    let iter_val = Self::eval_expression_static(iterable_expr, &mut loop_env)?;
                    let loop_v = loop_var.as_ref().unwrap();

                    match iter_val {
                         SuperValue::Object(map) => {
                             for (_key, val) in map {
                                 let mut iter_env = loop_env.clone().spawn_child();
                                 iter_env.define(loop_v.clone(), SuperType::Any, val, false).map_err(make_err.clone())?;
                                 Self::eval_statement_static(body, &mut iter_env)?;
                                 let _ = iter_env.kill_child().map_err(make_err.clone())?;
                             }
                         }
                         _ => return Err(make_err("Expected iterable".to_string()))
                    }

                } else {
                    if let Some(init_stmt) = init {
                        Self::eval_statement_static(init_stmt, &mut loop_env)?;
                    }

                    loop {
                        if let Some(cond_expr) = condition {
                            let cond_val = Self::eval_expression_static(cond_expr, &mut loop_env)?;
                            if let SuperValue::Bool(b) = cond_val {
                                if !b { break; }
                            } else {
                                return Err(make_err("Loop condition must be a boolean".to_string()));
                            }
                        }

                        Self::eval_statement_static(body, &mut loop_env)?;

                        if let Some(inc_stmt) = increment {
                             Self::eval_statement_static(inc_stmt, &mut loop_env)?;
                        }
                    }
                }

                let _ = loop_env.kill_child().map_err(make_err)?;
                Ok(SuperValue::Void)
            }
            Statement::TypeDeclaration { name, is_dataclass, fields } => {
                if *is_dataclass {
                    env.define(name.clone(), SuperType::Any, SuperValue::DataclassConstructor {
                        name: name.clone(),
                        fields: fields.clone(),
                    }, false).map_err(make_err)?;
                }
                Ok(SuperValue::Void)
            }
            Statement::FunctionDeclaration { name, parameters, return_type, body } => {
                env.define(name.clone(), SuperType::Any, SuperValue::Function {
                    parameters: parameters.clone(),
                    return_type: return_type.clone(),
                    body: Box::new(body.stmt.clone()),
                }, false).map_err(make_err)?;
                Ok(SuperValue::Void)
            }
            Statement::ClassDeclaration { name, extends, fields, methods, .. } => {
                let mut method_map = std::collections::HashMap::new();
                for method_stmt in methods {
                    if let Statement::FunctionDeclaration { name: method_name, parameters, return_type, body } = &method_stmt.stmt {
                        method_map.insert(method_name.clone(), SuperValue::Function {
                            parameters: parameters.clone(),
                            return_type: return_type.clone(),
                            body: Box::new(body.stmt.clone()),
                        });
                    }
                }

                env.define(name.clone(), SuperType::Any, SuperValue::Class {
                    name: name.clone(),
                    extends: extends.clone(),
                    fields: fields.clone(),
                    methods: method_map,
                }, false).map_err(make_err)?;

                Ok(SuperValue::Void)
            }
            Statement::InterfaceDeclaration { .. } => Ok(SuperValue::Void), // No-op at runtime for now
            Statement::EnumDeclaration { name, variants } => {
                let mut enum_map = std::collections::HashMap::new();
                for (i, var) in variants.into_iter().enumerate() {
                    enum_map.insert(var.clone(), SuperValue::Int(i as i64));
                }
                env.define(name.clone(), SuperType::Any, SuperValue::Object(enum_map), false).map_err(make_err)?;
                Ok(SuperValue::Void)
            }
            Statement::TryCatch { try_block, catch_var, catch_type: _, catch_block, finally_block } => {
                let res = Self::eval_statement_static(try_block, env);
                let mut final_res = res.clone();
                if let Err(e) = res {
                    if e.message.starts_with("Throw:") {
                        let mut catch_env = env.clone().spawn_child();
                        let error_msg = e.message.trim_start_matches("Throw:").trim().to_string();
                        catch_env.define(catch_var.clone(), SuperType::Any, SuperValue::String(error_msg), false).map_err(make_err.clone())?;
                        final_res = Self::eval_statement_static(catch_block, &mut catch_env);
                        let _ = catch_env.kill_child().map_err(make_err.clone())?;
                    } else {
                        final_res = Err(e);
                    }
                }
                if let Some(fin) = finally_block {
                    let _ = Self::eval_statement_static(fin, env)?;
                }
                final_res
            }
            Statement::Throw(expr) => {
                let val = Self::eval_expression_static(expr, env)?;
                Err(make_err(format!("Throw: {}", val)))
            }
            Statement::ImportStatement { path } => {
                let content = std::fs::read_to_string(&path)
                    .map_err(|_| make_err(format!("Could not read imported file '{}'", path)))?;

                let lexer = crate::lexer::Lexer::new(&content);
                let tokens = lexer.tokenize();
                let mut parser = crate::parser::Parser::new(tokens);
                let program = parser.parse().map_err(|e| make_err(format!("Syntax error in '{}': {}", path, e)))?;

                // Evaluate imported module in the current environment
                for stmt in program.statements {
                    Self::eval_statement_static(&stmt, env)?;
                }

                Ok(SuperValue::Void)
            }
            Statement::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    Self::eval_expression_static(expr, env)
                } else {
                    Ok(SuperValue::Void)
                }
            }
        }
    }

    pub fn eval_expression_static(spanned_expr: &SpannedExpression, env: &mut SymbolTable) -> Result<SuperValue, RuntimeError> {
        let line = spanned_expr.line;
        let make_err = |msg: String| RuntimeError { line, message: msg };

        match &spanned_expr.expr {
            Expression::IntLiteral(n) => Ok(SuperValue::Int(*n)),
            Expression::FloatLiteral(n) => Ok(SuperValue::Float(*n)),
            Expression::StringLiteral(s) => Ok(SuperValue::String(s.to_string())),
            Expression::BoolLiteral(b) => Ok(SuperValue::Bool(*b)),
            Expression::Identifier(name) => {
                if let Some(symbol) = env.lookup(name) {
                    Ok(symbol.value)
                } else {
                    Err(make_err(format!("Undefined variable '{}'", name)))
                }
            }
            Expression::BinaryOp { left, operator, right } => {
                let left_val = Self::eval_expression_static(left, env)?;
                let right_val = Self::eval_expression_static(right, env)?;

                match (left_val, right_val) {
                    (SuperValue::Int(a), SuperValue::Int(b)) => {
                        match operator {
                            BinaryOperator::Plus => Ok(SuperValue::Int(a + b)),
                            BinaryOperator::Minus => Ok(SuperValue::Int(a - b)),
                            BinaryOperator::Multiply => Ok(SuperValue::Int(a * b)),
                            BinaryOperator::Divide => Ok(SuperValue::Int(a / b)),
                            BinaryOperator::Equal => Ok(SuperValue::Bool(a == b)),
                            BinaryOperator::NotEqual => Ok(SuperValue::Bool(a != b)),
                            BinaryOperator::Less => Ok(SuperValue::Bool(a < b)),
                            BinaryOperator::Greater => Ok(SuperValue::Bool(a > b)),
                            BinaryOperator::LessEqual => Ok(SuperValue::Bool(a <= b)),
                            BinaryOperator::GreaterEqual => Ok(SuperValue::Bool(a >= b)),
                        }
                    }
                    (SuperValue::Float(a), SuperValue::Float(b)) => {
                         match operator {
                            BinaryOperator::Plus => Ok(SuperValue::Float(a + b)),
                            BinaryOperator::Minus => Ok(SuperValue::Float(a - b)),
                            BinaryOperator::Multiply => Ok(SuperValue::Float(a * b)),
                            BinaryOperator::Divide => Ok(SuperValue::Float(a / b)),
                            BinaryOperator::Equal => Ok(SuperValue::Bool(a == b)),
                            BinaryOperator::NotEqual => Ok(SuperValue::Bool(a != b)),
                            BinaryOperator::Less => Ok(SuperValue::Bool(a < b)),
                            BinaryOperator::Greater => Ok(SuperValue::Bool(a > b)),
                            BinaryOperator::LessEqual => Ok(SuperValue::Bool(a <= b)),
                            BinaryOperator::GreaterEqual => Ok(SuperValue::Bool(a >= b)),
                        }
                    }
                    (SuperValue::String(a), SuperValue::String(b)) => {
                        match operator {
                            BinaryOperator::Plus => Ok(SuperValue::String(format!("{}{}", a, b))),
                            BinaryOperator::Equal => Ok(SuperValue::Bool(a == b)),
                            BinaryOperator::NotEqual => Ok(SuperValue::Bool(a != b)),
                            _ => Err(make_err("Invalid operation on strings".to_string())),
                        }
                    }
                    (SuperValue::String(a), b) | (b, SuperValue::String(a)) => {
                        // Fallback string concatenation
                        if *operator == BinaryOperator::Plus {
                            Ok(SuperValue::String(format!("{}{}", a, b)))
                        } else {
                            Err(make_err("Type mismatch in binary operation".to_string()))
                        }
                    }
                    (SuperValue::Object(a), SuperValue::Object(b)) => {
                        match operator {
                            BinaryOperator::Equal => Ok(SuperValue::Bool(a == b)),
                            BinaryOperator::NotEqual => Ok(SuperValue::Bool(a != b)),
                            _ => Err(make_err("Invalid operation on objects".to_string())),
                        }
                    }
                    (a, b) => {
                         if *operator == BinaryOperator::Equal {
                             return Ok(SuperValue::Bool(a == b));
                         } else if *operator == BinaryOperator::NotEqual {
                             return Ok(SuperValue::Bool(a != b));
                         }
                         Err(make_err("Type mismatch in binary operation".to_string()))
                    }
                }
            }
            Expression::UnaryOp { operator, right } => {
                 let right_val = Self::eval_expression_static(right, env)?;
                 match (operator, right_val) {
                     (UnaryOperator::Minus, SuperValue::Int(n)) => Ok(SuperValue::Int(-n)),
                     (UnaryOperator::Minus, SuperValue::Float(n)) => Ok(SuperValue::Float(-n)),
                     (UnaryOperator::Not, SuperValue::Bool(b)) => Ok(SuperValue::Bool(!b)),
                     _ => Err(make_err("Invalid unary operation".to_string())),
                 }
            }
            Expression::FunctionCall { function, arguments } => {
                let mut self_binding = None;
                if let Expression::PropertyAccess { ref object, property: _ } = function.expr {
                    self_binding = Some(Self::eval_expression_static(object, env)?);
                }

                let func_val = Self::eval_expression_static(function, env)?;

                let mut eval_args = Vec::new();
                for arg in arguments {
                    eval_args.push(Self::eval_expression_static(arg, env)?);
                }

                match func_val {
                    SuperValue::Function { parameters, return_type, body } => {
                        if parameters.len() != eval_args.len() {
                            return Err(make_err(format!("Expected {} arguments, but got {}", parameters.len(), eval_args.len())));
                        }

                        let mut call_env = env.clone().spawn_child();
                        for (i, (param_name, param_type)) in parameters.iter().enumerate() {
                            call_env.define(param_name.clone(), param_type.clone(), eval_args[i].clone(), false).map_err(make_err.clone())?;
                        }

                        if let Some(obj) = self_binding {
                             call_env.define("self".to_string(), SuperType::Any, obj, true).map_err(make_err.clone())?;
                        }

                        // Fake a SpannedStatement to pass to evaluator
                        let fake_spanned = SpannedStatement { stmt: *body, line: 0 };
                        let result = Self::eval_statement_static(&fake_spanned, &mut call_env)?;
                        let _ = call_env.kill_child().map_err(make_err.clone())?;

                        if result != SuperValue::Void && !result.matches(&return_type) {
                            return Err(make_err(format!("Function returned {:?} but expected {:?}", result.get_type(), return_type)));
                        }

                        Ok(result)
                    }
                    SuperValue::DataclassConstructor { name, fields } => {
                         if fields.len() != eval_args.len() {
                            return Err(make_err(format!("Constructor for {} expected {} arguments, got {}", name, fields.len(), eval_args.len())));
                        }

                        let mut obj_map = std::collections::HashMap::new();
                        obj_map.insert("__type__".to_string(), SuperValue::String(name.clone()));

                        for (i, (field_name, field_type)) in fields.iter().enumerate() {
                             if !eval_args[i].matches(field_type) {
                                  return Err(make_err(format!("Argument {} must be of type {:?}", field_name, field_type)));
                             }
                             obj_map.insert(field_name.clone(), eval_args[i].clone());
                        }
                        Ok(SuperValue::Object(obj_map))
                    }
                    SuperValue::NativeFunction(name) => {
                        Self::call_native_function(&name, eval_args).map_err(make_err)
                    }
                    _ => Err(make_err("Tried to call a non-function value".to_string()))
                }
            }
            Expression::PropertyAccess { object, property } => {
                let obj_val = Self::eval_expression_static(object, env)?;
                match obj_val {
                    SuperValue::Object(mut map) => {
                         if let Some(val) = map.remove(property) {
                              Ok(val)
                         } else {
                              Err(make_err(format!("Property '{}' not found", property)))
                         }
                    }
                    _ => Err(make_err("Property access is only supported on objects".to_string()))
                }
            }
            Expression::ObjectInstantiation { class_name, arguments } => {
                let class_val = if let Some(sym) = env.lookup(class_name) {
                    sym.value
                } else {
                    return Err(make_err(format!("Undefined class '{}'", class_name)));
                };

                let mut eval_args = Vec::new();
                for arg in arguments {
                    eval_args.push(Self::eval_expression_static(arg, env)?);
                }

                match class_val {
                    SuperValue::Class { name, fields, methods, extends, .. } => {
                        let mut obj_map = std::collections::HashMap::new();
                        obj_map.insert("__type__".to_string(), SuperValue::String(name.clone()));

                        // In SPL, constructor logic is explicit via `constructor()` method.
                        // So we do not automatically bind fields here, we just instantiate an empty object,
                        // inject the fields as null/void, and then call constructor if arguments exist or if it exists.
                        // Actually, for Dataclasses we enforce arguments to fields directly.
                        // For normal classes, we just map fields.

                        for (field_name, _, _) in fields.clone() {
                            obj_map.insert(field_name, SuperValue::Void);
                        }

                        // Attach methods
                        for (method_name, method_val) in methods.clone() {
                             obj_map.insert(method_name, method_val);
                        }

                        // Apply extends
                        if let Some(parent_class_name) = &extends {
                            if let Some(parent_sym) = env.lookup(parent_class_name) {
                                if let SuperValue::Class { methods: p_methods, .. } = parent_sym.value {
                                    for (m_name, m_val) in p_methods {
                                        obj_map.insert(m_name, m_val);
                                    }
                                }
                            }
                        }

                        // Call constructor if it exists
                        if let Some(SuperValue::Function { parameters, body, return_type }) = obj_map.get("constructor").cloned() {
                            if parameters.len() != eval_args.len() {
                                return Err(make_err(format!("Constructor for {} expected {} arguments, got {}", name, parameters.len(), eval_args.len())));
                            }

                            let mut call_env = env.clone().spawn_child();
                            for (i, (param_name, param_type)) in parameters.into_iter().enumerate() {
                                call_env.define(param_name, param_type, eval_args[i].clone(), false).map_err(make_err.clone())?;
                            }
                            call_env.define("self".to_string(), SuperType::Any, SuperValue::Object(obj_map.clone()), true).map_err(make_err.clone())?;

                            let fake_spanned = crate::ast::SpannedStatement { stmt: *body, line: 0 };
                            let _ = Self::eval_statement_static(&fake_spanned, &mut call_env)?;

                            // Retrieve updated self
                            if let Some(self_sym) = call_env.lookup("self") {
                                if let SuperValue::Object(updated_map) = self_sym.value {
                                    obj_map = updated_map;
                                }
                            }

                            let _ = call_env.kill_child().map_err(make_err)?;
                        } else if !eval_args.is_empty() {
                            return Err(make_err(format!("Class {} has no constructor but was called with arguments", name)));
                        }


                        Ok(SuperValue::Object(obj_map))
                    }
                    SuperValue::DataclassConstructor { name, fields } => {
                        if fields.len() != eval_args.len() {
                           return Err(make_err(format!("Constructor for {} expected {} arguments, got {}", name, fields.len(), eval_args.len())));
                       }

                       let mut obj_map = std::collections::HashMap::new();
                       obj_map.insert("__type__".to_string(), SuperValue::String(name));

                       for (i, (field_name, field_type)) in fields.iter().enumerate() {
                            if !eval_args[i].matches(field_type) {
                                 return Err(make_err(format!("Argument {} must be of type {:?}", field_name, field_type)));
                            }
                            obj_map.insert(field_name.clone(), eval_args[i].clone());
                       }
                       Ok(SuperValue::Object(obj_map))
                   }
                   _ => Err(make_err(format!("'{}' is not a class or dataclass", class_name)))
                }
            }
            Expression::ArrayLiteral(elements) => {
                 let mut items = Vec::new();
                 for el in elements {
                     items.push(Self::eval_expression_static(el, env)?);
                 }
                 let mut obj_map = std::collections::HashMap::new();
                 obj_map.insert("__type__".to_string(), SuperValue::String("Array".to_string()));
                 for (i, v) in items.into_iter().enumerate() {
                      obj_map.insert(i.to_string(), v);
                 }
                 Ok(SuperValue::Object(obj_map))
            }
            Expression::PolyglotBlock { language, content } => {
                let mut vars = std::collections::HashMap::new();
                for entry in env.symbols.iter() {
                    let k = entry.key().clone();
                    let v = entry.value().value.to_string();
                    vars.insert(k, v);
                }

                match language.as_str() {
                    "js" | "ts" => {
                        let res = crate::ministers::js_bridge::eval_js_block(content, &vars).map_err(|e| make_err(e))?;
                        Ok(SuperValue::String(res))
                    }
                    "py" => {
                        let res = crate::ministers::python_bridge::eval_py_block(content, &vars).map_err(|e| make_err(e))?;
                        Ok(SuperValue::String(res))
                    }
                    _ => {
                        Ok(SuperValue::String(format!("[{}-Execution-Mock: {}]", language, content.trim())))
                    }
                }
            }
            Expression::Lambda { parameters, body } => {
                 Ok(SuperValue::Function {
                      parameters: parameters.clone(),
                      return_type: SuperType::Any,
                      body: Box::new(body.stmt.clone()),
                 })
            }
        }
    }

    fn call_native_function(name: &str, args: Vec<SuperValue>) -> Result<SuperValue, String> {
        match name {
            "print" => {
                for arg in args {
                    print!("{}", arg);
                }
                Ok(SuperValue::Void)
            }
            "println" => {
                for arg in args {
                    print!("{} ", arg);
                }
                println!();
                Ok(SuperValue::Void)
            }
            _ => Err(format!("Native function {} not implemented", name)),
        }
    }
}
