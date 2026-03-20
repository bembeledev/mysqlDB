#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    If,
    Else,
    For,
    While,
    Fn,
    Type,
    In,
    Of,
    As,
    Let,
    Var,
    Return,
    Dataclass,

    // Type Keywords
    TInt,
    TFloat,
    TString,
    TBool,
    TObject,
    TVoid,
    TAny,

    // Identifiers and Literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Assign,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    PlusPlus,
    MinusMinus,

    // Punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Dot,

    EOF,
}

impl Token {
    pub fn lookup_keyword(ident: &str) -> Token {
        match ident {
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "while" => Token::While,
            "fn" => Token::Fn,
            "type" => Token::Type,
            "in" => Token::In,
            "of" => Token::Of,
            "as" => Token::As,
            "let" => Token::Let,
            "var" => Token::Var,
            "return" => Token::Return,
            "dataclass" => Token::Dataclass,
            "true" => Token::BoolLiteral(true),
            "false" => Token::BoolLiteral(false),
            "int" => Token::TInt,
            "float" => Token::TFloat,
            "string" => Token::TString,
            "bool" => Token::TBool,
            "object" => Token::TObject,
            "void" => Token::TVoid,
            "any" => Token::TAny,
            _ => Token::Identifier(ident.to_string()),
        }
    }
}
