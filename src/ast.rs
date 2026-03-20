use crate::core::types::SuperType;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VariableDeclaration {
        name: String,
        is_mutable: bool,
        type_annotation: SuperType,
        initializer: Expression,
    },
    Assignment {
        name: String,
        value: Expression,
    },
    ExpressionStatement(Expression),
    Block(Vec<Statement>),
    If {
        condition: Expression,
        consequence: Box<Statement>,
        alternative: Option<Box<Statement>>,
    },
    Loop {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        increment: Option<Box<Statement>>,
        iterable: Option<Expression>,
        loop_var: Option<String>,
        body: Box<Statement>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<(String, SuperType)>,
        return_type: SuperType,
        body: Box<Statement>,
    },
    TypeDeclaration {
        name: String,
        is_dataclass: bool,
        fields: Vec<(String, SuperType)>,
    },
    ClassDeclaration {
        name: String,
        fields: Vec<(String, SuperType, bool)>, // name, type, is_mutable
        methods: Vec<Statement>, // Should be FunctionDeclarations
    },
    ImportStatement {
        path: String,
    },
    Return(Option<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Identifier(String),
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        operator: UnaryOperator,
        right: Box<Expression>,
    },
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    PropertyAccess {
        object: Box<Expression>,
        property: String,
    },
    ObjectInstantiation {
        class_name: String,
        arguments: Vec<Expression>,
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
