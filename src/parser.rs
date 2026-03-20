use crate::ast::{BinaryOperator, Expression, Program, Statement, UnaryOperator};
use crate::core::types::SuperType;
use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> Token {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].clone()
        } else {
            Token::EOF
        }
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn match_token(&mut self, expected: Token) -> bool {
        if self.current() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.current() == expected {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected token {:?}, found {:?}",
                expected,
                self.current()
            ))
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        while self.current() != Token::EOF {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        if let Token::Error(e) = self.current() {
            return Err(e);
        }

        match self.current() {
            Token::Let | Token::Var => self.parse_variable_declaration(),
            Token::For | Token::While => self.parse_loop(),
            Token::Fn => self.parse_function_declaration(),
            Token::Type => self.parse_type_declaration(),
            Token::Class => self.parse_class_declaration(),
            Token::Import => self.parse_import_statement(),
            Token::If => self.parse_if_statement(),
            Token::Return => self.parse_return_statement(),
            Token::LBrace => self.parse_block(),
            _ => self.parse_expression_or_assignment_statement(),
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, String> {
        let is_mutable = if self.match_token(Token::Var) {
            true
        } else if self.match_token(Token::Let) {
            false
        } else {
            return Err("Expected 'let' or 'var'".to_string());
        };

        let name = if let Token::Identifier(id) = self.current() {
            self.advance();
            id
        } else {
            return Err("Expected identifier after let/var".to_string());
        };

        self.expect(Token::Colon)?;
        let type_annotation = self.parse_type_annotation()?;

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

    fn parse_type_annotation(&mut self) -> Result<SuperType, String> {
        match self.current() {
            Token::TInt => {
                self.advance();
                Ok(SuperType::Int)
            }
            Token::TFloat => {
                self.advance();
                Ok(SuperType::Float)
            }
            Token::TString => {
                self.advance();
                Ok(SuperType::String)
            }
            Token::TBool => {
                self.advance();
                Ok(SuperType::Bool)
            }
            Token::TObject => {
                self.advance();
                Ok(SuperType::Object)
            }
            Token::TVoid => {
                self.advance();
                Ok(SuperType::Void)
            }
            Token::TAny => {
                self.advance();
                Ok(SuperType::Any)
            }
            _ => Err("Expected type annotation".to_string()),
        }
    }

    fn parse_loop(&mut self) -> Result<Statement, String> {
        if self.match_token(Token::While) {
            let condition = self.parse_expression()?;
            let body = Box::new(self.parse_block()?);
            return Ok(Statement::Loop {
                init: None,
                condition: Some(condition),
                increment: None,
                iterable: None,
                loop_var: None,
                body,
            });
        }

        if self.match_token(Token::For) {
            if self.match_token(Token::LParen) {
                // Determine loop style
                let mut is_c_style = false;
                let mut is_js_style = false;

                let start_pos = self.pos;
                let mut semicolons = 0;
                let mut paren_depth = 1;
                while self.pos < self.tokens.len() && paren_depth > 0 {
                    match self.tokens[self.pos] {
                        Token::Semicolon => semicolons += 1,
                        Token::LParen => paren_depth += 1,
                        Token::RParen => paren_depth -= 1,
                        _ => {}
                    }
                    self.pos += 1;
                }

                if semicolons == 2 {
                    is_c_style = true;
                } else {
                    is_js_style = true; // simplifying logic
                }
                self.pos = start_pos;


                if is_c_style {
                    // C/Java style
                    let init = if self.current() != Token::Semicolon {
                        Some(Box::new(self.parse_statement()?)) // parse_statement handles semicolon
                    } else {
                        self.advance();
                        None
                    };

                    let condition = if self.current() != Token::Semicolon {
                        Some(self.parse_expression()?)
                    } else {
                        None
                    };
                    self.expect(Token::Semicolon)?;

                    let increment = if self.current() != Token::RParen {
                        Some(Box::new(self.parse_expression_or_assignment_statement_no_semi()?))
                    } else {
                        None
                    };
                    self.expect(Token::RParen)?;

                    let body = Box::new(self.parse_statement()?);
                    return Ok(Statement::Loop {
                        init,
                        condition,
                        increment,
                        iterable: None,
                        loop_var: None,
                        body,
                    });
                } else if is_js_style {
                     // JS Style `for (let x of lista)`
                    let is_let = self.match_token(Token::Let);
                    let is_var = self.match_token(Token::Var);
                    if !is_let && !is_var {
                         return Err("Expected let or var in for-of loop".to_string());
                    }

                    let loop_var = if let Token::Identifier(id) = self.current() {
                        self.advance();
                        id
                    } else {
                        return Err("Expected identifier in for-of loop".to_string());
                    };

                    self.expect(Token::Of)?;
                    let iterable = self.parse_expression()?;
                    self.expect(Token::RParen)?;
                    let body = Box::new(self.parse_statement()?);

                    return Ok(Statement::Loop {
                        init: None,
                        condition: None,
                        increment: None,
                        iterable: Some(iterable),
                        loop_var: Some(loop_var),
                        body,
                    });
                }
            } else {
                // Python / PHP style
                // for item in lista { ... }
                let loop_var = if let Token::Identifier(id) = self.current() {
                    self.advance();
                    id
                } else {
                    return Err("Expected identifier in for-in loop".to_string());
                };

                self.expect(Token::In)?;
                let iterable = self.parse_expression()?;
                let body = Box::new(self.parse_block()?);

                return Ok(Statement::Loop {
                    init: None,
                    condition: None,
                    increment: None,
                    iterable: Some(iterable),
                    loop_var: Some(loop_var),
                    body,
                });
            }
        }

        Err("Expected loop".to_string())
    }

    fn parse_function_declaration(&mut self) -> Result<Statement, String> {
        self.expect(Token::Fn)?;
        let name = if let Token::Identifier(id) = self.current() {
            self.advance();
            id
        } else {
            return Err("Expected function name".to_string());
        };

        self.expect(Token::LParen)?;
        let mut parameters = Vec::new();
        if self.current() != Token::RParen {
            loop {
                let param_name = if let Token::Identifier(id) = self.current() {
                    self.advance();
                    id
                } else {
                    return Err("Expected parameter name".to_string());
                };
                self.expect(Token::Colon)?;
                let param_type = self.parse_type_annotation()?;
                parameters.push((param_name, param_type));

                if self.match_token(Token::Comma) {
                    continue;
                } else {
                    break;
                }
            }
        }
        self.expect(Token::RParen)?;

        let return_type = if self.match_token(Token::Minus) {
            // Need to handle -> syntax properly, for now, if '-', expect '>'
            if self.match_token(Token::Greater) {
                 self.parse_type_annotation()?
            } else {
                return Err("Expected '->' for return type".to_string());
            }
        } else {
            SuperType::Void
        };

        let body = Box::new(self.parse_block()?);

        Ok(Statement::FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn parse_type_declaration(&mut self) -> Result<Statement, String> {
        self.expect(Token::Type)?;
        let name = if let Token::Identifier(id) = self.current() {
            self.advance();
            id
        } else {
            return Err("Expected type name".to_string());
        };

        let mut is_dataclass = false;
        if self.match_token(Token::LParen) {
            if self.match_token(Token::Dataclass) {
                is_dataclass = true;
            } else {
                return Err("Expected 'dataclass' inside parenthesis for type declaration".to_string());
            }
            self.expect(Token::RParen)?;
        }

        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        while self.current() != Token::RBrace && self.current() != Token::EOF {
            let field_name = if let Token::Identifier(id) = self.current() {
                self.advance();
                id
            } else {
                return Err("Expected field name".to_string());
            };
            self.expect(Token::Colon)?;
            let field_type = self.parse_type_annotation()?;
            self.expect(Token::Semicolon)?;
            fields.push((field_name, field_type));
        }
        self.expect(Token::RBrace)?;

        Ok(Statement::TypeDeclaration {
            name,
            is_dataclass,
            fields,
        })
    }

    fn parse_class_declaration(&mut self) -> Result<Statement, String> {
        self.expect(Token::Class)?;
        let name = if let Token::Identifier(id) = self.current() {
            self.advance();
            id
        } else {
            return Err("Expected class name".to_string());
        };

        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while self.current() != Token::RBrace && self.current() != Token::EOF {
            match self.current() {
                Token::Let | Token::Var => {
                    let is_mutable = if self.match_token(Token::Var) { true } else { self.advance(); false };
                    let field_name = if let Token::Identifier(id) = self.current() { self.advance(); id } else { return Err("Expected field name".to_string()); };
                    self.expect(Token::Colon)?;
                    let field_type = self.parse_type_annotation()?;
                    self.expect(Token::Semicolon)?;
                    fields.push((field_name, field_type, is_mutable));
                }
                Token::Fn => {
                    methods.push(self.parse_function_declaration()?);
                }
                _ => return Err(format!("Expected field or method declaration in class, found {:?}", self.current())),
            }
        }
        self.expect(Token::RBrace)?;

        Ok(Statement::ClassDeclaration {
            name,
            fields,
            methods,
        })
    }

    fn parse_import_statement(&mut self) -> Result<Statement, String> {
        self.expect(Token::Import)?;
        let path = if let Token::StringLiteral(p) = self.current() {
            self.advance();
            p
        } else {
            return Err("Expected string literal for import path".to_string());
        };
        self.expect(Token::Semicolon)?;
        Ok(Statement::ImportStatement { path })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        self.expect(Token::If)?;
        let condition = self.parse_expression()?;
        let consequence = Box::new(self.parse_block()?);

        let alternative = if self.match_token(Token::Else) {
            if self.current() == Token::If {
                Some(Box::new(self.parse_if_statement()?))
            } else {
                Some(Box::new(self.parse_block()?))
            }
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            consequence,
            alternative,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        self.expect(Token::Return)?;
        let value = if self.current() != Token::Semicolon {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?;
        Ok(Statement::Return(value))
    }

    fn parse_block(&mut self) -> Result<Statement, String> {
        self.expect(Token::LBrace)?;
        let mut statements = Vec::new();
        while self.current() != Token::RBrace && self.current() != Token::EOF {
            statements.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;
        Ok(Statement::Block(statements))
    }

    fn parse_expression_or_assignment_statement(&mut self) -> Result<Statement, String> {
        let stmt = self.parse_expression_or_assignment_statement_no_semi()?;
        self.expect(Token::Semicolon)?;
        Ok(stmt)
    }

    fn parse_expression_or_assignment_statement_no_semi(&mut self) -> Result<Statement, String> {
         // This is simplified. True parsing needs to track LHS correctly.
         let expr = self.parse_expression()?;

         if self.match_token(Token::Assign) {
            if let Expression::Identifier(name) = expr {
                let value = self.parse_expression()?;
                return Ok(Statement::Assignment { name, value });
            } else {
                return Err("Invalid assignment target".to_string());
            }
         }

         Ok(Statement::ExpressionStatement(expr))
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_relational()?;

        while matches!(self.current(), Token::Equal | Token::NotEqual) {
            let operator = match self.current() {
                Token::Equal => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = Box::new(self.parse_relational()?);
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn parse_relational(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_term()?;

        while matches!(self.current(), Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual) {
             let operator = match self.current() {
                Token::Less => BinaryOperator::Less,
                Token::LessEqual => BinaryOperator::LessEqual,
                Token::Greater => BinaryOperator::Greater,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = Box::new(self.parse_term()?);
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_factor()?;

        while matches!(self.current(), Token::Plus | Token::Minus) {
             let operator = match self.current() {
                Token::Plus => BinaryOperator::Plus,
                Token::Minus => BinaryOperator::Minus,
                _ => unreachable!(),
            };
            self.advance();
            let right = Box::new(self.parse_factor()?);
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_unary()?;

        while matches!(self.current(), Token::Star | Token::Slash) {
             let operator = match self.current() {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                _ => unreachable!(),
            };
            self.advance();
            let right = Box::new(self.parse_unary()?);
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if matches!(self.current(), Token::Minus) {
            let operator = UnaryOperator::Minus;
            self.advance();
            let right = Box::new(self.parse_unary()?);
            return Ok(Expression::UnaryOp { operator, right });
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(Token::LParen) {
                let mut arguments = Vec::new();
                if self.current() != Token::RParen {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                }
                self.expect(Token::RParen)?;
                expr = Expression::FunctionCall {
                    function: Box::new(expr),
                    arguments,
                };
            } else if self.match_token(Token::Dot) {
                let property = if let Token::Identifier(id) = self.current() {
                    self.advance();
                    id
                } else {
                    return Err("Expected property name after '.'".to_string());
                };
                expr = Expression::PropertyAccess {
                    object: Box::new(expr),
                    property,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        if self.match_token(Token::New) {
            let class_name = if let Token::Identifier(id) = self.current() {
                self.advance();
                id
            } else {
                return Err("Expected class name after 'new'".to_string());
            };

            self.expect(Token::LParen)?;
            let mut arguments = Vec::new();
            if self.current() != Token::RParen {
                loop {
                    arguments.push(self.parse_expression()?);
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
            }
            self.expect(Token::RParen)?;

            return Ok(Expression::ObjectInstantiation {
                class_name,
                arguments,
            });
        }

        match self.current() {
            Token::IntLiteral(n) => {
                self.advance();
                Ok(Expression::IntLiteral(n))
            }
            Token::FloatLiteral(n) => {
                self.advance();
                Ok(Expression::FloatLiteral(n))
            }
            Token::StringLiteral(s) => {
                self.advance();
                Ok(Expression::StringLiteral(s))
            }
            Token::BoolLiteral(b) => {
                self.advance();
                Ok(Expression::BoolLiteral(b))
            }
            Token::Identifier(id) => {
                self.advance();
                Ok(Expression::Identifier(id))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(format!("Expected expression, found {:?}", self.current())),
        }
    }
}
