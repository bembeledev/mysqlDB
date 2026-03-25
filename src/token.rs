#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ========================
    // Keywords
    // ========================
    If,
    Else,
    For,
    While,
    Function,
    Type,
    In,
    Of,
    As,
    Let,
    Var,
    Return,
    Break,
    Continue,

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

    // ========================
    // Boolean Keywords (melhor separado)
    // ========================
    True,
    False,

    // ========================
    // Type Keywords
    // ========================
    TInt,
    TFloat,
    TString,
    TBool,
    TObject,
    TVoid,
    TAny,

    // ========================
    // Identifiers & Literals
    // ========================
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),

    // ========================
    // Operators
    // ========================
    Plus,   // +
    Minus,  // -
    Star,   // *
    Slash,  // /
    Modulo, // %

    Assign,       // =
    PlusAssign,   // +=
    MinusAssign,  // -=
    StarAssign,   // *=
    SlashAssign,  // /=
    ModuloAssign, // %=

    EqualEqual, // ==
    NotEqual,   // !=

    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=

    And, // &&
    Or,  // ||
    Not, // !
    Bang,

    PlusPlus,   // ++
    MinusMinus, // --

    Arrow,    // ->
    FatArrow, // =>

    Question,      // ?
    NullCoalesce,  // ??
    OptionalChain, // ?.

    Range,      // ..
    RangeEqual, // ..=

    // ========================
    // Delimiters / Symbols
    // ========================
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

    At,   // @
    Hash, // #
    Pipe, // |

    // ========================
    // Comments (opcional - pode ignorar no lexer)
    // ========================
    LineComment(String),
    BlockComment(String),

    // ========================
    // Polyglot Block
    // ========================
    PolyglotBlock(String, String),

    // ========================
    // Special
    // ========================
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
            "function" => Token::Function,
            "type" => Token::Type,
            "in" => Token::In,
            "of" => Token::Of,
            "as" => Token::As,
            "let" => Token::Let,
            "var" => Token::Var,
            "return" => Token::Return,
            "break" => Token::Break,
            "continue" => Token::Continue,

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

            "true" => Token::True,
            "false" => Token::False,

            "int" => Token::TInt,
            "float" => Token::TFloat,
            "string" => Token::TString,
            "bool" => Token::TBool,
            "object" => Token::TObject,
            "void" => Token::TVoid,
            "any" => Token::TAny,

            // Caso não seja uma keyword/anotação, retorna como Identificador comum
            _ => {
                if ident.starts_with('@') {
                    // Se começar com @ mas não for Override, pode retornar um erro ou um
                    // token de anotação genérica se a sua AST suportar.
                    Token::Error(format!("Anotação desconhecida: {}", ident))
                } else {
                    Token::Identifier(ident.to_string())
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenInfo {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}