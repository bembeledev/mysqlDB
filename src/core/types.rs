use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuperType {
    Int,
    Float,
    String,
    Bool,
    Object,
    Void,
    Any,
}

use crate::ast::Statement;

#[derive(Debug, Clone, PartialEq)]
pub enum SuperValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Object(HashMap<String, SuperValue>),
    Void,
    Function {
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
        fields: Vec<(String, SuperType, bool)>, // name, type, is_mutable
        methods: std::collections::HashMap<String, SuperValue>, // Map of Function SuperValues
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
            SuperValue::Function { .. } => write!(f, "[Function]"),
            SuperValue::NativeFunction(name) => write!(f, "[Native Function {}]", name),
            SuperValue::DataclassConstructor { name, .. } => write!(f, "[Dataclass {}]", name),
            SuperValue::Class { name, .. } => write!(f, "[Class {}]", name),
            SuperValue::Object(map) => {
                write!(f, "{{ ")?;
                let mut first = true;
                for (k, v) in map {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                    first = false;
                }
                write!(f, " }}")
            }
        }
    }
}

impl SuperValue {
    pub fn matches(&self, expected_type: &SuperType) -> bool {
        match expected_type {
            SuperType::Any => true,
            SuperType::Int => matches!(self, SuperValue::Int(_)),
            SuperType::Float => matches!(self, SuperValue::Float(_)),
            SuperType::String => matches!(self, SuperValue::String(_)),
            SuperType::Bool => matches!(self, SuperValue::Bool(_)),
            SuperType::Object => matches!(self, SuperValue::Object(_)),
            SuperType::Void => matches!(self, SuperValue::Void),
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
            SuperValue::Function { .. } => SuperType::Any, // simplified
            SuperValue::DataclassConstructor { .. } => SuperType::Any,
            SuperValue::Class { .. } => SuperType::Any,
            SuperValue::NativeFunction(_) => SuperType::Any,
        }
    }
}
