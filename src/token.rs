#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub line: usize,
}

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
    Class,
    Import,
    New,
    Public,
    Private,
    Protected,
    Static,
    Final,
    Abstract,
    Interface,
    Extends,
    Implements,
    Override,
    Try,
    Catch,
    Finally,
    Throw,
    Enum,

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
    Arrow, // =>

    // Polyglot Block (language, content)
    PolyglotBlock(String, String),

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

    Error(String),
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
            "class" => Token::Class,
            "import" => Token::Import,
            "new" => Token::New,
            "public" => Token::Public,
            "private" => Token::Private,
            "protected" => Token::Protected,
            "static" => Token::Static,
            "final" => Token::Final,
            "abstract" => Token::Abstract,
            "interface" => Token::Interface,
            "extends" => Token::Extends,
            "implements" => Token::Implements,
            "@Override" => Token::Override,
            "try" => Token::Try,
            "catch" => Token::Catch,
            "finally" => Token::Finally,
            "throw" => Token::Throw,
            "enum" => Token::Enum,
            "true" => Token::BoolLiteral(true),
            "false" => Token::BoolLiteral(false),
            "int" => Token::TInt,
            "float" => Token::TFloat,
            "string" => Token::TString,
            "bool" => Token::TBool,
            "object" => Token::TObject,
            "void" => Token::TVoid,
            "any" => Token::TAny,
            "array" => Token::TAny, // Soft fallback
            "decimal" => Token::TFloat, // map decimal to float
            _ => Token::Identifier(ident.to_string()),
        }
    }
}
