use crate::ast::{BinaryOperator, Expression, ImportType, Program, Statement, UnaryOperator};
use crate::core::symbol_table::SymbolTable;
use crate::core::types::{SuperType, SuperValue};
use crate::lexer::Lexer;
use crate::parser::Parser;
use colored::Colorize;

#[derive(Debug, Clone, PartialEq)]
pub enum EvalResult {
    Value(SuperValue),
    Return(SuperValue),
    Error(String),
    Break,
    Continue,
}

impl EvalResult {
    pub fn to_result(self) -> Result<SuperValue, String> {
        match self {
            EvalResult::Value(v) | EvalResult::Return(v) => Ok(v),
            EvalResult::Error(e) => Err(e),
            EvalResult::Break => Err("Break outside loop".into()),
            EvalResult::Continue => Err("Continue outside loop".into()),
        }
    }
}

// Fora do impl Interpreter, talvez num ficheiro module_loader.rs ou no próprio interpreter.rs
pub fn load_module_extern(module_path: &str) -> Result<SymbolTable, String> {
    // 🎯 O segredo é permitir subpastas e garantir a extensão
    let file_path = if module_path.ends_with(".super") {
        module_path.to_string()
    } else {
        format!("{}.super", module_path)
    };

    let source = std::fs::read_to_string(&file_path)
        .map_err(|_| format!("Erro: Módulo não encontrado em '{}'", file_path))?;
    let mut temp_interpreter = Interpreter::new();
    let tokens = Lexer::new(&source).tokenize();
    let program = Parser::new(tokens)
        .parse()
        .map_err(|e| format!("Erro no módulo '{}': {}", module_path, e))?;

    // Executa e retorna apenas a SymbolTable (os globais do módulo)
    match temp_interpreter.eval_program(program) {
        Ok(_) => Ok(temp_interpreter.globals),
        Err(e) => Err(format!("Erro ao executar módulo: {}", e)),
    }
}

pub struct Interpreter {
    pub globals: SymbolTable,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = SymbolTable::new();

        // 🎯 Registar os "Ministros" (Funções Nativas)
        // Utilitários de I/O
        globals.define_native("prompt", "prompt");
        globals.define_native("prompt_read", "prompt_read");
        globals.define_native("println", "println");
        globals.define_native("print", "print");
        globals.define_native("clear", "clear");

        // 🧮 Funções Matemáticas Nativas (Mapeadas do f64 do Rust)
        let math_functions = vec![
            // Potência e Raízes
            "sqrt", "cbrt", "pow", "exp", // Logaritmos
            "ln", "log10", "log2", "log", // Trigonometria
            "sin", "cos", "tan", "asin", "acos", "atan", "atan2",
            // Arredondamento e Utilitários
            "abs", "floor", "ceil", "round", "fract",
        ];

        for func in math_functions {
            globals.define_native(func, func);
        }

        Interpreter { globals }
    }

    pub fn eval_program(&mut self, program: Program) -> Result<SuperValue, String> {
        let mut last_value = SuperValue::Void;

        for statement in program.statements {
            match Self::eval_statement_static(statement, &mut self.globals) {
                EvalResult::Value(v) => last_value = v,
                EvalResult::Return(v) => return Ok(v),
                EvalResult::Error(e) => return Err(format!("Runtime Error: {}", e)),
                EvalResult::Break | EvalResult::Continue => {
                    return Err("Control signal escaped loop".into());
                }
            }
        }

        Ok(last_value)
    }

    pub fn eval_statement_static(stmt: Statement, env: &mut SymbolTable) -> EvalResult {
        match stmt {
            Statement::Import {
                module,
                import_type,
            } => {
                // 🎯 Chamada para a função independente
                let module_interpreter = match load_module_extern(&module) {
                    Ok(mi) => mi,
                    Err(e) => return EvalResult::Error(e), // Propaga a String de erro
                };
                match import_type {
                    ImportType::Star => {
                        for entry in module_interpreter.symbols.iter() {
                            let name = entry.key();
                            let symbol = entry.value();

                            // 2. CORREÇÃO: Tratamos o Result do define sem usar o '?'
                            if let Err(e) = env.define(
                                name.clone(),
                                Some(symbol.symbol_type.clone()),
                                symbol.value.clone(),
                                symbol.is_mutable,
                            ) {
                                return EvalResult::Error(format!(
                                    "Erro ao importar '{}': {}",
                                    name, e
                                ));
                            }
                        }
                    }
                    ImportType::Symbols(list) => {
                        for name in list {
                            if let Some(symbol) = module_interpreter.lookup(&name) {
                                let _ = env.define(
                                    name.clone(),
                                    Some(symbol.symbol_type.clone()),
                                    symbol.value.clone(),
                                    symbol.is_mutable,
                                );
                            } else {
                                return EvalResult::Error(format!("Símbolo '{}' não existe", name));
                            }
                        }
                    }
                }
                EvalResult::Value(SuperValue::Void)
            }

            Statement::Block(statements) => {
                let mut child_env = env.clone().spawn_child();
                let mut last_res = EvalResult::Value(SuperValue::Void);

                for s in statements {
                    last_res = Self::eval_statement_static(s, &mut child_env);
                    if !matches!(last_res, EvalResult::Value(_)) {
                        break;
                    }
                }

                last_res
            }

            Statement::VariableDeclaration {
                name,
                is_mutable,
                type_annotation,
                initializer,
            } => match Self::eval_expression_static(initializer, env) {
                Ok(value) => {
                    if let Err(e) = env.define(name, type_annotation, value, is_mutable) {
                        EvalResult::Error(e)
                    } else {
                        EvalResult::Value(SuperValue::Void)
                    }
                }
                Err(e) => EvalResult::Error(e),
            },

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
                Ok(_) => EvalResult::Error("If condition must be boolean".into()),
                Err(e) => EvalResult::Error(e),
            },

            Statement::While { condition, body } => {
                let mut loop_env = env.clone().spawn_child();

                loop {
                    match Self::eval_expression_static(condition.clone(), &mut loop_env) {
                        Ok(SuperValue::Bool(false)) => break,
                        Ok(SuperValue::Bool(true)) => (),
                        Ok(_) => {
                            return EvalResult::Error("While condition must be boolean".into());
                        }
                        Err(e) => return EvalResult::Error(e),
                    }

                    match Self::eval_statement_static(*body.clone(), &mut loop_env) {
                        EvalResult::Return(v) => return EvalResult::Return(v),
                        EvalResult::Error(e) => return EvalResult::Error(e),
                        EvalResult::Break => break,
                        EvalResult::Continue => continue,
                        _ => {}
                    }
                }

                EvalResult::Value(SuperValue::Void)
            }

            Statement::For {
                init,
                condition,
                increment,
                body,
            } => {
                let mut loop_env = env.clone().spawn_child(); // 1. Cria o ambiente do loop

                // 2. Inicializa (ex: var i = 1)
                if let Some(init_stmt) = init {
                    Self::eval_statement_static(*init_stmt, &mut loop_env);
                }

                loop {
                    // 3. Verifica condição (i <= 5)
                    if let Some(cond_expr) = &condition {
                        match Self::eval_expression_static(cond_expr.clone(), &mut loop_env) {
                            Ok(SuperValue::Bool(false)) => break, // Sai se for falso
                            Ok(SuperValue::Bool(true)) => (),
                            _ => return EvalResult::Error("Condition must be boolean".into()),
                        }
                    }

                    // 4. Executa o corpo (verificar_paridade)
                    let res = Self::eval_statement_static(*body.clone(), &mut loop_env);
                    match res {
                        EvalResult::Return(_) | EvalResult::Error(_) => return res,
                        EvalResult::Break => break,
                        _ => {}
                    }

                    // 🎯 O ERRO ESTÁ AQUI:
                    // O incremento (i = i + 1) deve ser executado no MESMO loop_env
                    if let Some(inc_stmt) = &increment {
                        // Se o teu eval_statement_static para ExpressionStatement não
                        // estiver a atualizar a variável no 'env' (SymbolTable), o 'i' nunca muda!
                        Self::eval_statement_static(*inc_stmt.clone(), &mut loop_env);
                    }
                }
                EvalResult::Value(SuperValue::Void)
            }

            Statement::ForIn {
                variable,
                iterable,
                body,
            } => {
                let mut loop_env = env.clone().spawn_child();

                let iter_value = match Self::eval_expression_static(iterable, &mut loop_env) {
                    Ok(v) => v,
                    Err(e) => return EvalResult::Error(e),
                };

                let items = match iter_value {
                    SuperValue::Array(arr) => arr,
                    _ => return EvalResult::Error("For-in requires an array".into()),
                };

                for item in items {
                    if let Err(e) = loop_env.define(
                        variable.clone(),
                        Some(item.get_type()),
                        item.clone(),
                        false,
                    ) {
                        return EvalResult::Error(e);
                    }

                    match Self::eval_statement_static(*body.clone(), &mut loop_env) {
                        EvalResult::Return(v) => return EvalResult::Return(v),
                        EvalResult::Error(e) => return EvalResult::Error(e),
                        EvalResult::Break => break,
                        EvalResult::Continue => continue,
                        _ => {}
                    }
                }

                EvalResult::Value(SuperValue::Void)
            }

            Statement::Break => EvalResult::Break,
            Statement::Continue => EvalResult::Continue,

            Statement::Return(expr_opt) => {
                let val = match expr_opt {
                    Some(e) => match Self::eval_expression_static(e, env) {
                        Ok(v) => v,
                        Err(e) => return EvalResult::Error(e),
                    },
                    None => SuperValue::Void,
                };
                EvalResult::Return(val)
            }

            Statement::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
            } => {
                let func = SuperValue::Function {
                    name: name.clone(),
                    parameters,
                    return_type,
                    body,
                };

                if let Err(e) = env.define(name, Some(SuperType::Any), func, false) {
                    EvalResult::Error(e)
                } else {
                    EvalResult::Value(SuperValue::Void)
                }
            }

            _ => EvalResult::Value(SuperValue::Void),
        }
    }

    pub fn eval_expression_static(
        expr: Expression,
        env: &mut SymbolTable,
    ) -> Result<SuperValue, String> {
        match expr {
            Expression::IntLiteral(n) => Ok(SuperValue::Int(n)),
            Expression::FloatLiteral(f) => Ok(SuperValue::Float(f)),
            Expression::StringLiteral(s) => Ok(SuperValue::String(s)),
            Expression::BoolLiteral(b) => Ok(SuperValue::Bool(b)),
            Expression::Null => Ok(SuperValue::Null),

            Expression::ArrayLiteral(elements) => {
                let mut vals = vec![];
                for e in elements {
                    vals.push(Self::eval_expression_static(e, env)?);
                }
                Ok(SuperValue::Array(vals))
            }

            Expression::Identifier(name) => env
                .lookup(&name)
                .map(|s| s.value.clone())
                .ok_or(format!("Undefined variable: {}", name)),

            Expression::UnaryOp { operator, right } => {
                let r = Self::eval_expression_static(*right, env)?;
                match (operator, r) {
                    (UnaryOperator::Minus, SuperValue::Int(n)) => Ok(SuperValue::Int(-n)),
                    (UnaryOperator::Minus, SuperValue::Float(f)) => Ok(SuperValue::Float(-f)),
                    (UnaryOperator::Not, SuperValue::Bool(b)) => Ok(SuperValue::Bool(!b)),
                    _ => Err("Invalid unary operation".into()),
                }
            }

            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let l = Self::eval_expression_static(*left, env)?;
                let r = Self::eval_expression_static(*right, env)?;

                match (l, r, operator.clone()) {
                    // Int operations
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

                    // Float operations
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::Plus) => {
                        Ok(SuperValue::Float(a + b))
                    }
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::Minus) => {
                        Ok(SuperValue::Float(a - b))
                    }
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::Multiply) => {
                        Ok(SuperValue::Float(a * b))
                    }
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::Divide) => {
                        Ok(SuperValue::Float(a / b))
                    }
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::Modulo) => {
                        Ok(SuperValue::Float(a % b))
                    }

                    // Mixed operations (int + float)
                    (SuperValue::Int(a), SuperValue::Float(b), BinaryOperator::Plus) => {
                        Ok(SuperValue::Float(a as f64 + b))
                    }
                    (SuperValue::Float(a), SuperValue::Int(b), BinaryOperator::Plus) => {
                        Ok(SuperValue::Float(a + b as f64))
                    }
                    (SuperValue::Int(a), SuperValue::Float(b), BinaryOperator::Multiply) => {
                        Ok(SuperValue::Float(a as f64 * b))
                    }
                    (SuperValue::Float(a), SuperValue::Int(b), BinaryOperator::Multiply) => {
                        Ok(SuperValue::Float(a * b as f64))
                    }

                    // Comparisons
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

                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::Equal) => {
                        Ok(SuperValue::Bool(a == b))
                    }
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::NotEqual) => {
                        Ok(SuperValue::Bool(a != b))
                    }
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::Less) => {
                        Ok(SuperValue::Bool(a < b))
                    }
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::LessEqual) => {
                        Ok(SuperValue::Bool(a <= b))
                    }
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::Greater) => {
                        Ok(SuperValue::Bool(a > b))
                    }
                    (SuperValue::Float(a), SuperValue::Float(b), BinaryOperator::GreaterEqual) => {
                        Ok(SuperValue::Bool(a >= b))
                    }

                    // String operations
                    (SuperValue::String(a), SuperValue::String(b), BinaryOperator::Plus) => {
                        Ok(SuperValue::String(format!("{}{}", a, b)))
                    }
                    (SuperValue::String(a), SuperValue::String(b), BinaryOperator::Equal) => {
                        Ok(SuperValue::Bool(a == b))
                    }
                    (SuperValue::String(a), SuperValue::String(b), BinaryOperator::NotEqual) => {
                        Ok(SuperValue::Bool(a != b))
                    }

                    // Boolean operations
                    (SuperValue::Bool(a), SuperValue::Bool(b), BinaryOperator::And) => {
                        Ok(SuperValue::Bool(a && b))
                    }
                    (SuperValue::Bool(a), SuperValue::Bool(b), BinaryOperator::Or) => {
                        Ok(SuperValue::Bool(a || b))
                    }

                    _ => Err(format!("Unsupported binary operation: {:?}", operator)),
                }
            }

            Expression::FunctionCall {
                function,
                arguments,
            } => {
                let func = Self::eval_expression_static(*function, env)?;
                let mut args = vec![];
                for a in arguments {
                    args.push(Self::eval_expression_static(a, env)?);
                }

                match func {
                    SuperValue::Function {
                        name: _,
                        parameters,
                        body,
                        return_type,
                        ..
                    } => {
                        if parameters.len() != args.len() {
                            return Err(format!(
                                "Expected {} args, got {}",
                                parameters.len(),
                                args.len()
                            ));
                        }

                        let mut call_env = env.clone().spawn_child();
                        for (i, (param_name, param_type)) in parameters.iter().enumerate() {
                            call_env.define(
                                param_name.clone(),
                                Some(param_type.clone()),
                                args[i].clone(),
                                false,
                            )?;
                        }

                        match Self::eval_statement_static(*body, &mut call_env) {
                            EvalResult::Return(v) | EvalResult::Value(v) => {
                                if v.matches(&return_type) {
                                    Ok(v)
                                } else {
                                    Err(format!(
                                        "Return type mismatch: expected {:?}, got {:?}",
                                        return_type,
                                        v.get_type()
                                    ))
                                }
                            }
                            EvalResult::Error(e) => Err(e),
                            _ => Err("Invalid control flow in function".into()),
                        }
                    }
                    SuperValue::NativeFunction(name) => Self::call_native_function(&name, args),
                    _ => Err("Cannot call non-function value".into()),
                }
            }

            Expression::Lambda {
                parameters,
                return_type,
                body,
            } => Ok(SuperValue::Function {
                name: "<lambda>".to_string(),
                parameters,
                return_type,
                body,
            }),

            Expression::Assignment { left, value } => {
                if let Expression::Identifier(name) = *left {
                    let val = Self::eval_expression_static(*value, env)?;

                    // 🎯 CORREÇÃO 1: Removemos o .clone() para alterar o 'env' real.
                    // 🎯 CORREÇÃO 2: Usamos '&' para converter String em &str.
                    env.assign(&name, val.clone())?;

                    Ok(val)
                } else {
                    Err("Invalid assignment target".into())
                }
            }

            _ => Ok(SuperValue::Void),
        }
    }

    fn call_native_function(name: &str, args: Vec<SuperValue>) -> Result<SuperValue, String> {
        use std::io::{self, Write};

        let get_float = |arg: &SuperValue| -> Result<f64, String> {
            match arg {
                SuperValue::Float(f) => Ok(*f),
                SuperValue::Int(i) => Ok(*i as f64),
                _ => Err(format!("A função '{}' esperava um número.", name)),
            }
        };
        match name {
            // 🖨️ Output: println("Olá", nome)
            "println" | "print" => {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    print!("{}", arg); // Usa o teu Display do types.rs
                }
                if name == "println" {
                    println!();
                }
                let _ = io::stdout().flush();
                Ok(SuperValue::Void)
            }

            // ⌨️ Input de Texto: var s = prompt("Nome: ")
            "prompt" => {
                if let Some(msg) = args.first() {
                    print!("{}", msg);
                    let _ = io::stdout().flush();
                }

                let mut buffer = String::new();
                io::stdin()
                    .read_line(&mut buffer)
                    .map_err(|e| format!("Erro de I/O no Reino: {}", e))?;

                Ok(SuperValue::String(buffer.trim().to_string()))
            }

            // 🔢 Input de Números: var n = prompt_read("Idade: ")
            "prompt_read" => {
                // Reutiliza a lógica do "prompt" acima
                let res = Self::call_native_function("prompt", args)?;
                if let SuperValue::String(s) = res {
                    s.parse::<i64>()
                        .map(SuperValue::Int)
                        .map_err(|_| "Súbdito, isso não é um número inteiro válido!".to_string())
                } else {
                    Err("Falha crítica na leitura de dados".into())
                }
            }

            // 🧹 Utilitários
            "clear" => {
                print!("{}[2J{}[1;1H", 27 as char, 27 as char);
                let _ = io::stdout().flush();
                Ok(SuperValue::Void)
            }

            // --- Potência e Raízes ---
            "sqrt" => Ok(SuperValue::Float(get_float(&args[0])?.sqrt())),
            "cbrt" => Ok(SuperValue::Float(get_float(&args[0])?.cbrt())),
            "pow" => {
                let base = get_float(&args[0])?;
                let exp = get_float(&args[1])?;
                Ok(SuperValue::Float(base.powf(exp)))
            }

            // --- Logaritmos ---
            "ln" => Ok(SuperValue::Float(get_float(&args[0])?.ln())),
            "log10" => Ok(SuperValue::Float(get_float(&args[0])?.log10())),
            "log2" => Ok(SuperValue::Float(get_float(&args[0])?.log2())),
            "log" => {
                let base = get_float(&args[0])?;
                let x = get_float(&args[1])?;
                Ok(SuperValue::Float(x.log(base)))
            }

            // --- Trigonometria ---
            "sin" => Ok(SuperValue::Float(get_float(&args[0])?.sin())),
            "cos" => Ok(SuperValue::Float(get_float(&args[0])?.cos())),
            "tan" => Ok(SuperValue::Float(get_float(&args[0])?.tan())),
            "asin" => Ok(SuperValue::Float(get_float(&args[0])?.asin())),
            "acos" => Ok(SuperValue::Float(get_float(&args[0])?.acos())),
            "atan" => Ok(SuperValue::Float(get_float(&args[0])?.atan())),
            "atan2" => {
                let y = get_float(&args[0])?;
                let x = get_float(&args[1])?;
                Ok(SuperValue::Float(y.atan2(x)))
            }

            // --- Arredondamento e Absoluto ---
            "abs" => Ok(SuperValue::Float(get_float(&args[0])?.abs())),
            "floor" => Ok(SuperValue::Float(get_float(&args[0])?.floor())),
            "ceil" => Ok(SuperValue::Float(get_float(&args[0])?.ceil())),
            "round" => Ok(SuperValue::Float(get_float(&args[0])?.round())),
            "fract" => Ok(SuperValue::Float(get_float(&args[0])?.fract())),

            // --- Exponencial ---
            "exp" => Ok(SuperValue::Float(get_float(&args[0])?.exp())),

            _ => Err(format!("Ministério '{}' não encontrado.", name)),
        }
    }
}
