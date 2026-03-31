use crate::token::{Token, TokenInfo};

pub struct Lexer<'a> {
    input: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    fn match_next(&mut self, expected: char) -> bool {
        if self.input.peek() == Some(&expected) {
            self.input.next();
            true
        } else {
            false
        }
    }
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            line: 1,
            column: 1,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        if let Some('\n') = c {
            self.line += 1;
            self.column = 1;
        } else if c.is_some() {
            self.column += 1;
        }
        c
    }

    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn read_string(&mut self) -> String {
        let mut string = String::new();
        while let Some(c) = self.advance() {
            if c == '"' {
                break;
            }
            string.push(c);
        }
        string
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();


        if let Some(c) = self.advance() {
            match c {
                // Identificadores e Keywords
                c if c.is_alphabetic() || c == '_' => {
                    let mut ident = String::from(c);
                    ident.push_str(&self.read_identifier());

                    // Lógica de Bloco Poliglota (mantida a sua)
                    if matches!(ident.as_str(), "js" | "py" | "java" | "cpp" | "cs") {
                        if self.peek() == Some(&'{') {
                            // It's a polyglot block. Read until matched '}'
                            self.skip_whitespace();
                            self.advance(); // consume '{'

                            let mut block_content = String::new();
                            let mut brace_depth = 1;

                            while let Some(&nc) = self.peek() {
                                if nc == '{' {
                                    brace_depth += 1;
                                    block_content.push(nc);
                                    self.advance();
                                } else if nc == '}' {
                                    brace_depth -= 1;
                                    if brace_depth == 0 {
                                        self.advance(); // consume closing '}'
                                        break;
                                    } else {
                                        block_content.push(nc);
                                        self.advance();
                                    }
                                } else {
                                    block_content.push(nc);
                                    self.advance();
                                }
                            }

                            return Token::PolyglotBlock(ident, block_content);
                        }
                    }
                    Token::lookup_keyword(&ident)
                }

                // Números
                c if c.is_ascii_digit() => {
                    let mut num = String::from(c);
                    while let Some(&nc) = self.peek() {
                        if nc.is_ascii_digit() {
                            num.push(nc);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if self.peek() == Some(&'.') {
                        num.push('.');
                        self.advance(); // consume '.'
                        while let Some(&nc) = self.peek() {
                            if nc.is_ascii_digit() {
                                num.push(nc);
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        Token::FloatLiteral(num.parse::<f64>().unwrap())
                    } else {
                        Token::IntLiteral(num.parse::<i64>().unwrap())
                    }
                }

                '"' => Token::StringLiteral(self.read_string()),

                // ========================
                // CORREÇÕES DE OPERADORES
                // ========================
                '=' => {
                    if self.match_next('=') {
                        Token::EqualEqual
                    }
                    // ==
                    else if self.match_next('>') {
                        Token::FatArrow
                    }
                    // =>
                    else {
                        Token::Assign
                    } // =
                }
                '+' => {
                    if self.match_next('+') {
                        Token::PlusPlus
                    }
                    // ++
                    else if self.match_next('=') {
                        Token::PlusAssign
                    }
                    // +=
                    else {
                        Token::Plus
                    } // +
                }
                '-' => {
                    if self.match_next('-') {
                        Token::MinusMinus
                    }
                    // --
                    else if self.match_next('>') {
                        Token::Arrow
                    }
                    // ->
                    else if self.match_next('=') {
                        Token::MinusAssign
                    }
                    // -=
                    else {
                        Token::Minus
                    } // -
                }
                '*' => {
                    if self.match_next('=') {
                        Token::StarAssign
                    } else {
                        Token::Star
                    }
                }
                '/' => {
                    if self.match_next('/') {
                        while let Some(nc) = self.advance() {
                            if nc == '\n' {
                                break;
                            }
                        }
                        self.next_token()
                    } else if self.match_next('=') {
                        Token::SlashAssign
                    } else {
                        Token::Slash
                    }
                }
                '%' => {
                    if self.match_next('=') {
                        Token::ModuloAssign
                    } else {
                        Token::Modulo
                    }
                }
                // Lógica
                '!' => {
                    if self.match_next('=') {
                        Token::NotEqual
                    } else {
                        Token::Not
                    }
                }
                '&' => {
                    if self.match_next('&') {
                        Token::And
                    } else {
                        Token::Error("Expected &".into())
                    }
                }
                '|' => {
                    if self.match_next('|') {
                        Token::Or
                    } else {
                        Token::Pipe
                    }
                }

                '<' => {
                    if self.match_next('=') {
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                }
                '>' => {
                    if self.match_next('=') {
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                }

                // Símbolos
                '(' => Token::LParen,
                ')' => Token::RParen,
                '{' => Token::LBrace,
                '}' => Token::RBrace,
                '[' => Token::LBracket,
                ']' => Token::RBracket,
                ';' => {
                    Token::Semicolon
                }
                ',' => Token::Comma,
                '.' => Token::Dot,
                ':' => Token::Colon,
                '@' => {
                    // Lê o identificador que vem logo após o '@'
                    let mut annotation = String::from("@");
                    annotation.push_str(&self.read_identifier());

                    // O lookup_keyword recebe "@Override" ou "@QualquerCoisa"
                    Token::lookup_keyword(&annotation)
                }
                _ => Token::Error(format!("Unexpected character: {}", c)),
            }
        } else {
            Token::EOF
        }
    }
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::EOF {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    pub fn next_token_info(&mut self) -> TokenInfo {
        self.skip_whitespace();

        // 🎯 Captura a posição exata onde o token começa
        let start_line = self.line;
        let start_column = self.column;

        // Gera o token usando a lógica que já temos
        let token = self.next_token();

        // Retorna a struct preenchida corretamente
        TokenInfo {
            token,
            line: start_line,
            column: start_column,
        }
    }
}
