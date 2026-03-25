use crate::core::types::SuperType;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

// ========================
// STATEMENTS
// ========================
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VariableDeclaration {
        name: String,
        is_mutable: bool,
        type_annotation: Option<SuperType>,
        initializer: Expression,
    },

    ExpressionStatement(Expression),

    Block(Vec<Statement>),

    If {
        condition: Expression,
        consequence: Box<Statement>,
        alternative: Option<Box<Statement>>,
    },

    While {
        condition: Expression,
        body: Box<Statement>,
    },

    For {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        increment: Option<Box<Statement>>,
        body: Box<Statement>,
    },

    ForEach {
        var: String,
        iterable: Expression,
        body: Box<Statement>,
    },

    Break,
    Continue,

    FunctionDeclaration {
        name: String,
        parameters: Vec<(String, SuperType)>,
        return_type: SuperType,
        body: Box<Statement>,
    },

    Return(Option<Expression>),

    // ========================
    // OOP
    // ========================
    ClassDeclaration {
        name: String,
        is_abstract: bool,
        extends: Option<String>,
        implements: Vec<String>,
        generics: Option<Vec<String>>,
        fields: Vec<(String, SuperType, bool)>,
        methods: Vec<Statement>,
    },

    InterfaceDeclaration {
        name: String,
        methods: Vec<Statement>,
    },

    TypeDeclaration {
        name: String,
        is_dataclass: bool,
        fields: Vec<(String, SuperType)>,
    },

    EnumDeclaration {
        name: String,
        variants: Vec<String>,
    },

    // ========================
    // EXCEPTIONS
    // ========================
    TryCatch {
        try_block: Box<Statement>,
        catch_var: String,
        catch_type: String,
        catch_block: Box<Statement>,
        finally_block: Option<Box<Statement>>,
    },

    Throw(Expression),

    // ========================
    // IMPORT
    // ========================
    ImportStatement {
        path: String,
    },
    ForIn { 
        variable: String, 
        iterable: Expression, 
        body: Box<Statement> 
    },
}

// ========================
// EXPRESSIONS
// ========================
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Null,

    // Identifiers
    Identifier(String),

    // ========================
    // OPERATIONS
    // ========================
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },

    UnaryOp {
        operator: UnaryOperator,
        right: Box<Expression>,
    },

    // ========================
    // ASSIGNMENTS
    // ========================
    Assignment {
        left: Box<Expression>,
        value: Box<Expression>,
    },

    CompoundAssignment {
        left: Box<Expression>,
        operator: BinaryOperator,
        value: Box<Expression>,
    },

    MemberAssignment {
        object: Box<Expression>,
        property: String,
        value: Box<Expression>,
    },

    // ========================
    // ACCESS / CALLS
    // ========================
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },

    PropertyAccess {
        object: Box<Expression>,
        property: String,
    },

    IndexAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },

    // ========================
    // OBJECTS / ARRAYS
    // ========================
    ObjectInstantiation {
        class_name: String,
        arguments: Vec<Expression>,
    },

    ArrayLiteral(Vec<Expression>),

    // ========================
    // SPECIAL
    // ========================
    This,
    Super,

    PolyglotBlock {
        language: String,
        content: String,
    },

    // ========================
    // LAMBDA
    // ========================
    Lambda {
        parameters: Vec<(String, SuperType)>,
        body: Box<Statement>,
        return_type: SuperType,
    },
}

// ========================
// OPERATORS
// ========================
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,

    Equal,
    NotEqual,

    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Minus,
    Not,
}