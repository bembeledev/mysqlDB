use std::collections::HashMap;

use crate::ast::{BinaryOperator, Expression, Program, Statement, UnaryOperator};
use crate::core::symbol_table::SymbolTable;
use crate::core::types::{SuperType, SuperValue};

pub struct Interpreter {
    pub globals: SymbolTable,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            globals: SymbolTable::new(),
        }
    }

    pub fn eval_program(&mut self, program: Program) -> Result<SuperValue, String> {
        let mut result = SuperValue::Void;

        // Extract globals to avoid borrow checker issues
        let mut current_env = std::mem::replace(&mut self.globals, SymbolTable::new());

        for statement in program.statements {
            match Self::eval_statement_static(statement, &mut current_env) {
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

    pub fn eval_statement_static(stmt: Statement, env: &mut SymbolTable) -> Result<SuperValue, String> {
        match stmt {
            Statement::VariableDeclaration { name, is_mutable, type_annotation, initializer } => {
                let value = Self::eval_expression_static(initializer, env)?;
                env.define(name, type_annotation, value, is_mutable)?;
                Ok(SuperValue::Void)
            }
            Statement::Assignment { name, value } => {
                let eval_val = Self::eval_expression_static(value, env)?;
                env.assign(&name, eval_val)?;
                Ok(SuperValue::Void)
            }
            Statement::ExpressionStatement(expr) => {
                Self::eval_expression_static(expr, env)
            }
            Statement::Block(statements) => {
                let mut new_env = env.clone().spawn_child();
                let mut result = SuperValue::Void;
                for stmt in statements {
                    // Very simple return handling
                    if let Statement::Return(val) = stmt {
                        result = if let Some(expr) = val {
                            Self::eval_expression_static(expr, &mut new_env)?
                        } else {
                            SuperValue::Void
                        };
                        break;
                    }
                    result = Self::eval_statement_static(stmt, &mut new_env)?;
                }
                let _ = new_env.kill_child()?; // Destroy the local scope
                Ok(result)
            }
            Statement::If { condition, consequence, alternative } => {
                let cond_val = Self::eval_expression_static(condition, env)?;
                if let SuperValue::Bool(b) = cond_val {
                    if b {
                        Self::eval_statement_static(*consequence, env)
                    } else if let Some(alt) = alternative {
                        Self::eval_statement_static(*alt, env)
                    } else {
                        Ok(SuperValue::Void)
                    }
                } else {
                    Err("If condition must be a boolean".to_string())
                }
            }
            Statement::Loop { init, condition, increment, iterable, loop_var, body } => {
                let mut loop_env = env.clone().spawn_child();

                if let Some(iterable_expr) = iterable {
                    // Python / JS Style For-In / For-Of
                    let iter_val = Self::eval_expression_static(iterable_expr, &mut loop_env)?;
                    let loop_v = loop_var.unwrap();

                    match iter_val {
                         SuperValue::Object(map) => {
                             for (_key, val) in map {
                                 loop_env.define(loop_v.clone(), SuperType::Any, val, false)?;
                                 Self::eval_statement_static(*body.clone(), &mut loop_env)?;
                             }
                         }
                         // TODO strings, arrays etc.
                         _ => return Err("Expected iterable".to_string())
                    }

                } else {
                     // C / Java Style
                    if let Some(init_stmt) = init {
                        Self::eval_statement_static(*init_stmt, &mut loop_env)?;
                    }

                    loop {
                        if let Some(cond_expr) = &condition {
                            let cond_val = Self::eval_expression_static(cond_expr.clone(), &mut loop_env)?;
                            if let SuperValue::Bool(b) = cond_val {
                                if !b { break; }
                            } else {
                                return Err("Loop condition must be a boolean".to_string());
                            }
                        }

                        Self::eval_statement_static(*body.clone(), &mut loop_env)?;

                        if let Some(inc_stmt) = &increment {
                             Self::eval_statement_static(*inc_stmt.clone(), &mut loop_env)?;
                        }
                    }
                }

                let _ = loop_env.kill_child()?;
                Ok(SuperValue::Void)
            }
            Statement::TypeDeclaration { name, is_dataclass, fields } => {
                if is_dataclass {
                    env.define(name.clone(), SuperType::Any, SuperValue::DataclassConstructor {
                        name: name.clone(),
                        fields,
                    }, false)?;
                }
                Ok(SuperValue::Void)
            }
            Statement::FunctionDeclaration { name, parameters, return_type, body } => {
                env.define(name.clone(), SuperType::Any, SuperValue::Function {
                    parameters,
                    return_type,
                    body,
                }, false)?;
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

    pub fn eval_expression_static(expr: Expression, env: &mut SymbolTable) -> Result<SuperValue, String> {
        match expr {
            Expression::IntLiteral(n) => Ok(SuperValue::Int(n)),
            Expression::FloatLiteral(n) => Ok(SuperValue::Float(n)),
            Expression::StringLiteral(s) => Ok(SuperValue::String(s)),
            Expression::BoolLiteral(b) => Ok(SuperValue::Bool(b)),
            Expression::Identifier(name) => {
                if let Some(symbol) = env.lookup(&name) {
                    Ok(symbol.value)
                } else {
                    Err(format!("Undefined variable '{}'", name))
                }
            }
            Expression::BinaryOp { left, operator, right } => {
                let left_val = Self::eval_expression_static(*left, env)?;
                let right_val = Self::eval_expression_static(*right, env)?;

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
                            _ => Err("Invalid operation on strings".to_string()),
                        }
                    }
                    (SuperValue::Object(a), SuperValue::Object(b)) => {
                        match operator {
                            BinaryOperator::Equal => Ok(SuperValue::Bool(a == b)),
                            BinaryOperator::NotEqual => Ok(SuperValue::Bool(a != b)),
                            _ => Err("Invalid operation on objects".to_string()),
                        }
                    }
                    _ => Err(format!("Type mismatch in binary operation")),
                }
            }
            Expression::UnaryOp { operator, right } => {
                 let right_val = Self::eval_expression_static(*right, env)?;
                 match (operator, right_val) {
                     (UnaryOperator::Minus, SuperValue::Int(n)) => Ok(SuperValue::Int(-n)),
                     (UnaryOperator::Minus, SuperValue::Float(n)) => Ok(SuperValue::Float(-n)),
                     (UnaryOperator::Not, SuperValue::Bool(b)) => Ok(SuperValue::Bool(!b)),
                     _ => Err("Invalid unary operation".to_string()),
                 }
            }
            Expression::FunctionCall { function, arguments } => {
                let func_val = Self::eval_expression_static(*function, env)?;

                let mut eval_args = Vec::new();
                for arg in arguments {
                    eval_args.push(Self::eval_expression_static(arg, env)?);
                }

                match func_val {
                    SuperValue::Function { parameters, return_type, body } => {
                        if parameters.len() != eval_args.len() {
                            return Err(format!("Expected {} arguments, but got {}", parameters.len(), eval_args.len()));
                        }

                        let mut call_env = env.clone().spawn_child();
                        for (i, (param_name, param_type)) in parameters.iter().enumerate() {
                            call_env.define(param_name.clone(), param_type.clone(), eval_args[i].clone(), false)?;
                        }

                        let result = Self::eval_statement_static(*body, &mut call_env)?;
                        let _ = call_env.kill_child()?;

                        if !result.matches(&return_type) {
                            return Err(format!("Function returned {:?} but expected {:?}", result.get_type(), return_type));
                        }

                        Ok(result)
                    }
                    SuperValue::DataclassConstructor { name, fields } => {
                         if fields.len() != eval_args.len() {
                            return Err(format!("Constructor for {} expected {} arguments, got {}", name, fields.len(), eval_args.len()));
                        }

                        let mut obj_map = std::collections::HashMap::new();
                        // For dataclass, we also implicitly add the __type__ property so it's identifiable
                        obj_map.insert("__type__".to_string(), SuperValue::String(name.clone()));

                        for (i, (field_name, field_type)) in fields.iter().enumerate() {
                             if !eval_args[i].matches(field_type) {
                                  return Err(format!("Argument {} must be of type {:?}", field_name, field_type));
                             }
                             obj_map.insert(field_name.clone(), eval_args[i].clone());
                        }
                        Ok(SuperValue::Object(obj_map))
                    }
                    _ => Err("Tried to call a non-function value".to_string())
                }
            }
            Expression::PropertyAccess { object, property } => {
                let obj_val = Self::eval_expression_static(*object, env)?;
                match obj_val {
                    SuperValue::Object(mut map) => {
                         if let Some(val) = map.remove(&property) {
                              Ok(val)
                         } else {
                              Err(format!("Property '{}' not found", property))
                         }
                    }
                    _ => Err("Property access is only supported on objects".to_string())
                }
            }
        }
    }
}
