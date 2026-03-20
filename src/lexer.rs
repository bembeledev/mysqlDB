use crate::token::Token;

pub struct Lexer<'a> {
    input: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.input.next()
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
                '/' => Token::Slash,
                '=' => {
                    if self.peek() == Some(&'=') {
                        self.advance();
                        Token::Equal
                    } else {
                        Token::Assign
                    }
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
}
