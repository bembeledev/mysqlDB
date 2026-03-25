use std::collections::HashMap;
use crate::ast::Statement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuperType {
    Int,
    Float,
    String,
    Bool,
    Object,
    Void,
    Any,
    Custom(String)
}

#[derive(Debug, Clone, PartialEq)]
pub enum SuperValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Object(HashMap<String, SuperValue>),
    Void,
    /// O segredo para a recursividade:
    /// Transporta o valor de um 'return' e sinaliza a interrupção do bloco.
    ReturnSignal(Box<SuperValue>),
    Function {
        name: String,
        parameters: Vec<(String, SuperType)>,
        return_type: SuperType,
        body: Box<Statement>,
    },
    DataclassConstructor {
        name: String,
        fields: Vec<(String, SuperType)>,
    },
    Class {
        name: String,
        fields: Vec<(String, SuperType, bool)>,
        methods: HashMap<String, SuperValue>,
    },
    NativeFunction(String),
}

impl std::fmt::Display for SuperValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuperValue::Int(n) => write!(f, "{}", n),
            SuperValue::Float(n) => write!(f, "{}", n),
            SuperValue::String(s) => write!(f, "\"{}\"", s),
            SuperValue::Bool(b) => write!(f, "{}", b),
            SuperValue::Void => write!(f, "void"),
            SuperValue::ReturnSignal(val) => write!(f, "return {}", val),
            SuperValue::Function { .. } => write!(f, "[Function]"),
            SuperValue::NativeFunction(name) => write!(f, "[Native Function {}]", name),
            SuperValue::DataclassConstructor { name, .. } => write!(f, "[Dataclass {}]", name),
            SuperValue::Class { name, .. } => write!(f, "[Class {}]", name),
            SuperValue::Object(map) => {
                write!(f, "{{ ")?;
                let mut first = true;
                for (k, v) in map {
                    if !first { write!(f, ", ")?; }
                    write!(f, "{}: {}", k, v)?;
                    first = false;
                }
                write!(f, " }}")
            }
        }
    }
}

impl SuperValue {
    /// Verifica se o valor atual é um sinal de interrupção 'return'
    pub fn is_return(&self) -> bool {
        matches!(self, SuperValue::ReturnSignal(_))
    }

    /// Extrai o valor real de dentro de um ReturnSignal, ou retorna o próprio valor
    pub fn unwrap_return(self) -> Self {
        match self {
            SuperValue::ReturnSignal(val) => *val,
            _ => self,
        }
    }

    pub fn matches(&self, expected_type: &SuperType) -> bool {
        // Se for um sinal de retorno, validamos o valor interno
        if let SuperValue::ReturnSignal(val) = self {
            return val.matches(expected_type);
        }

        match expected_type {
            SuperType::Any => true,
            SuperType::Int => matches!(self, SuperValue::Int(_)),
            SuperType::Float => matches!(self, SuperValue::Float(_)),
            SuperType::String => matches!(self, SuperValue::String(_)),
            SuperType::Bool => matches!(self, SuperValue::Bool(_)),
            SuperType::Object => matches!(self, SuperValue::Object(_)),
            SuperType::Void => matches!(self, SuperValue::Void),
            SuperType::Custom(expected_name) => {
            match self {
                SuperValue::Class { name, .. } => name == expected_name,
                SuperValue::DataclassConstructor { name, .. } => name == expected_name,
                _ => false,
            }
        }
        }
    }

    pub fn get_type(&self) -> SuperType {
        match self {
            SuperValue::Int(_) => SuperType::Int,
            SuperValue::Float(_) => SuperType::Float,
            SuperValue::String(_) => SuperType::String,
            SuperValue::Bool(_) => SuperType::Bool,
            SuperValue::Object(_) => SuperType::Object,
            SuperValue::Void => SuperType::Void,
            SuperValue::ReturnSignal(val) => val.get_type(),
            _ => SuperType::Any,
        }
    }
}