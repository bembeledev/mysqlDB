use crate::ast::{BinaryOperator, Expression, Program, SpannedExpression, SpannedStatement, Statement, UnaryOperator};
use crate::core::types::SuperType;
use crate::token::{SpannedToken, Token};

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> Token {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].token.clone()
        } else {
            Token::EOF
        }
    }

    fn current_line(&self) -> usize {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].line
        } else if self.tokens.is_empty() {
            1
        } else {
            self.tokens.last().unwrap().line
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

    fn parse_statement(&mut self) -> Result<SpannedStatement, String> {
        let line = self.current_line();
        let stmt = self.parse_statement_inner()?;
        Ok(SpannedStatement { stmt, line })
    }

    fn parse_statement_inner(&mut self) -> Result<Statement, String> {
        if let Token::Error(e) = self.current() {
            return Err(e);
        }

        while matches!(self.current(), Token::Public | Token::Private | Token::Protected | Token::Static | Token::Final | Token::Override | Token::Abstract) {
            self.advance();
        }

        match self.current() {
            Token::Let | Token::Var => self.parse_variable_declaration(),
            Token::For | Token::While => self.parse_loop(),
            Token::Fn => self.parse_function_declaration(),
            Token::Type => self.parse_type_declaration(),
            Token::Class => self.parse_class_declaration(),
            Token::Interface => self.parse_interface_declaration(),
            Token::Enum => self.parse_enum_declaration(),
            Token::Import => self.parse_import_statement(),
            Token::If => self.parse_if_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Try => self.parse_try_catch(),
            Token::Throw => {
                 self.advance();
                 let expr = self.parse_expression()?;
                 self.expect(Token::Semicolon)?;
                 Ok(Statement::Throw(expr))
            }
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

        // skip Final if present after let
        if self.match_token(Token::Final) {}

        while matches!(self.current(), Token::Final | Token::Static | Token::Public | Token::Private | Token::Protected) {
             self.advance();
        }

        let name = if let Token::Identifier(id) = self.current() {
            self.advance();
            id
        } else if let Token::Final = self.current() {
             self.advance(); // consume misplaced modifier
             if let Token::Identifier(id) = self.current() {
                  self.advance();
                  id
             } else {
                 return Err(format!("Expected identifier, found {:?}", self.current()));
             }
        } else {
            return Err(format!("Expected identifier after let/var, found {:?}", self.current()));
        };

        let type_annotation = if self.match_token(Token::Colon) {
             self.parse_type_annotation()?
        } else {
             SuperType::Any
        };

        let initializer = if self.match_token(Token::Assign) {
             self.parse_expression()?
        } else {
             let line = self.current_line();
             SpannedExpression { expr: Expression::IntLiteral(0), line } // Default simplified
        };

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
            Token::Identifier(_) => {
                // E.g. Container<SuperPagamento>, we accept any class type as Any for simplicity
                self.advance();
                if self.match_token(Token::Less) {
                    self.advance(); // consume inner type generic
                    self.expect(Token::Greater)?;
                }
                Ok(SuperType::Any)
            }
            _ => Err(format!("Expected type annotation, found {:?}", self.current())),
        }
    }

    fn parse_loop(&mut self) -> Result<Statement, String> {
        if self.match_token(Token::While) {
            let condition = self.parse_expression()?;
            let body_line = self.current_line();
            let body = Box::new(SpannedStatement { stmt: self.parse_block()?, line: body_line });
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
                    match self.tokens[self.pos].token {
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
                        let init_line = self.current_line();
                        Some(Box::new(SpannedStatement { stmt: self.parse_statement_inner()?, line: init_line }))
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
                        let inc_line = self.current_line();
                        Some(Box::new(SpannedStatement { stmt: self.parse_expression_or_assignment_statement_no_semi()?, line: inc_line }))
                    } else {
                        None
                    };
                    self.expect(Token::RParen)?;

                    let body_line = self.current_line();
                    let body = Box::new(SpannedStatement { stmt: self.parse_statement_inner()?, line: body_line });
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
                    let body_line = self.current_line();
                    let body = Box::new(SpannedStatement { stmt: self.parse_statement_inner()?, line: body_line });

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
                let body_line = self.current_line();
                let body = Box::new(SpannedStatement { stmt: self.parse_block()?, line: body_line });

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
                // Relax type expectation for JS-style flexibility, but mostly require it:
                let param_type = if self.match_token(Token::Colon) {
                    self.parse_type_annotation()?
                } else {
                    SuperType::Any
                };
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
            if self.match_token(Token::Greater) {
                 self.parse_type_annotation()?
            } else {
                return Err("Expected '->' for return type".to_string());
            }
        } else {
            SuperType::Any
        };

        let body_line = self.current_line();
        let body_stmt = if self.current() == Token::LBrace {
            self.parse_block()?
        } else if self.match_token(Token::Semicolon) {
            // Abstract method (no body)
            Statement::Block(vec![])
        } else {
            return Err(format!("Expected LBrace or Semicolon, found {:?}", self.current()));
        };
        let body = Box::new(SpannedStatement { stmt: body_stmt, line: body_line });

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
        // Abstract check is skipped in parse_statement loop but we could track it. Default to false here for simplicity,
        // unless we peek backward. Let's just say false.
        let is_abstract = false;

        self.expect(Token::Class)?;
        let name = if let Token::Identifier(id) = self.current() {
            self.advance();
            id
        } else {
            return Err("Expected class name".to_string());
        };

        let mut generics = None;
        if self.match_token(Token::Less) {
             if let Token::Identifier(g) = self.current() {
                 generics = Some(vec![g]); // simplified 1 generic support
                 self.advance();
             }
             self.expect(Token::Greater)?;
        }

        let mut extends = None;
        let mut implements = Vec::new();

        while self.current() != Token::LBrace && self.current() != Token::EOF {
             if self.match_token(Token::Extends) {
                 if let Token::Identifier(ex) = self.current() {
                     extends = Some(ex);
                     self.advance();
                 } else { return Err("Expected superclass name".to_string()); }
             } else if self.match_token(Token::Implements) {
                 if let Token::Identifier(imp) = self.current() {
                     implements.push(imp);
                     self.advance();
                 } else { return Err("Expected interface name".to_string()); }
             } else {
                 self.advance(); // Skip unexpected modifiers here, just in case
             }
        }

        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while self.current() != Token::RBrace && self.current() != Token::EOF {
            // skip modifiers again
            while matches!(self.current(), Token::Public | Token::Private | Token::Protected | Token::Static | Token::Final | Token::Override | Token::Abstract) {
                self.advance();
            }

            match self.current() {
                Token::Let | Token::Var => {
                    let is_mutable = if self.match_token(Token::Var) { true } else { self.advance(); false };
                    while matches!(self.current(), Token::Final | Token::Static | Token::Public | Token::Private | Token::Protected) { self.advance(); }

                    let field_name = if let Token::Identifier(id) = self.current() {
                         self.advance(); id
                    } else if self.match_token(Token::Final) {
                         if let Token::Identifier(id) = self.current() { self.advance(); id } else { return Err(format!("Expected field name, found {:?}", self.current())); }
                    } else {
                         return Err(format!("Expected field name, found {:?}", self.current()));
                    };

                    // optional colon and type in classes for simplicity
                    let field_type = if self.match_token(Token::Colon) {
                        self.parse_type_annotation()?
                    } else {
                        SuperType::Any
                    };

                    if self.match_token(Token::Assign) {
                        let _ = self.parse_expression()?; // Skip default values
                    }

                    self.expect(Token::Semicolon)?;
                    fields.push((field_name, field_type, is_mutable));
                }
                Token::Fn => {
                    let line = self.current_line();
                    let stmt = self.parse_function_declaration()?;
                    methods.push(SpannedStatement { stmt, line });
                }
                _ => return Err(format!("Expected field or method declaration in class, found {:?}", self.current())),
            }
        }
        self.expect(Token::RBrace)?;

        Ok(Statement::ClassDeclaration {
            name,
            is_abstract,
            extends,
            implements,
            generics,
            fields,
            methods,
        })
    }

    fn parse_interface_declaration(&mut self) -> Result<Statement, String> {
        self.expect(Token::Interface)?;
        let name = if let Token::Identifier(id) = self.current() {
            self.advance();
            id
        } else {
            return Err("Expected interface name".to_string());
        };
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        while self.current() != Token::RBrace && self.current() != Token::EOF {
             while matches!(self.current(), Token::Public | Token::Private | Token::Protected | Token::Static | Token::Final | Token::Override | Token::Abstract) {
                self.advance();
             }
             if self.match_token(Token::Fn) {
                 let method_name = if let Token::Identifier(id) = self.current() { self.advance(); id } else { return Err("Expected method name".to_string()); };
                 self.expect(Token::LParen)?;
                 // Param parsing skip for now in interfaces
                 while self.current() != Token::RParen && self.current() != Token::EOF { self.advance(); }
                 self.expect(Token::RParen)?;

                 let _return_type = if self.match_token(Token::Minus) {
                     if self.match_token(Token::Greater) { self.parse_type_annotation()? } else { return Err("Expected ->".to_string()); }
                 } else { crate::core::types::SuperType::Void };
                 self.expect(Token::Semicolon)?;

                 let line = self.current_line();
                 methods.push(SpannedStatement {
                     stmt: Statement::FunctionDeclaration { name: method_name, parameters: vec![], return_type: crate::core::types::SuperType::Void, body: Box::new(SpannedStatement { stmt: Statement::Block(vec![]), line }) },
                     line
                 });
             } else {
                 return Err("Interfaces only support method signatures".to_string());
             }
        }
        self.expect(Token::RBrace)?;
        Ok(Statement::InterfaceDeclaration { name, methods })
    }

    fn parse_enum_declaration(&mut self) -> Result<Statement, String> {
        self.expect(Token::Enum)?;
        let name = if let Token::Identifier(id) = self.current() {
            self.advance();
            id
        } else {
            return Err("Expected enum name".to_string());
        };
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        while self.current() != Token::RBrace && self.current() != Token::EOF {
            if let Token::Identifier(v) = self.current() {
                 variants.push(v);
                 self.advance();
                 if self.match_token(Token::Comma) { continue; } else { break; }
            } else {
                 return Err("Expected enum variant".to_string());
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Statement::EnumDeclaration { name, variants })
    }

    fn parse_try_catch(&mut self) -> Result<Statement, String> {
        self.expect(Token::Try)?;
        let try_line = self.current_line();
        let try_block = Box::new(SpannedStatement { stmt: self.parse_block()?, line: try_line });

        self.expect(Token::Catch)?;
        self.expect(Token::LParen)?;

        // Example: catch (e: Exception) or catch (Exception e) - we'll handle e: Exception
        let catch_var = if let Token::Identifier(id) = self.current() { self.advance(); id } else { return Err("Expected catch var".to_string()); };
        self.expect(Token::Colon)?;
        let catch_type = if let Token::Identifier(id) = self.current() { self.advance(); id } else { return Err("Expected catch type".to_string()); };
        self.expect(Token::RParen)?;
        let catch_line = self.current_line();
        let catch_block = Box::new(SpannedStatement { stmt: self.parse_block()?, line: catch_line });

        let finally_block = if self.match_token(Token::Finally) {
             let fin_line = self.current_line();
             Some(Box::new(SpannedStatement { stmt: self.parse_block()?, line: fin_line }))
        } else { None };

        Ok(Statement::TryCatch { try_block, catch_var, catch_type, catch_block, finally_block })
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
        let cons_line = self.current_line();
        let consequence = Box::new(SpannedStatement { stmt: self.parse_block()?, line: cons_line });

        let alternative = if self.match_token(Token::Else) {
            let alt_line = self.current_line();
            if self.current() == Token::If {
                Some(Box::new(SpannedStatement { stmt: self.parse_if_statement()?, line: alt_line }))
            } else {
                Some(Box::new(SpannedStatement { stmt: self.parse_block()?, line: alt_line }))
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
         let expr = self.parse_expression()?;

         if self.match_token(Token::Assign) {
            match expr.expr {
                Expression::Identifier(name) => {
                    let value = self.parse_expression()?;
                    return Ok(Statement::Assignment { name, value });
                }
                Expression::PropertyAccess { object, property } => {
                    let value = self.parse_expression()?;
                    return Ok(Statement::PropertyAssignment { object: *object, property, value });
                }
                _ => return Err("Invalid assignment target".to_string()),
            }
         }

         Ok(Statement::ExpressionStatement(expr))
    }

    fn parse_expression(&mut self) -> Result<SpannedExpression, String> {
        let line = self.current_line();
        let expr = self.parse_equality()?;
        Ok(SpannedExpression { expr, line })
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
            let right_line = self.current_line();
            let right_expr = self.parse_relational()?;
            let right = Box::new(SpannedExpression { expr: right_expr, line: right_line });
            let left_line = self.current_line(); // approximate
            expr = Expression::BinaryOp {
                left: Box::new(SpannedExpression { expr, line: left_line }),
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
            let right_line = self.current_line();
            let right_expr = self.parse_term()?;
            let right = Box::new(SpannedExpression { expr: right_expr, line: right_line });
            let left_line = self.current_line(); // approximate
            expr = Expression::BinaryOp {
                left: Box::new(SpannedExpression { expr, line: left_line }),
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
            let right_line = self.current_line();
            let right_expr = self.parse_factor()?;
            let right = Box::new(SpannedExpression { expr: right_expr, line: right_line });
            let left_line = self.current_line();
            expr = Expression::BinaryOp {
                left: Box::new(SpannedExpression { expr, line: left_line }),
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
            let right_line = self.current_line();
            let right_expr = self.parse_unary()?;
            let right = Box::new(SpannedExpression { expr: right_expr, line: right_line });
            let left_line = self.current_line();
            expr = Expression::BinaryOp {
                left: Box::new(SpannedExpression { expr, line: left_line }),
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
            let right_line = self.current_line();
            let right_expr = self.parse_unary()?;
            let right = Box::new(SpannedExpression { expr: right_expr, line: right_line });
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
                let left_line = self.current_line();
                expr = Expression::FunctionCall {
                    function: Box::new(SpannedExpression { expr, line: left_line }),
                    arguments,
                };
            } else if self.match_token(Token::Dot) {
                let property = if let Token::Identifier(id) = self.current() {
                    self.advance();
                    id
                } else {
                    return Err("Expected property name after '.'".to_string());
                };
                let left_line = self.current_line();
                expr = Expression::PropertyAccess {
                    object: Box::new(SpannedExpression { expr, line: left_line }),
                    property,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        if let Token::PolyglotBlock(lang, content) = self.current() {
            self.advance();
            return Ok(Expression::PolyglotBlock { language: lang, content });
        }

        if self.match_token(Token::New) {
            let class_name = if let Token::Identifier(id) = self.current() {
                self.advance();
                id
            } else {
                return Err("Expected class name after 'new'".to_string());
            };

            // Skip generics in instantiation (e.g. new Container<String>())
            if self.match_token(Token::Less) {
                 if let Token::Identifier(_) = self.current() {
                     self.advance();
                 } else if let Token::TString = self.current() { self.advance(); }
                 self.expect(Token::Greater)?;
            }

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
            Token::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                if self.current() != Token::RBracket {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(Token::Comma) { break; }
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expression::ArrayLiteral(elements))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr.expr)
            }
            _ => Err(format!("Expected expression, found {:?}", self.current())),
        }
    }
}
