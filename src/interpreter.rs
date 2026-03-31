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

fn evaluate_predicate(predicate: &SuperValue, value: SuperValue) -> Result<SuperValue, String> {
    match predicate {
        SuperValue::Function { .. } | SuperValue::NativeFunction(_) => {
            self::evaluate_function(predicate, vec![value])
        }
        _ => Err("Predicado deve ser uma função".into()),
    }
}

fn evaluate_function(func: &SuperValue, args: Vec<SuperValue>) -> Result<SuperValue, String> {
    match func {
        SuperValue::Function {
            parameters, body, ..
        } => {
            // Implementar chamada de função
            // (use sua lógica existente de chamada de função)
            unimplemented!()
        }
        SuperValue::NativeFunction(name) => Interpreter::call_native_function(name, args),
        _ => Err("Não é uma função".into()),
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

        let all_funcs = vec![
            "len",
            "length",
            "push",
            "push_all",
            "insert",
            "unshift",
            "extend",
            "pop",
            "shift",
            "remove",
            "remove_at",
            "clear",
            "contains",
            "index_of",
            "last_index_of",
            "find",
            "find_index",
            "some",
            "every",
            "is_empty",
            "map",
            "filter",
            "reduce",
            "reduce_right",
            "sort",
            "reverse",
            "swap",
            "concat",
            "slice",
            "splice",
            "join",
            "to_string",
            "clone",
            "sum",
            "average",
            "avg",
            "min",
            "max",
            "unique",
            "dedup",
            "flatten",
            "flatten_deep",
            "chunk",
            "for_each",
        ];

        for func in all_funcs {
            globals.define_native(func, func);
        }

        let obj_funcs = vec![
            "keys",
            "values",
            "entries",
            "has_prop",
            "remove_prop",
            "assign",
            "json",
        ];
        for func in obj_funcs {
            globals.define_native(func, func);
        }

        // No teu interpreter.rs, dentro de Interpreter::new()
        let file_functions = vec![
            "file_read",   // Lê todo o conteúdo
            "file_write",  // Sobrescreve/Cria ficheiro
            "file_append", // Adiciona ao fim do ficheiro
            "file_exists", // Verifica se existe
            "file_remove", // Apaga o ficheiro
        ];

        for func in file_functions {
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
                // 🎯 Caso para Incremento/Decremento (Prefix e Postfix)
                if matches!(
                    operator,
                    UnaryOperator::PreIncrement
                        | UnaryOperator::PostIncrement
                        | UnaryOperator::PreDecrement
                        | UnaryOperator::PostDecrement
                ) {
                    if let Expression::Identifier(name) = *right {
                        // 1. Procurar o símbolo na DashMap
                        let mut symbol_entry = match env.symbols.get_mut(&name) {
                            Some(entry) => entry,
                            None => {
                                return Err(format!(
                                    "Runtime Error: Símbolo '{}' não encontrado.",
                                    name
                                )
                                .into());
                            }
                        };

                        // 2. Extrair o valor atual e converter para f64 (Domínio R)
                        let current_val = symbol_entry.value.to_f64()?;

                        let is_inc = matches!(
                            operator,
                            UnaryOperator::PreIncrement | UnaryOperator::PostIncrement
                        );
                        let new_val = if is_inc {
                            current_val + 1.0
                        } else {
                            current_val - 1.0
                        };

                        // 3. Atualizar o valor dentro do Símbolo na DashMap
                        symbol_entry.value = SuperValue::Float(new_val);

                        // 4. Retorno lógico
                        return if matches!(
                            operator,
                            UnaryOperator::PreIncrement | UnaryOperator::PreDecrement
                        ) {
                            Ok(SuperValue::Float(new_val)) // ++x -> Novo
                        } else {
                            Ok(SuperValue::Float(current_val)) // x++ -> Antigo
                        };
                    }
                }

                // 🎯 Caso para Minus e Not (Operações que não alteram a tabela)
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

            Expression::IndexAccess { array, index } => {
                let container = Self::eval_expression_static(*array, env)?;
                let key_val = Self::eval_expression_static(*index, env)?;

                match (container, key_val) {
                    // Suporte para Arrays (já tens)
                    (SuperValue::Array(items), idx_v) => {
                        let idx = idx_v.to_f64()? as usize;
                        if idx < items.len() {
                            Ok(items[idx].clone())
                        } else {
                            Err("Índice fora de limites".into())
                        }
                    }
                    // 🎯 ADICIONA ISTO: Suporte para Objetos pessoa["nome"]
                    (SuperValue::Object(map), SuperValue::String(key)) => {
                        Ok(map.get(&key).cloned().unwrap_or(SuperValue::Null))
                    }
                    _ => Err("Tentativa de indexar tipo inválido".into()),
                }
            }

            Expression::Assignment { left, value } => {
                let new_val = Self::eval_expression_static(*value, env)?;

                match *left {
                    // Caso 1: Atribuição simples (var x = 10)
                    Expression::Identifier(name) => {
                        env.assign(&name, new_val.clone())?;

                        Ok(new_val)
                    }

                    // 🎯 ADICIONA ISTO: Permite fazer pessoa.idade = 30 ou pessoa.falar = fn()...
                    Expression::PropertyAccess { object, property } => {
                        let mut target = Self::eval_expression_static(*object, env)?;

                        if let SuperValue::Object(ref mut map) = target {
                            map.insert(property, new_val.clone());
                            Ok(new_val)
                        } else {
                            Err("Só é possível atribuir propriedades a objetos".into())
                        }
                    }

                    // Caso 2: Atribuição por índice (dados[0] = 12)
                    Expression::IndexAccess { array, index } => {
                        let container = Self::eval_expression_static(*array, env)?;
                        let key_val = Self::eval_expression_static(*index, env)?;

                        match (container, key_val) {
                            // Suporte para Arrays (já tens)
                            (SuperValue::Array(items), idx_v) => {
                                let idx = idx_v.to_f64()? as usize;
                                if idx < items.len() {
                                    Ok(items[idx].clone())
                                } else {
                                    Err("Índice fora de limites".into())
                                }
                            }
                            // 🎯 ADICIONA ISTO: Suporte para Objetos pessoa["nome"]
                            (SuperValue::Object(map), SuperValue::String(key)) => {
                                Ok(map.get(&key).cloned().unwrap_or(SuperValue::Null))
                            }
                            _ => Err("Tentativa de indexar tipo inválido".into()),
                        }
                    }
                    _ => Err("Runtime Error: Invalid assignment target".into()),
                }
            }

            // 🎯 ADICIONA ESTE BLOCO AQUI:
            Expression::PropertyAccess { object, property } => {
                // 1. Resolve o objeto à esquerda do ponto (ex: 'pessoa' em 'pessoa.nome')
                let obj_value = Self::eval_expression_static(*object, env)?;

                match (obj_value, property.as_str()) {
                    // --- 1. Propriedades Nativas de ARRAYS ---
                    (SuperValue::Array(items), "len") | (SuperValue::Array(items), "length") => {
                        Ok(SuperValue::Int(items.len() as i64))
                    }
                    (SuperValue::Array(items), "first") => {
                        Ok(items.first().cloned().unwrap_or(SuperValue::Null))
                    }
                    (SuperValue::Array(items), "last") => {
                        Ok(items.last().cloned().unwrap_or(SuperValue::Null))
                    }

                    // --- 2. Acesso a OBJECTOS (Dicionários HashMap) ---
                    (SuperValue::Object(map), prop_name) => {
                        if let Some(val) = map.get(prop_name) {
                            Ok(val.clone()) // Retorna o valor da chave
                        } else {
                            // Se a propriedade não existe, retornamos Null (comportamento JS)
                            Ok(SuperValue::Null)
                        }
                    }

                    // --- 3. Caso de Erro: Tipo não suporta propriedades ---
                    (val, prop) => Err(format!(
                        "Erro de Membro: O tipo '{:?}' não possui a propriedade '{}'",
                        val.get_type(),
                        prop
                    )),
                }
            }

            // No teu interpreter.rs, dentro de eval_expression_static
            Expression::ObjectLiteral(pairs) => {
                let mut map = std::collections::HashMap::new();
                for (key, value_expr) in pairs {
                    // Avalia cada valor da propriedade recursivamente
                    let val = Self::eval_expression_static(value_expr, env)?;
                    map.insert(key, val);
                }
                Ok(SuperValue::Object(map))
            }

            _ => Ok(SuperValue::Void),
        }
    }

    fn call_native_function(name: &str, mut args: Vec<SuperValue>) -> Result<SuperValue, String> {
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
            "prompt_int" => {
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
            "cls" => {
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

            // Dentro do teu match name em call_native_function
            "file_read" => {
                let path = args[0].as_string()?; // Assume que tens um helper as_string()
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Erro ao ler ficheiro '{}': {}", path, e))?;
                Ok(SuperValue::String(content))
            }

            "file_write" => {
                let path = args[0].as_string()?;
                let content = args[1].as_string()?;
                std::fs::write(&path, content)
                    .map_err(|e| format!("Erro ao escrever em '{}': {}", path, e))?;
                Ok(SuperValue::Void)
            }

            "file_append" => {
                use std::fs::OpenOptions;
                use std::io::Write;

                let path = args[0].as_string()?;
                let content = args[1].as_string()?;

                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .map_err(|e| format!("Erro ao abrir '{}': {}", path, e))?;

                writeln!(file, "{}", content).map_err(|e| e.to_string())?;
                Ok(SuperValue::Void)
            }

            "file_exists" => {
                let path = args[0].as_string()?;
                Ok(SuperValue::Bool(std::path::Path::new(&path).exists()))
            }

            "file_remove" => {
                let path = args[0].as_string()?;
                std::fs::remove_file(&path).map_err(|e| e.to_string())?;
                Ok(SuperValue::Void)
            }

            // ========== FUNÇÕES BÁSICAS ==========
            "len" | "length" => {
                if let SuperValue::Array(items) = &args[0] {
                    Ok(SuperValue::Int(items.len() as i64))
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== ADIÇÃO ==========
            "push" => {
                let mut arr = args[0].clone();
                if let SuperValue::Array(ref mut items) = arr {
                    items.push(args[1].clone());
                    Ok(arr)
                } else {
                    Err("Esperado array".into())
                }
            }

            "push_all" => {
                let mut arr = args[0].clone();

                // 🎯 A CORREÇÃO: Removemos 'ref mut' e mantemos apenas o nome da variável
                // O Rust entende que 'items1' é uma referência mutável para o conteúdo de 'arr'
                if let (SuperValue::Array(items1), SuperValue::Array(items2)) =
                    (&mut arr, args[1].clone())
                {
                    items1.extend(items2); // 🚀 Herança direta da performance do Rust
                    Ok(arr)
                } else {
                    Err("Ambos os argumentos para 'push_all' devem ser arrays".into())
                }
            }

            "insert" => {
                let idx = args[1].to_f64()? as usize;
                let mut arr = args[0].clone();
                if let SuperValue::Array(ref mut items) = arr {
                    if idx <= items.len() {
                        items.insert(idx, args[2].clone());
                        Ok(arr)
                    } else {
                        Err("Índice fora de limites".into())
                    }
                } else {
                    Err("Esperado array".into())
                }
            }

            "unshift" => {
                let mut arr = args[0].clone();
                if let SuperValue::Array(ref mut items) = arr {
                    items.insert(0, args[1].clone());
                    Ok(arr)
                } else {
                    Err("Esperado array".into())
                }
            }

            "extend" => {
                let mut arr1 = args[0].clone();

                // 🎯 A CORREÇÃO: Removemos 'ref mut' e mantemos 'items1'
                // O Rust entende que 'items1' é a referência mutável necessária para o extend
                if let (SuperValue::Array(items1), SuperValue::Array(items2)) =
                    (&mut arr1, args[1].clone())
                {
                    items1.extend(items2); // 🚀 Herança direta da performance do Rust
                    Ok(arr1)
                } else {
                    Err("Ambos os argumentos para 'extend' devem ser arrays".into())
                }
            }

            // ========== REMOÇÃO ==========
            "pop" => {
                if let SuperValue::Array(ref mut items) = args[0] {
                    Ok(items.pop().unwrap_or(SuperValue::Null))
                } else {
                    Err("Esperado array".into())
                }
            }

            "shift" => {
                if let SuperValue::Array(ref mut items) = args[0] {
                    if items.is_empty() {
                        Ok(SuperValue::Null)
                    } else {
                        Ok(items.remove(0))
                    }
                } else {
                    Err("Esperado array".into())
                }
            }

            "remove" => {
                let idx = args[1].to_f64()? as usize;
                if let SuperValue::Array(ref mut items) = args[0] {
                    if idx < items.len() {
                        Ok(items.remove(idx))
                    } else {
                        Err("Índice inválido".into())
                    }
                } else {
                    Err("Esperado array".into())
                }
            }

            "remove_at" => {
                let idx = args[1].to_f64()? as usize;
                let mut arr = args[0].clone();
                if let SuperValue::Array(ref mut items) = arr {
                    if idx < items.len() {
                        items.remove(idx);
                        Ok(arr)
                    } else {
                        Err("Índice inválido".into())
                    }
                } else {
                    Err("Esperado array".into())
                }
            }

            "clear" => {
                if let SuperValue::Array(ref mut items) = args[0] {
                    items.clear();
                    Ok(SuperValue::Void)
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== PESQUISA ==========
            "contains" => {
                if let SuperValue::Array(items) = &args[0] {
                    Ok(SuperValue::Bool(items.contains(&args[1])))
                } else {
                    Err("Esperado array".into())
                }
            }

            "index_of" => {
                if let SuperValue::Array(items) = &args[0] {
                    let pos = items.iter().position(|x| x == &args[1]);
                    Ok(pos
                        .map(|i| SuperValue::Int(i as i64))
                        .unwrap_or(SuperValue::Int(-1)))
                } else {
                    Err("Esperado array".into())
                }
            }

            "last_index_of" => {
                if let SuperValue::Array(items) = &args[0] {
                    let pos = items.iter().rposition(|x| x == &args[1]);
                    Ok(pos
                        .map(|i| SuperValue::Int(i as i64))
                        .unwrap_or(SuperValue::Int(-1)))
                } else {
                    Err("Esperado array".into())
                }
            }

            "find" => {
                if let SuperValue::Array(items) = &args[0] {
                    // Nota: args[1] deve ser uma função de predicado
                    // Por simplicidade, vamos fazer busca linear
                    for item in items {
                        if let SuperValue::Bool(true) = evaluate_predicate(&args[1], item.clone())?
                        {
                            return Ok(item.clone());
                        }
                    }
                    Ok(SuperValue::Null)
                } else {
                    Err("Esperado array".into())
                }
            }

            "find_index" => {
                if let SuperValue::Array(items) = &args[0] {
                    for (i, item) in items.iter().enumerate() {
                        if let SuperValue::Bool(true) = evaluate_predicate(&args[1], item.clone())?
                        {
                            return Ok(SuperValue::Int(i as i64));
                        }
                    }
                    Ok(SuperValue::Int(-1))
                } else {
                    Err("Esperado array".into())
                }
            }

            "some" => {
                if let SuperValue::Array(items) = &args[0] {
                    for item in items {
                        if let SuperValue::Bool(true) = evaluate_predicate(&args[1], item.clone())?
                        {
                            return Ok(SuperValue::Bool(true));
                        }
                    }
                    Ok(SuperValue::Bool(false))
                } else {
                    Err("Esperado array".into())
                }
            }

            "every" => {
                if let SuperValue::Array(items) = &args[0] {
                    for item in items {
                        if let SuperValue::Bool(false) = evaluate_predicate(&args[1], item.clone())?
                        {
                            return Ok(SuperValue::Bool(false));
                        }
                    }
                    Ok(SuperValue::Bool(true))
                } else {
                    Err("Esperado array".into())
                }
            }

            "is_empty" => {
                if let SuperValue::Array(items) = &args[0] {
                    Ok(SuperValue::Bool(items.is_empty()))
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== TRANSFORMAÇÃO ==========
            "map" => {
                if let SuperValue::Array(items) = &args[0] {
                    let mut result = vec![];
                    for item in items {
                        let mapped = evaluate_function(&args[1], vec![item.clone()])?;
                        result.push(mapped);
                    }
                    Ok(SuperValue::Array(result))
                } else {
                    Err("Esperado array".into())
                }
            }

            "filter" => {
                if let SuperValue::Array(items) = &args[0] {
                    let mut result = vec![];
                    for item in items {
                        if let SuperValue::Bool(true) = evaluate_predicate(&args[1], item.clone())?
                        {
                            result.push(item.clone());
                        }
                    }
                    Ok(SuperValue::Array(result))
                } else {
                    Err("Esperado array".into())
                }
            }

            "reduce" => {
                if let SuperValue::Array(items) = &args[0] {
                    let mut accumulator = args[2].clone();
                    for item in items {
                        accumulator = evaluate_function(&args[1], vec![accumulator, item.clone()])?;
                    }
                    Ok(accumulator)
                } else {
                    Err("Esperado array".into())
                }
            }

            "reduce_right" => {
                if let SuperValue::Array(items) = &args[0] {
                    let mut accumulator = args[2].clone();
                    for item in items.iter().rev() {
                        accumulator =
                            self::evaluate_function(&args[1], vec![accumulator, item.clone()])?;
                    }
                    Ok(accumulator)
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== ORDENAÇÃO ==========
            "sort" => {
                let mut arr = args[0].clone();
                if let SuperValue::Array(ref mut items) = arr {
                    if args.len() > 1 {
                        // Com função comparadora
                        items.sort_by(|a, b| {
                            let cmp_result =
                                evaluate_function(&args[1], vec![a.clone(), b.clone()])
                                    .unwrap_or(SuperValue::Int(0));
                            match cmp_result {
                                SuperValue::Int(n) => n.cmp(&0),
                                _ => std::cmp::Ordering::Equal,
                            }
                        });
                    } else {
                        // Ordenação padrão - implementação manual
                        items.sort_by(|a, b| {
                            match (a, b) {
                                (SuperValue::Int(a_val), SuperValue::Int(b_val)) => {
                                    a_val.cmp(b_val)
                                }
                                (SuperValue::Float(a_val), SuperValue::Float(b_val)) => a_val
                                    .partial_cmp(b_val)
                                    .unwrap_or(std::cmp::Ordering::Equal),
                                (SuperValue::Int(a_val), SuperValue::Float(b_val)) => (*a_val
                                    as f64)
                                    .partial_cmp(b_val)
                                    .unwrap_or(std::cmp::Ordering::Equal),
                                (SuperValue::Float(a_val), SuperValue::Int(b_val)) => a_val
                                    .partial_cmp(&(*b_val as f64))
                                    .unwrap_or(std::cmp::Ordering::Equal),
                                (SuperValue::String(a_val), SuperValue::String(b_val)) => {
                                    a_val.cmp(b_val)
                                }
                                (SuperValue::Bool(a_val), SuperValue::Bool(b_val)) => {
                                    a_val.cmp(b_val)
                                }
                                _ => std::cmp::Ordering::Equal, // Tipos diferentes não podem ser comparados
                            }
                        });
                    }
                    Ok(arr)
                } else {
                    Err("Esperado array".into())
                }
            }
            "reverse" => {
                let mut arr = args[0].clone();
                if let SuperValue::Array(ref mut items) = arr {
                    items.reverse();
                    Ok(arr)
                } else {
                    Err("Esperado array".into())
                }
            }

            "swap" => {
                let i = args[1].to_f64()? as usize;
                let j = args[2].to_f64()? as usize;
                let mut arr = args[0].clone();
                if let SuperValue::Array(ref mut items) = arr {
                    if i < items.len() && j < items.len() {
                        items.swap(i, j);
                        Ok(arr)
                    } else {
                        Err("Índices inválidos".into())
                    }
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== CONCATENAÇÃO ==========
            "concat" => {
                // 1. Criamos a cópia mutável do primeiro array
                let mut arr1 = args[0].clone();

                // 2. 🎯 A CORREÇÃO: Removemos 'ref mut' e o '&mut'
                // O Rust entende que se 'arr1' é mutável, o conteúdo extraído também pode ser.
                if let (SuperValue::Array(items1), SuperValue::Array(items2)) =
                    (&mut arr1, args[1].clone())
                {
                    items1.extend(items2); // Herança direta do Vec::extend do Rust
                    Ok(arr1)
                } else {
                    Err("Ambos os argumentos para 'concat' devem ser arrays".into())
                }
            }

            // ========== FATIAMENTO ==========
            "slice" => {
                let start = args[1].to_f64()? as usize;
                let end = if args.len() > 2 {
                    args[2].to_f64()? as usize
                } else {
                    usize::MAX
                };
                if let SuperValue::Array(items) = &args[0] {
                    let start_idx = start.min(items.len());
                    let end_idx = end.min(items.len());
                    let sub = items[start_idx..end_idx].to_vec();
                    Ok(SuperValue::Array(sub))
                } else {
                    Err("Esperado array".into())
                }
            }

            "splice" => {
                let start = args[1].to_f64()? as usize;
                let delete_count = args[2].to_f64()? as usize;
                let mut arr = args[0].clone();

                if let SuperValue::Array(ref mut items) = arr {
                    let start_idx = start.min(items.len());
                    let end_idx = (start_idx + delete_count).min(items.len());

                    // Elementos removidos
                    let removed = items[start_idx..end_idx].to_vec();

                    // Remove os elementos
                    items.drain(start_idx..end_idx);

                    // Insere novos elementos se houver
                    if args.len() > 3 {
                        if let SuperValue::Array(new_items) = &args[3] {
                            for (i, item) in new_items.iter().enumerate() {
                                items.insert(start_idx + i, item.clone());
                            }
                        }
                    }

                    Ok(SuperValue::Array(removed))
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== UTILITÁRIOS ==========
            "join" => {
                let sep = args[1].to_string();
                if let SuperValue::Array(items) = &args[0] {
                    let s = items
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(&sep);
                    Ok(SuperValue::String(s))
                } else {
                    Err("Esperado array".into())
                }
            }

            "to_string" => {
                if let SuperValue::Array(items) = &args[0] {
                    let s = items
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    Ok(SuperValue::String(format!("[{}]", s)))
                } else {
                    Err("Esperado array".into())
                }
            }

            "clone" => {
                if let SuperValue::Array(items) = &args[0] {
                    Ok(SuperValue::Array(items.clone()))
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== ESTATÍSTICAS ==========
            "sum" => {
                if let SuperValue::Array(items) = &args[0] {
                    let mut total = 0.0;
                    for item in items {
                        match item {
                            SuperValue::Int(n) => total += *n as f64,
                            SuperValue::Float(f) => total += *f,
                            _ => {}
                        }
                    }
                    Ok(SuperValue::Float(total))
                } else {
                    Err("Esperado array".into())
                }
            }

            "average" | "avg" => {
                if let SuperValue::Array(items) = &args[0] {
                    if items.is_empty() {
                        return Ok(SuperValue::Float(0.0));
                    }
                    let mut total = 0.0;
                    let mut count = 0;
                    for item in items {
                        match item {
                            SuperValue::Int(n) => {
                                total += *n as f64;
                                count += 1;
                            }
                            SuperValue::Float(f) => {
                                total += *f;
                                count += 1;
                            }
                            _ => {}
                        }
                    }
                    if count > 0 {
                        Ok(SuperValue::Float(total / count as f64))
                    } else {
                        Ok(SuperValue::Float(0.0))
                    }
                } else {
                    Err("Esperado array".into())
                }
            }

            "min" => {
                if let SuperValue::Array(items) = &args[0] {
                    if items.is_empty() {
                        return Ok(SuperValue::Null);
                    }
                    let mut min_val = &items[0];
                    for item in &items[1..] {
                        if item < min_val {
                            min_val = item;
                        }
                    }
                    Ok(min_val.clone())
                } else {
                    Err("Esperado array".into())
                }
            }

            "max" => {
                if let SuperValue::Array(items) = &args[0] {
                    if items.is_empty() {
                        return Ok(SuperValue::Null);
                    }
                    let mut max_val = &items[0];
                    for item in &items[1..] {
                        if item > max_val {
                            max_val = item;
                        }
                    }
                    Ok(max_val.clone())
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== ÚNICOS E DUPLICATAS ==========
            "unique" => {
                if let SuperValue::Array(items) = &args[0] {
                    let mut unique = vec![];
                    for item in items {
                        if !unique.contains(item) {
                            unique.push(item.clone());
                        }
                    }
                    Ok(SuperValue::Array(unique))
                } else {
                    Err("Esperado array".into())
                }
            }

            "dedup" => {
                let mut arr = args[0].clone();
                if let SuperValue::Array(ref mut items) = arr {
                    items.dedup();
                    Ok(arr)
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== ACHATAMENTO ==========
            "flatten" => {
                if let SuperValue::Array(items) = &args[0] {
                    let mut flat = vec![];
                    for item in items {
                        if let SuperValue::Array(sub_items) = item {
                            flat.extend(sub_items.clone());
                        } else {
                            flat.push(item.clone());
                        }
                    }
                    Ok(SuperValue::Array(flat))
                } else {
                    Err("Esperado array".into())
                }
            }

            "flatten_deep" => {
                fn flatten_deep_recursive(value: &SuperValue) -> Vec<SuperValue> {
                    match value {
                        SuperValue::Array(items) => {
                            let mut result = vec![];
                            for item in items {
                                result.extend(flatten_deep_recursive(item));
                            }
                            result
                        }
                        _ => vec![value.clone()],
                    }
                }

                if let SuperValue::Array(items) = &args[0] {
                    let mut flat = vec![];
                    for item in items {
                        flat.extend(flatten_deep_recursive(item));
                    }
                    Ok(SuperValue::Array(flat))
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== AGRUPAMENTO ==========
            "chunk" => {
                let size = args[1].to_f64()? as usize;
                if let SuperValue::Array(items) = &args[0] {
                    let mut chunks = vec![];
                    let mut i = 0;
                    while i < items.len() {
                        let end = (i + size).min(items.len());
                        chunks.push(SuperValue::Array(items[i..end].to_vec()));
                        i += size;
                    }
                    Ok(SuperValue::Array(chunks))
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== ITERAÇÃO ==========
            "for_each" => {
                if let SuperValue::Array(items) = &args[0] {
                    for item in items {
                        evaluate_function(&args[1], vec![item.clone()])?;
                    }
                    Ok(SuperValue::Void)
                } else {
                    Err("Esperado array".into())
                }
            }

            // ========== MANIPULAÇÃO DE OBJECTOS (ESTILO JS) ==========
            "keys" => {
                if let SuperValue::Object(map) = &args[0] {
                    let keys = map
                        .keys()
                        .map(|k| SuperValue::String(k.clone()))
                        .collect::<Vec<_>>();
                    Ok(SuperValue::Array(keys))
                } else {
                    Err("Esperado objeto".into())
                }
            }

            "values" => {
                if let SuperValue::Object(map) = &args[0] {
                    let values = map.values().cloned().collect::<Vec<_>>();
                    Ok(SuperValue::Array(values))
                } else {
                    Err("Esperado objeto".into())
                }
            }

            "has_prop" => {
                let key = args[1].as_string()?;
                if let SuperValue::Object(map) = &args[0] {
                    Ok(SuperValue::Bool(map.contains_key(&key)))
                } else {
                    Err("Esperado objeto".into())
                }
            }

            "remove_prop" => {
                let key = args[1].as_string()?;
                let mut obj = args[0].clone();
                if let SuperValue::Object(ref mut map) = obj {
                    map.remove(&key); // Herança direta: HashMap::remove
                    Ok(obj)
                } else {
                    Err("Esperado objeto".into())
                }
            }

            "assign" => {
                let mut target = args[0].clone(); // 1. Clona o alvo
                let source = args[1].clone(); // 2. Clona a origem

                // Verifica se ambos são objetos antes de fundir
                if let (SuperValue::Object(map_t), SuperValue::Object(map_s)) =
                    (&mut target, source)
                {
                    for (k, v) in map_s {
                        map_t.insert(k, v);
                    }
                    Ok(target)
                } else {
                    // 💡 Dica: Melhora a mensagem para saberes o que falhou
                    Err(format!(
                        "Erro no assign: Esperava objetos, mas recebeu {:?} e {:?}",
                        args[0].get_type(),
                        args[1].get_type()
                    ))
                }
            }

            "json" => {
                // Uma forma simples de ver o objeto formatado
                Ok(SuperValue::String(format!("{}", args[0])))
            }

            _ => Err(format!("Ministério '{}' não encontrado.", name)),
        }
    }
}
