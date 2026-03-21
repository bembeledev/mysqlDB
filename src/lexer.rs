use crate::token::{SpannedToken, Token};

pub struct Lexer<'a> {
    input: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            line: 1,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        if c == Some('\n') {
            self.line += 1;
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
                c if c.is_alphabetic() || c == '_' => {
                    let mut ident = String::from(c);
                    ident.push_str(&self.read_identifier());

                    // Check for polyglot block: e.g. `js { ... }`
                    if matches!(ident.as_str(), "js" | "py" | "java" | "cpp" | "cs" | "c" | "php" | "ts") {
                        let mut temp_peek = self.input.clone();
                        // skip whitespace in lookahead
                        while let Some(&nc) = temp_peek.peek() {
                            if nc.is_whitespace() {
                                temp_peek.next();
                            } else {
                                break;
                            }
                        }
                        if temp_peek.peek() == Some(&'{') {
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
                '+' => {
                    if self.peek() == Some(&'+') {
                        self.advance();
                        Token::PlusPlus
                    } else {
                        Token::Plus
                    }
                }
                '-' => {
                    if self.peek() == Some(&'-') {
                        self.advance();
                        Token::MinusMinus
                    } else {
                        Token::Minus
                    }
                }
                '*' => Token::Star,
                '/' => {
                    if self.peek() == Some(&'/') {
                        // Skip inline comment
                        while let Some(&nc) = self.peek() {
                            if nc == '\n' {
                                break;
                            }
                            self.advance();
                        }
                        return self.next_token();
                    } else {
                        Token::Slash
                    }
                }
                '=' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        Token::Equal
                    } else if self.peek() == Some(&'>') {
                        self.advance();
                        Token::Arrow
                    } else {
                        Token::Assign
                    }
                }
                '@' => {
                    let mut ident = String::from("@");
                    ident.push_str(&self.read_identifier());
                    Token::lookup_keyword(&ident)
                }
                '!' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        Token::NotEqual
                    } else {
                        Token::Error("Unexpected character: !".to_string())
                    }
                }
                '<' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        Token::LessEqual
                    } else {
                        Token::Less
                    }
                }
                '>' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        Token::GreaterEqual
                    } else {
                        Token::Greater
                    }
                }
                '(' => Token::LParen,
                ')' => Token::RParen,
                '{' => Token::LBrace,
                '}' => Token::RBrace,
                '[' => Token::LBracket,
                ']' => Token::RBracket,
                ',' => Token::Comma,
                ':' => Token::Colon,
                ';' => Token::Semicolon,
                '.' => Token::Dot,
                _ => Token::Error(format!("Unexpected character: {}", c)),
            }
        } else {
            Token::EOF
        }
    }

    pub fn tokenize(mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();
        loop {
            let line = self.line;
            let token = self.next_token();
            let span = SpannedToken { token: token.clone(), line };
            if token == Token::EOF {
                tokens.push(span);
                break;
            }
            tokens.push(span);
        }
        tokens
    }
}
