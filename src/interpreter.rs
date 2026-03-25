use crate::ast::{BinaryOperator, Expression, Program, Statement, UnaryOperator};
use crate::core::symbol_table::SymbolTable;
use crate::core::types::{SuperType, SuperValue};
use colored::Colorize;

// 👑 O RESULTADO DA AVALIAÇÃO
// Permite que o 'return' interrompa a execução dos statements seguintes
#[derive(Debug, Clone, PartialEq)]
pub enum EvalResult {
    Value(SuperValue),
    Return(SuperValue),
    Error(String),
    Break,
    Continue,
}

impl EvalResult {
    // Helper para converter para o Result padrão do Rust
    pub fn to_result(self) -> Result<SuperValue, String> {
        match self {
            EvalResult::Value(v) | EvalResult::Return(v) => Ok(v),
            EvalResult::Error(e) => Err(e),
            EvalResult::Break => Err("Sinal 'break' escapou para o escopo global".into()),
            EvalResult::Continue => Err("Sinal 'continue' escapou para o escopo global".into()),
        }
    }
}

pub struct Interpreter {
    pub globals: SymbolTable,
}
/*
impl Interpreter {
    pub fn new() -> Self {
        let mut globals = SymbolTable::new();
        let _ = globals.define(
            "print".to_string(),
            Option::from(SuperType::Any),
            SuperValue::NativeFunction("print".to_string()),
            false,
        );
        let _ = globals.define(
            "println".to_string(),
            Option::from(SuperType::Any),
            SuperValue::NativeFunction("println".to_string()),
            false,
        );

        Interpreter { globals }
    }

    pub fn eval_program(&mut self, program: Program) -> Result<SuperValue, String> {
        let mut current_env = std::mem::replace(&mut self.globals, SymbolTable::new());
        let mut last_value = SuperValue::Void;

        for statement in program.statements {
            match Self::eval_statement_static(statement, &mut current_env) {
                EvalResult::Value(v) => last_value = v,
                EvalResult::Return(v) => {
                    last_value = v;
                    break;
                }
                EvalResult::Error(e) => {
                    // 🎯 CRÍTICO: Devolve o erro para o main.rs ver!
                    self.globals = current_env;
                    return Err(format!("Program Error: {}", e));
                }
                EvalResult::Break | EvalResult::Continue => {
                    self.globals = current_env;
                    return Err("Comando de loop usado fora de um loop".into());
                }
            }
        }
        self.globals = current_env;
        Ok(last_value)
    }

    pub fn eval_statement_static(stmt: Statement, env: &mut SymbolTable) -> EvalResult {
        match stmt {
            Statement::Block(statements) => {
                println!("{}", "  [DEBUG] Entrando em Bloco...".blue());
                let mut new_env = env.clone().spawn_child();
                let mut last_res = EvalResult::Value(SuperValue::Void);

                for (i, s) in statements.into_iter().enumerate() {
                    last_res = Self::eval_statement_static(s, &mut new_env);
                    if matches!(last_res, EvalResult::Return(_) | EvalResult::Error(_)) {
                        break;
                    }
                }
                let _ = new_env.kill_child();
                println!("{}", "  [DEBUG] Saindo do Bloco.".blue());
                last_res
            }

            Statement::VariableDeclaration {
                name,
                is_mutable,
                type_annotation,
                initializer,
            } => {
                println!("  [DEBUG] Declarando variável: {}", name);
                match Self::eval_expression_static(initializer, env) {
                    Ok(value) => {
                        if let Err(e) = env.define(name, type_annotation, value, is_mutable) {
                            EvalResult::Error(e)
                        } else {
                            EvalResult::Value(SuperValue::Void)
                        }
                    }
                    Err(e) => EvalResult::Error(e),
                }
            }
            Statement::ExpressionStatement(expr) => match Self::eval_expression_static(expr, env) {
                Ok(v) => EvalResult::Value(v),
                Err(e) => EvalResult::Error(e),
            },
            Statement::If {
                condition,
                consequence,
                alternative,
            } => match Self::eval_expression_static(condition, env) {
                Ok(SuperValue::Bool(b)) => {
                    if b {
                        Self::eval_statement_static(*consequence, env)
                    } else if let Some(alt) = alternative {
                        Self::eval_statement_static(*alt, env)
                    } else {
                        EvalResult::Value(SuperValue::Void)
                    }
                }
                Ok(_) => EvalResult::Error("If condition must be a boolean".into()),
                Err(e) => EvalResult::Error(e),
            },
            Statement::Loop {
                init,
                condition,
                increment,
                iterable,
                loop_var,
                body,
            } => {
                let mut loop_env = env.clone().spawn_child();

                // Inicialização do loop C-style
                if let Some(init_stmt) = init {
                    if let EvalResult::Error(e) =
                        Self::eval_statement_static(*init_stmt, &mut loop_env)
                    {
                        return EvalResult::Error(e);
                    }
                }

                loop {
                    // Checar condição
                    if let Some(cond_expr) = &condition {
                        match Self::eval_expression_static(cond_expr.clone(), &mut loop_env) {
                            Ok(SuperValue::Bool(false)) => break,
                            Ok(SuperValue::Bool(true)) => (),
                            _ => return EvalResult::Error("Loop condition must be boolean".into()),
                        }
                    }

                    // Executar corpo
                    let res = Self::eval_statement_static(*body.clone(), &mut loop_env);
                    match res {
                        EvalResult::Return(_) | EvalResult::Error(_) => return res,
                        EvalResult::Break => break, // Sai do loop imediatamente
                        EvalResult::Continue => {
                            // Se houver incremento, executa-o antes de continuar
                            if let Some(inc_stmt) = &increment {
                                Self::eval_statement_static(*inc_stmt.clone(), &mut loop_env);
                            }
                            continue;
                        }
                        _ => {} // Continua normalmente
                    }

                    // Incremento
                    if let Some(inc_stmt) = &increment {
                        Self::eval_statement_static(*inc_stmt.clone(), &mut loop_env);
                    }
                }
                EvalResult::Value(SuperValue::Void)
            }
            Statement::Return(expr_opt) => {
                let val = if let Some(expr) = expr_opt {
                    match Self::eval_expression_static(expr, env) {
                        Ok(v) => v,
                        Err(e) => return EvalResult::Error(e),
                    }
                } else {
                    SuperValue::Void
                };
                EvalResult::Return(val) // 🎯 Dispara o sinal de retorno
            }
            Statement::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
            } => {
                // 1. Criamos o valor da função
                // Passamos o 'name' para dentro do SuperValue para que a função saiba quem ela é
                let func = SuperValue::Function {
                    name: name.clone(),
                    parameters,
                    return_type,
                    body,
                };

                // 2. Definimos no ambiente global
                // Usamos Option::from(SuperType::Any) porque, no momento da declaração,
                // o "tipo" da variável que guarda a função é genérico (Function).
                if let Err(e) = env.define(name, Some(SuperType::Any), func, false) {
                    EvalResult::Error(e)
                } else {
                    EvalResult::Value(SuperValue::Void)
                }
            }
            // ... (Outros statements seguem a mesma lógica de propagação)
            _ => EvalResult::Value(SuperValue::Void),
        }
    }

    pub fn eval_expression_static(expr: Expression,env: &mut SymbolTable,) -> Result<SuperValue, String> {
        match expr {
            Expression::IntLiteral(n) => Ok(SuperValue::Int(n)),
            Expression::FloatLiteral(n) => Ok(SuperValue::Float(n)),
            Expression::StringLiteral(s) => Ok(SuperValue::String(s)),
            Expression::BoolLiteral(b) => Ok(SuperValue::Bool(b)),
            Expression::Identifier(name) => env
                .lookup(&name)
                .map(|s| s.value.clone())
                .ok_or(format!("Undefined variable '{}'", name)),
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let l = Self::eval_expression_static(*left, env)?;
                let r = Self::eval_expression_static(*right, env)?;

                let left_type = l.get_type();
                let right_type = r.get_type();

                match (l, r, operator) {
                    // Aritmética Inteira
                    (SuperValue::Int(a), SuperValue::Int(b), BinaryOperator::Plus) => {
                        Ok(SuperValue::Int(a + b))
                    }
                    (SuperValue::Int(a), SuperValue::Int(b), BinaryOperator::Minus) => {
                        Ok(SuperValue::Int(a - b))
                    }
                    (SuperValue::Int(a), SuperValue::Int(b), BinaryOperator::Multiply) => {
                        Ok(SuperValue::Int(a * b))
                    }
                    (SuperValue::Int(a), SuperValue::Int(b), BinaryOperator::Divide) => {
                        Ok(SuperValue::Int(a / b))
                    }
                    (SuperValue::Int(a), SuperValue::Int(b), BinaryOperator::Modulo) => {
                        Ok(SuperValue::Int(a % b))
                    }

                    // 🎯 COMPARAÇÕES (Essencial para o Loop e If)
                    (SuperValue::Int(a), SuperValue::Int(b), BinaryOperator::Equal) => {
                        Ok(SuperValue::Bool(a == b))
                    }
                    (SuperValue::Int(a), SuperValue::Int(b), BinaryOperator::NotEqual) => {
                        Ok(SuperValue::Bool(a != b))
                    }
                    (SuperValue::Int(a), SuperValue::Int(b), BinaryOperator::Less) => {
                        Ok(SuperValue::Bool(a < b))
                    }
                    (SuperValue::Int(a), SuperValue::Int(b), BinaryOperator::LessEqual) => {
                        Ok(SuperValue::Bool(a <= b))
                    }
                    (SuperValue::Int(a), SuperValue::Int(b), BinaryOperator::Greater) => {
                        Ok(SuperValue::Bool(a > b))
                    }
                    (SuperValue::Int(a), SuperValue::Int(b), BinaryOperator::GreaterEqual) => {
                        Ok(SuperValue::Bool(a >= b))
                    }

                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::Plus) => {
                        Ok(SuperValue::Float(a + b))
                    }
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::Divide) => {
                        Ok(SuperValue::Float(a / b))
                    }
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::Modulo) => {
                        Ok(SuperValue::Float(a % b))
                    }

                    // Comparações de Strings
                    (SuperValue::String(a), SuperValue::String(b), BinaryOperator::Equal) => {
                        Ok(SuperValue::Bool(a == b))
                    }
                    (SuperValue::String(a), SuperValue::String(b), BinaryOperator::Plus) => {
                        Ok(SuperValue::String(format!("{}{}", a, b)))
                    }

                    // Igualdade genérica (para tipos diferentes)
                    (a, b, BinaryOperator::Equal) => Ok(SuperValue::Bool(a == b)),
                    (a, b, BinaryOperator::NotEqual) => Ok(SuperValue::Bool(a != b)),

                    _ => Err(format!(
                        "Operação inválida entre {:?} e {:?}",
                        left_type, right_type
                    )),
                }
            }
            // Em src/interpreter.rs dentro de eval_expression_static
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                let func_val = Self::eval_expression_static(*function, env)?;

                let mut eval_args = Vec::new();
                for arg in arguments {
                    eval_args.push(Self::eval_expression_static(arg, env)?);
                }

                match func_val {
                    SuperValue::Function {
                        parameters,
                        return_type,
                        body,
                        ..
                    } => {
                        // 1. Validar quantidade de argumentos
                        if parameters.len() != eval_args.len() {
                            return Err(format!(
                                "Expected {} args, got {}",
                                parameters.len(),
                                eval_args.len()
                            ));
                        }

                        // 2. Criar o escopo da função (Escopo Filho)
                        let mut call_env = env.clone().spawn_child();

                        // 3. Definir os parâmetros no novo escopo
                        for (i, (param_name, param_type)) in parameters.iter().enumerate() {
                            call_env.define(
                                param_name.clone(),
                                Option::from(param_type.clone()),
                                eval_args[i].clone(),
                                false,
                            )?;
                        }

                        // ---------------------------------------------------------
                        // 🎯 COLOQUE O SEU CÓDIGO AQUI:
                        // ---------------------------------------------------------
                        let result = match Self::eval_statement_static(*body.clone(), &mut call_env)
                        {
                            // Se a função disparou um 'return' ou apenas terminou o bloco normalmente
                            EvalResult::Return(v) | EvalResult::Value(v) => {
                                if v.matches(&return_type) {
                                    Ok(v)
                                } else {
                                    Err(format!(
                                        "Type mismatch: function expected {:?}, got {:?}",
                                        return_type,
                                        v.get_type()
                                    ))
                                }
                            }
                            EvalResult::Error(e) => Err(e),
                            _ => Err("Invalid control flow: break/continue outside of loop".into()),
                        };

                        // 4. Limpar o escopo antes de sair
                        let _ = call_env.kill_child();

                        result // Retorna o Ok(v) ou Err(e) para quem chamou a função
                    }

                    SuperValue::NativeFunction(name) => {
                        Self::call_native_function(&name, eval_args)
                    }

                    _ => Err("Tried to call a non-function value".into()),
                }
            }
            // No seu eval_expression_static
            Expression::Lambda {
                parameters,
                return_type,
                body,
            } => Ok(SuperValue::Function {
                name: "<lambda>".to_string(),
                parameters: parameters.clone(),
                return_type: return_type.clone(),
                body: body.clone(),
            }),
            _ => Ok(SuperValue::Void),
        }
    }

    fn call_native_function(name: &str, args: Vec<SuperValue>) -> Result<SuperValue, String> {
        match name {
            "print" => {
                for a in args {
                    print!("{}", a);
                }
                Ok(SuperValue::Void)
            }
            "println" => {
                for a in args {
                    print!("{} ", a);
                }
                println!();
                Ok(SuperValue::Void)
            }
            _ => Err(format!("Native function {} not implemented", name)),
        }
    }
}
*/