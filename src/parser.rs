use colored::Colorize;

use crate::ast::{BinaryOperator, Expression, ImportType, Program, Statement, UnaryOperator};
use crate::core::types::SuperType;
use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ========================
    // CORE HELPERS
    // ========================

    fn current(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF)
    }

    fn consume(&mut self, expected: Token, message: &str) -> Result<Token, String> {
        if self.check(expected.clone()) {
            return Ok(self.advance());
        }

        // Se o token atual não for o esperado, geramos o erro
        let token_atual = self.current();

        // Dica: Como não tens .line, podes imprimir o token_atual
        // para o Joelson saber o que o Parser encontrou de errado.
        Err(format!(
            "{} - Esperava: {:?}. Encontrado: {:?}. Detalhe: {}",
            "Erro de Sintaxe".red().bold(),
            expected,
            token_atual,
            message
        ))
    }

    fn advance(&mut self) -> Token {
        let t = self.current();
        if !self.is_at_end() {
            self.pos += 1;
        }
        t
    }

    fn peek_next(&self) -> Token {
        self.tokens.get(self.pos + 1).cloned().unwrap_or(Token::EOF)
    }

    fn check(&self, token: Token) -> bool {
        std::mem::discriminant(&self.current()) == std::mem::discriminant(&token)
    }

    fn previous(&self) -> Token {
        // Retorna o token que está uma posição atrás do atual
        self.tokens[self.pos - 1].clone()
    }
    /// Consome um token baseado numa string identificadora (ex: "IMPORT", "{", "IDENTIFIER")
    fn peek_check(&self, token: Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        // Olha o token na posição seguinte à atual
        self.tokens[self.pos + 1] == token
    }

    fn match_token(&mut self, token: Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token: Token) -> Result<Token, String> {
        if self.check(token.clone()) {
            let t = self.current();
            self.advance();
            Ok(t)
        } else {
            Err(format!(
                "Expected {:?}, found {:?}, Linha: ",
                token,
                self.current()
            ))
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current(), Token::EOF)
    }

    // ========================
    // ENTRY POINT
    // ========================

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    // ========================
    // STATEMENTS
    // ========================

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current() {
            // 🎯 Suporte para Classes Abstratas ou Normais
            Token::Abstract | Token::Class => self.parse_class_declaration(),

            // 🎯 Suporte para modificadores antes de Variáveis ou Funções
            Token::Public | Token::Private | Token::Protected | Token::Static | Token::Final => {
                // Espreitamos o próximo token para decidir o que fazer
                let next = self.peek_next();
                match next {
                    Token::Function => self.parse_function_declaration(),
                    Token::Let | Token::Var => self.parse_variable_declaration(),
                    _ => self.parse_expression_statement(),
                }
            }

            Token::Let | Token::Var => self.parse_variable_declaration(),
            Token::Function => self.parse_function_declaration(),
            Token::Interface => self.parse_interface_declaration(),
            Token::Enum => self.parse_enum_declaration(),
            Token::Type => self.parse_type_declaration(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_loop(),
            Token::For => self.parse_for_loop(),
            Token::Break => {
                self.advance();
                self.expect(Token::Semicolon)?;
                Ok(Statement::Break)
            }
            Token::Continue => {
                self.advance();
                self.expect(Token::Semicolon)?;
                Ok(Statement::Continue)
            }
            Token::Return => self.parse_return(),
            Token::Throw => self.parse_throw(),
            Token::Try => self.parse_try_catch(),
            Token::Import => self.parse_import(),
            Token::LBrace => self.parse_block(),
            _ => {
                let expr = self.parse_expression()?;
                // 🎯 Consome o ';' apenas se ele existir, para evitar travar o loop
                if self.check(Token::Semicolon) {
                    self.advance();
                }
                Ok(Statement::ExpressionStatement(expr))
            }
        }
    }
    fn parse_variable_declaration(&mut self) -> Result<Statement, String> {
        let is_mutable = self.match_token(Token::Var);
        if !is_mutable {
            self.expect(Token::Let)?;
        }

        let name = if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
            id
        } else {
            unreachable!()
        };

        let type_annotation = if self.match_token(Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(Token::Assign)?;
        let initializer = self.parse_expression()?;
        self.expect(Token::Semicolon)?;

        Ok(Statement::VariableDeclaration {
            name,
            is_mutable,
            type_annotation,
            initializer,
        })
    }

    fn parse_type(&mut self) -> Result<SuperType, String> {
        let t = match self.current() {
            Token::TInt => SuperType::Int,
            Token::TFloat => SuperType::Float,
            Token::TString => SuperType::String,
            Token::TBool => SuperType::Bool,
            Token::TObject => SuperType::Object,
            Token::TVoid => SuperType::Void,
            Token::TAny => SuperType::Any,
            Token::Identifier(id) => SuperType::Custom(id), // Assumindo que SuperType suporte Custom
            _ => return Err("Expected type".into()),
        };
        self.advance();
        Ok(t)
    }

    fn parse_function_declaration(&mut self) -> Result<Statement, String> {
        self.advance(); // Consome 'function' ou 'fn'

        let name = if let Token::Identifier(id) = self.current() {
            self.advance();
            id
        } else {
            "".into()
        };

        self.expect(Token::LParen)?;
        let parameters = self.parse_function_parameters()?;
        self.expect(Token::RParen)?;

        let return_type = if self.match_token(Token::Arrow) {
            self.parse_type()?
        } else {
            SuperType::Void
        };

        // 🎯 A SOLUÇÃO HÍBRIDA:
        // Não usamos match_token(Semicolon) direto, pois ele "rouba" o token.
        let body = if self.check(Token::LBrace) {
            // Se encontrar '{', processa o bloco (Caos: fib, fatorial, etc)
            Box::new(self.parse_block()?)
        } else if self.match_token(Token::Semicolon) {
            // Se encontrar ';', é uma assinatura (Interfaces/Abstract)
            Box::new(Statement::Block(vec![]))
        } else {
            return Err(format!(
                "Esperado '{{' ou ';' na linha {:?}",
                self.current()
            ));
        };

        Ok(Statement::FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
        })
    }
    fn parse_function_parameters(&mut self) -> Result<Vec<(String, SuperType)>, String> {
        let mut params = vec![];
        if !self.check(Token::RParen) {
            loop {
                let name =
                    if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
                        id
                    } else {
                        unreachable!()
                    };
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;
                params.push((name, ty));
                if !self.match_token(Token::Comma) {
                    break;
                }
            }
        }
        Ok(params)
    }

    fn parse_block(&mut self) -> Result<Statement, String> {
        self.expect(Token::LBrace)?;
        let mut stmts = vec![];
        while !self.check(Token::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;
        Ok(Statement::Block(stmts))
    }

    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // if
        self.expect(Token::LParen)?;
        let condition = self.parse_expression()?;
        self.expect(Token::RParen)?;

        let consequence = Box::new(self.parse_statement()?);
        let alternative = if self.match_token(Token::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            consequence,
            alternative,
        })
    }

    fn parse_while_loop(&mut self) -> Result<Statement, String> {
        self.advance(); // while
        self.expect(Token::LParen)?;
        let condition = self.parse_expression()?;
        self.expect(Token::RParen)?;
        let body = Box::new(self.parse_statement()?);
        Ok(Statement::While { condition, body })
    }

    fn parse_for_loop(&mut self) -> Result<Statement, String> {
        self.advance(); // Consome o 'for'

        // 1. PRIMEIRO: Tenta detetar o estilo Python/JS/Rust (sem parênteses)
        if let Token::Identifier(var_name) = self.current() {
            if self.peek_check(Token::In) || self.peek_check(Token::Identifier("in".into())) {
                self.advance(); // Consome a variável
                self.advance(); // Consome o 'in'

                let iterable = self.parse_expression()?;
                let body = Box::new(self.parse_block()?);

                return Ok(Statement::ForIn {
                    variable: var_name,
                    iterable,
                    body,
                });
            }
        }

        // 2. SEGUNDO: Se não for for-in, entra no ESTILO JAVA/PHP/JS CLÁSSICO 🎯
        self.expect(Token::LParen)?;

        let init = if self.match_token(Token::Semicolon) {
            None
        } else {
            let s = self.parse_statement()?;
            // 🔧 AJUSTE: O parse_statement já consumiu o ';' no parse_variable_declaration.
            if self.check(Token::Semicolon) {
                self.advance();
            }
            Some(Box::new(s))
        };

        let condition = if self.match_token(Token::Semicolon) {
            None
        } else {
            let e = self.parse_expression()?;
            // 🔧 AJUSTE: Garante que o ';' da condição é removido antes do incremento.
            if self.check(Token::Semicolon) {
                self.advance();
            } else {
                self.expect(Token::Semicolon)?;
            }
            Some(e)
        };

        let increment = if self.check(Token::RParen) {
            None
        } else {
            let e = self.parse_expression()?;
            Some(Box::new(Statement::ExpressionStatement(e)))
        };

        self.expect(Token::RParen)?;
        let body = Box::new(self.parse_block()?);

        Ok(Statement::For {
            init,
            condition,
            increment,
            body,
        })
    }
    fn parse_class_declaration(&mut self) -> Result<Statement, String> {
        // 1. Assinatura da Classe (Abstract, Name, Extends, Implements)
        let is_abstract = self.match_token(Token::Abstract);
        self.expect(Token::Class)?;

        let class_name = if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
            id
        } else {
            unreachable!()
        };

        let mut extends = None;
        if self.match_token(Token::Extends) {
            if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
                extends = Some(id);
            }
        }

        let mut implements = vec![];
        if self.match_token(Token::Implements) {
            loop {
                if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
                    implements.push(id);
                }
                if !self.match_token(Token::Comma) {
                    break;
                }
            }
        }

        // 1. Inicializamos generics como None (Option<Vec<String>>)
        let mut generics: Option<Vec<String>> = None;

        // 2. Se encontrarmos o '<', transformamos o None em Some(Vec)
        if self.match_token(Token::Less) {
            let mut g_list = vec![];
            loop {
                if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
                    g_list.push(id);
                }

                // Se houver vírgula, continua a ler (ex: <T, U>)
                if !self.match_token(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::Greater)?;
            generics = Some(g_list); // 🎯 RESOLVE: Agora é Option<Vec<String>>
        }

        // 2. Abertura do Corpo da Classe
        self.expect(Token::LBrace)?;
        let mut fields = vec![]; // Agora 'fields' existe ANTES de ser usado
        let mut methods = vec![];

        // 3. Loop de Membros (Variáveis e Funções)
        while !self.check(Token::RBrace) && !self.is_at_end() {
            // Pular modificadores
            while matches!(
                self.current(),
                Token::Public
                    | Token::Private
                    | Token::Protected
                    | Token::Static
                    | Token::Abstract
                    | Token::Override
                    | Token::Final
            ) {
                self.advance();
            }

            if self.check(Token::Function) {
                let method = self.parse_function_declaration()?;
                methods.push(method);
            } else {
                // Lógica para campos (Fields)
                let is_mut = self.match_token(Token::Var);
                if !is_mut {
                    self.expect(Token::Let)?;
                }

                let f_name =
                    if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
                        id
                    } else {
                        unreachable!()
                    };

                self.expect(Token::Colon)?;
                let f_type = self.parse_type()?;

                // 🎯 RESOLVE: Expected Semicolon, found Assign
                if self.match_token(Token::Assign) {
                    self.parse_expression()?; // Consome o valor inicial (ex: = 0)
                }

                self.expect(Token::Semicolon)?;
                fields.push((f_name, f_type, is_mut));
            }
        }

        self.expect(Token::RBrace)?;

        // 4. Retorno Final
        Ok(Statement::ClassDeclaration {
            name: class_name,
            is_abstract,
            extends,
            implements,
            generics,
            fields,
            methods,
        })
    }
    fn parse_try_catch(&mut self) -> Result<Statement, String> {
        self.advance(); // try
        let try_block = Box::new(self.parse_block()?);
        self.expect(Token::Catch)?;
        self.expect(Token::LParen)?;
        let catch_var = if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
            id
        } else {
            unreachable!()
        };
        // Seu AST espera catch_type como String
        let catch_type = if self.match_token(Token::Colon) {
            if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
                id
            } else {
                "Any".into()
            }
        } else {
            "Any".into()
        };
        self.expect(Token::RParen)?;
        let catch_block = Box::new(self.parse_block()?);

        let finally_block = if self.match_token(Token::Finally) {
            Some(Box::new(self.parse_block()?))
        } else {
            None
        };

        Ok(Statement::TryCatch {
            try_block,
            catch_var,
            catch_type,
            catch_block,
            finally_block,
        })
    }

    // ========================
    // EXPRESSIONS (Resumo)
    // ========================

    fn parse_expression_statement(&mut self) -> Result<Statement, String> {
        let expr = self.parse_expression()?;
        self.expect(Token::Semicolon)?;
        Ok(Statement::ExpressionStatement(expr))
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_assignment()
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        // 1. Primeiro verificamos os operadores que vêm ANTES (Prefix)
        if self.match_token(Token::Minus)
            || self.match_token(Token::Bang)
            || self.match_token(Token::PlusPlus)
            || self.match_token(Token::MinusMinus)
        {
            let token = self.previous();
            let operator = match token {
                Token::Minus => UnaryOperator::Minus,
                Token::Bang => UnaryOperator::Not,
                Token::PlusPlus => UnaryOperator::PreIncrement, // ++x
                Token::MinusMinus => UnaryOperator::PreDecrement, // --x
                _ => unreachable!(),
            };

            // Chamada recursiva para permitir coisas como !!true ou -++x
            let right = self.parse_unary()?;

            // Validação: Incremento prefixo exige um Identificador (variável)
            if matches!(
                operator,
                UnaryOperator::PreIncrement | UnaryOperator::PreDecrement
            ) {
                if !matches!(right, Expression::Identifier(_)) {
                    return Err("Erro: Operadores ++ e -- prefixados exigem uma variável.".into());
                }
            }

            return Ok(Expression::UnaryOp {
                operator,
                right: Box::new(right),
            });
        }

        // 2. Se não houver prefixo, lemos o termo base e passamos para o sufixo (Postfix)
        let expr = self.parse_primary()?;
        self.parse_postfix(expr)
    }
    // ✨ NOVO MÉTODO: Trata o que vem DEPOIS da variável

    fn parse_postfix(&mut self, mut expr: Expression) -> Result<Expression, String> {
        loop {
            // 🎯 1. CASO: Acesso a Array (dados[0])
            if self.match_token(Token::LBracket) {
                let index = self.parse_expression()?; // Avalia o que está dentro do [ ]
                self.consume(Token::RBracket, "Esperado ']' após o índice do array.")?;

                // Revolvemos a expressão atual num IndexAccess
                expr = Expression::IndexAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
                continue; // Continua no loop para ver se há mais [ ] ou ++
            }

            // 🎯 2. CASO: Incremento/Decremento (x++ / x--)
            if self.match_token(Token::PlusPlus) || self.match_token(Token::MinusMinus) {
                let token = self.previous();
                let operator = match token {
                    Token::PlusPlus => UnaryOperator::PostIncrement,
                    Token::MinusMinus => UnaryOperator::PostDecrement,
                    _ => unreachable!(),
                };

                // Validamos se o alvo é algo que pode ser incrementado
                if !matches!(
                    expr,
                    Expression::Identifier(_) | Expression::IndexAccess { .. }
                ) {
                    return Err(
                        "Erro: Incremento postfix exige uma variável ou posição de array.".into(),
                    );
                }

                expr = Expression::UnaryOp {
                    operator,
                    right: Box::new(expr),
                };
                continue;
            }

            // Se não houver mais nada, saímos do loop
            break;
        }

        Ok(expr)
    }

    fn parse_assignment(&mut self) -> Result<Expression, String> {
        let expr = self.parse_equality()?;

        // 🎯 Verificamos se o próximo token é um dos nossos operadores de atribuição
        if matches!(
            self.current(),
            Token::Assign
                | Token::PlusAssign
                | Token::MinusAssign
                | Token::StarAssign
                | Token::SlashAssign
                | Token::ModuloAssign
        ) {
            let operator_token = self.advance(); // Consome o +=, -=, etc.
            let value = self.parse_assignment()?; // Recursão para permitir a = b = c += 10

            // Se for um '=' simples, fazemos a atribuição normal
            if operator_token == Token::Assign {
                return Ok(Expression::Assignment {
                    left: Box::new(expr),
                    value: Box::new(value),
                });
            }

            // 🚀 DESUGARING: Se for +=, -=, etc., transformamos em: left = left + value
            let binary_op = match operator_token {
                Token::PlusAssign => BinaryOperator::Plus,
                Token::MinusAssign => BinaryOperator::Minus,
                Token::StarAssign => BinaryOperator::Multiply,
                Token::SlashAssign => BinaryOperator::Divide,
                Token::ModuloAssign => BinaryOperator::Modulo,
                _ => unreachable!(),
            };

            return Ok(Expression::Assignment {
                left: Box::new(expr.clone()), // O alvo da atribuição (ex: dono)
                value: Box::new(Expression::BinaryOp {
                    left: Box::new(expr),   // O valor atual (ex: dono)
                    operator: binary_op,    // A operação (ex: +)
                    right: Box::new(value), // O novo valor (ex: 10)
                }),
            });
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
        // 🎯 Importante: agora chama o nível de comparação abaixo dele
        let mut left = self.parse_comparison()?;

        while matches!(self.current(), Token::EqualEqual | Token::NotEqual) {
            let op = if self.match_token(Token::EqualEqual) {
                BinaryOperator::Equal
            } else {
                self.advance(); // 🎯 O PONTO E VÍRGULA AQUI É O QUE RESOLVE O SEU ERRO!
                BinaryOperator::NotEqual
            };

            let right = self.parse_comparison()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }
    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_term()?;

        while matches!(
            self.current(),
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual
        ) {
            let op_token = self.current(); // 🎯 Captura o token atual
            self.advance(); // 🎯 Avança a posição (sem atribuir)

            let op = match op_token {
                Token::Less => BinaryOperator::Less,
                Token::LessEqual => BinaryOperator::LessEqual,
                Token::Greater => BinaryOperator::Greater,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => unreachable!(),
            };

            let right = self.parse_term()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_factor()?;
        while matches!(self.current(), Token::Plus | Token::Minus) {
            let op = if self.match_token(Token::Plus) {
                BinaryOperator::Plus
            } else {
                self.advance();
                BinaryOperator::Minus
            };
            let right = self.parse_factor()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary()?;
        while matches!(self.current(), Token::Star | Token::Slash | Token::Modulo) {
            let op = if self.match_token(Token::Star) {
                BinaryOperator::Multiply
            } else if self.match_token(Token::Slash) {
                BinaryOperator::Divide
            } else {
                self.advance(); // Consome o Token::Modulo
                BinaryOperator::Modulo
            };
            let right = self.parse_primary()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        // 1. Resolvemos o "átomo" inicial (literal ou identificador)
        let mut expr = match self.current() {
            Token::IntLiteral(n) => {
                self.advance();
                Expression::IntLiteral(n)
            }
            Token::FloatLiteral(n) => {
                self.advance();
                Expression::FloatLiteral(n)
            }
            Token::StringLiteral(s) => {
                self.advance();
                Expression::StringLiteral(s)
            }
            Token::Identifier(id) => {
                self.advance();
                Expression::Identifier(id)
            }
            Token::True => {
                self.advance();
                Expression::BoolLiteral(true)
            }
            Token::False => {
                self.advance();
                Expression::BoolLiteral(false)
            }

            Token::LBrace => {
                self.advance(); // Consome '{'
                let mut pairs = Vec::new();

                if !self.check(Token::RBrace) {
                    loop {
                        // A chave pode ser uma String ou um Identificador (nome da propriedade)
                        let key = match self.advance() {
                            Token::StringLiteral(s) => s,
                            Token::Identifier(id) => id,
                            _ => {
                                return Err(
                                    "Esperada chave (string ou identificador) no objeto".into()
                                );
                            }
                        };

                        self.expect(Token::Colon)?; // Consome ':'
                        let value = self.parse_expression()?; // Lê o valor da propriedade

                        pairs.push((key, value));

                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                }

                self.expect(Token::RBrace)?; // Consome '}'
                Expression::ObjectLiteral(pairs) // Certifica-te que tens este enum no ast.rs
            }

            Token::New => {
                self.advance(); // Consome 'new'

                // Lê o nome da classe (ex: SuperPagamento ou Container)
                let class_name =
                    if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
                        id
                    } else {
                        return Err("Esperado nome da classe após 'new'".into());
                    };

                // Suporte opcional para Generics: <T>
                if self.match_token(Token::Less) {
                    self.parse_type()?; // Consome o tipo dentro do < >
                    self.expect(Token::Greater)?;
                }

                self.expect(Token::LParen)?;
                let mut arguments = vec![];
                if !self.check(Token::RParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                }
                self.expect(Token::RParen)?;

                Expression::ObjectInstantiation {
                    class_name,
                    arguments,
                }
            }

            Token::Function => {
                // Chamamos a função que já existe no seu parser
                let stmt = self.parse_function_declaration()?;

                // Convertemos o Statement para Expression::Lambda usando os dados extraídos
                if let Statement::FunctionDeclaration {
                    parameters,
                    return_type,
                    body,
                    ..
                } = stmt
                {
                    Expression::Lambda {
                        parameters,
                        body,
                        return_type,
                    }
                } else {
                    unreachable!()
                }
            }

            Token::LParen => {
                self.advance();
                let e = self.parse_expression()?;
                self.expect(Token::RParen)?;
                e
            }

            // No teu parser.rs, dentro de parse_primary()
            Token::LBracket => {
                self.advance(); // Consome '['
                let mut elements = vec![];

                if !self.check(Token::RBracket) {
                    loop {
                        // Permite expressões complexas dentro do array: [1+1, get_nome()]
                        elements.push(self.parse_expression()?);

                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                }

                self.expect(Token::RBracket)?; // Consome ']'
                Expression::ArrayLiteral(elements) // Retorna o nó para o AST
            }

            _ => return Err(format!("Unexpected token {:?}", self.current())),
        };

        // 2. Loop de Sufixos: Permite chamadas encadeadas como f(10).prop ou f(10)(20)
        while self.check(Token::LParen) || self.check(Token::Dot) {
            if self.match_token(Token::LParen) {
                let arguments = self.parse_arguments()?; // Resolve o erro de múltiplos argumentos 
                self.expect(Token::RParen)?;
                expr = Expression::FunctionCall {
                    function: Box::new(expr),
                    arguments,
                };
            } else if self.match_token(Token::Dot) {
                let property =
                    if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
                        id
                    } else {
                        unreachable!()
                    };
                expr = Expression::PropertyAccess {
                    object: Box::new(expr),
                    property,
                };
            }
        }

        Ok(expr)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expression>, String> {
        let mut args = vec![];

        if !self.check(Token::RParen) {
            loop {
                args.push(self.parse_expression()?);

                if !self.match_token(Token::Comma) {
                    break;
                }
            }
        }

        Ok(args)
    }

    // Métodos faltantes (Enum, Interface, Import, Return, Throw) seguindo o mesmo padrão...
    fn parse_return(&mut self) -> Result<Statement, String> {
        self.advance();
        let val = if !self.check(Token::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?;
        Ok(Statement::Return(val))
    }

    fn parse_throw(&mut self) -> Result<Statement, String> {
        self.advance();
        let expr = self.parse_expression()?;
        self.expect(Token::Semicolon)?;
        Ok(Statement::Throw(expr))
    }

    fn parse_enum_declaration(&mut self) -> Result<Statement, String> {
        self.advance();
        let name = if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
            id
        } else {
            unreachable!()
        };
        self.expect(Token::LBrace)?;
        let mut variants = vec![];
        while !self.check(Token::RBrace) {
            if let Token::Identifier(v) = self.expect(Token::Identifier("".into()))? {
                variants.push(v);
            }
            if !self.match_token(Token::Comma) {
                break;
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Statement::EnumDeclaration { name, variants })
    }

    fn parse_type_declaration(&mut self) -> Result<Statement, String> {
        self.advance();
        let name = if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
            id
        } else {
            unreachable!()
        };
        self.expect(Token::LBrace)?;
        // Simplificado para Dataclass conforme seu AST
        Ok(Statement::TypeDeclaration {
            name,
            is_dataclass: true,
            fields: vec![],
        })
    }

    fn parse_interface_declaration(&mut self) -> Result<Statement, String> {
        self.advance(); // consome 'interface'
        let name = if let Token::Identifier(id) = self.expect(Token::Identifier("".into()))? {
            id
        } else {
            unreachable!()
        };
        self.expect(Token::LBrace)?;
        let mut methods = vec![];

        while !self.check(Token::RBrace) && !self.is_at_end() {
            // 🎯 AJUSTE: Ignorar modificadores como 'public' antes da função
            while matches!(
                self.current(),
                Token::Public | Token::Private | Token::Protected | Token::Static | Token::Abstract
            ) {
                self.advance();
            }

            if self.check(Token::Function) || self.check(Token::Identifier("fn".into())) {
                methods.push(self.parse_function_declaration()?);
            } else {
                // Se não for função, avança para não entrar em loop infinito
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Statement::InterfaceDeclaration { name, methods })
    }

    fn parse_import(&mut self) -> Result<Statement, String> {
        self.consume(Token::Import, "Esperava 'import'")?;
        self.consume(Token::LBrace, "Esperava '{'")?;

        let mut symbols = Vec::new();
        let mut import_all = false;

        // Lógica para ler dentro das { }
        while self.current() != Token::RBrace && !self.is_at_end() {
            match self.current() {
                Token::Star => {
                    import_all = true;
                    self.advance();
                }
                Token::Identifier(name) => {
                    symbols.push(name);
                    self.advance();
                }
                _ => {
                    return Err(
                        "Token inválido dentro do import. Use '*' ou nomes de símbolos.".into(),
                    );
                }
            }

            // Se houver uma vírgula, consome-a para permitir o próximo símbolo
            if self.current() == Token::Comma {
                self.advance();
            } else if self.current() != Token::RBrace {
                // Se não há vírgula e não é o fim, algo está errado (ex: import { a b })
                return Err("Esperava ',' ou '}' no import.".into());
            }
        }

        self.consume(Token::RBrace, "Esperava '}'")?;
        self.consume(Token::From, "Esperava 'from'")?;

        // 🎯 Captura do Módulo (Caminho ou Nome)
        let module_name = match self.advance() {
        Token::Identifier(name) => name,
        Token::StringLiteral(path) => path,
        _ => return Err("Syntax Error: O nome do módulo deve ser um identificador ou um caminho entre aspas.".into()),
    };

        self.consume(Token::Semicolon, "Faltou o ';' no final do import")?;

        let import_type = if import_all {
            ImportType::Star
        } else {
            ImportType::Symbols(symbols)
        };

        Ok(Statement::Import {
            module: module_name,
            import_type,
        })
    }
}
