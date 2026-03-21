use crate::core::types::SuperType;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<SpannedStatement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedStatement {
    pub stmt: Statement,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedExpression {
    pub expr: Expression,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VariableDeclaration {
        name: String,
        is_mutable: bool,
        type_annotation: SuperType,
        initializer: SpannedExpression,
    },
    Assignment {
        name: String,
        value: SpannedExpression,
    },
    PropertyAssignment {
        object: SpannedExpression,
        property: String,
        value: SpannedExpression,
    },
    ExpressionStatement(SpannedExpression),
    Block(Vec<SpannedStatement>),
    If {
        condition: SpannedExpression,
        consequence: Box<SpannedStatement>,
        alternative: Option<Box<SpannedStatement>>,
    },
    Loop {
        init: Option<Box<SpannedStatement>>,
        condition: Option<SpannedExpression>,
        increment: Option<Box<SpannedStatement>>,
        iterable: Option<SpannedExpression>,
        loop_var: Option<String>,
        body: Box<SpannedStatement>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<(String, SuperType)>,
        return_type: SuperType,
        body: Box<SpannedStatement>,
    },
    TypeDeclaration {
        name: String,
        is_dataclass: bool,
        fields: Vec<(String, SuperType)>,
    },
    ClassDeclaration {
        name: String,
        is_abstract: bool,
        extends: Option<String>,
        implements: Vec<String>,
        generics: Option<Vec<String>>,
        fields: Vec<(String, SuperType, bool)>, // name, type, is_mutable (modifiers stripped for now)
        methods: Vec<SpannedStatement>, // FunctionDeclarations
    },
    InterfaceDeclaration {
        name: String,
        methods: Vec<SpannedStatement>,
    },
    EnumDeclaration {
        name: String,
        variants: Vec<String>,
    },
    TryCatch {
        try_block: Box<SpannedStatement>,
        catch_var: String,
        catch_type: String,
        catch_block: Box<SpannedStatement>,
        finally_block: Option<Box<SpannedStatement>>,
    },
    Throw(SpannedExpression),
    ImportStatement {
        path: String,
    },
    Return(Option<SpannedExpression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Identifier(String),
    BinaryOp {
        left: Box<SpannedExpression>,
        operator: BinaryOperator,
        right: Box<SpannedExpression>,
    },
    UnaryOp {
        operator: UnaryOperator,
        right: Box<SpannedExpression>,
    },
    FunctionCall {
        function: Box<SpannedExpression>,
        arguments: Vec<SpannedExpression>,
    },
    PropertyAccess {
        object: Box<SpannedExpression>,
        property: String,
    },
    ObjectInstantiation {
        class_name: String,
        arguments: Vec<SpannedExpression>,
    },
    ArrayLiteral(Vec<SpannedExpression>),
    PolyglotBlock {
        language: String,
        content: String,
    },
    Lambda {
        parameters: Vec<(String, SuperType)>,
        body: Box<SpannedStatement>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Minus,
    Not,
}
