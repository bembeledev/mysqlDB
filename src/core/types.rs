use crate::ast::Statement;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum SuperType {
    Int,
    Float,
    String,
    Bool,
    Object,
    Void,
    Any,
    Array,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SuperValue {
    Any,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Object(HashMap<String, SuperValue>),
    Void,
    Null,

    Array(Vec<SuperValue>),
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
            SuperValue::Any => write!(f, "any"),
            SuperValue::Int(n) => write!(f, "{}", n),
            SuperValue::Float(n) => write!(f, "{}", n),
            SuperValue::String(s) => write!(f, "{}", s),
            SuperValue::Bool(b) => write!(f, "{}", b),
            SuperValue::Void => write!(f, "void"),
            SuperValue::ReturnSignal(val) => write!(f, "return {}", val),
            SuperValue::Array(elements) => {
                write!(f, "[")?;
                for (i, el) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", el)?;
                }
                write!(f, "]")
            }
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
            SuperValue::Null => write!(f, "null"),
        }
    }
}

impl SuperValue {
    /// Verifica se o valor atual é um sinal de interrupção 'return'
    pub fn is_return(&self) -> bool {
        matches!(self, SuperValue::ReturnSignal(_))
    }

    /// Converte o valor atual para f64 se for um tipo numérico.
    /// Essencial para operações no domínio de R (Reais).
    pub fn to_f64(&self) -> Result<f64, String> {
        match self {
            // Se já for Float, apenas extraímos o valor
            SuperValue::Float(f) => Ok(*f),

            // Se for Int, promovemos para Float (Casting)
            SuperValue::Int(i) => Ok(*i as f64),

            // Se tentar converter algo que não é número (ex: String ou Bool)
            _ => Err(format!(
                "Erro de Tipo: Não é possível converter {:?} para um número real (R).",
                self
            )),
        }
    }

    /// Extrai o valor real de dentro de um ReturnSignal, ou retorna o próprio valor
    pub fn unwrap_return(self) -> Self {
        match self {
            SuperValue::ReturnSignal(val) => *val,
            _ => self,
        }
    }

    pub fn as_string(&self) -> Result<String, String> {
        match self {
            SuperValue::String(s) => Ok(s.clone()),
            _ => Err(format!(
                "Esperava uma String, mas encontrou {:?}",
                self.get_type()
            )),
        }
    }

    // Aproveita e adiciona este para as funções matemáticas se ainda não tiveres
    pub fn as_float(&self) -> Result<f64, String> {
        match self {
            SuperValue::Float(f) => Ok(*f),
            SuperValue::Int(i) => Ok(*i as f64),
            _ => Err(format!(
                "Esperava um número, mas encontrou {:?}",
                self.get_type()
            )),
        }
    }

    pub fn matches(&self, expected_type: &SuperType) -> bool {
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
            // 🎯 Adiciona o mapeamento do Array aqui:
            SuperType::Array => matches!(self, SuperValue::Array(_)),
            SuperType::Custom(expected_name) => match self {
                SuperValue::Class { name, .. } => name == expected_name,
                SuperValue::DataclassConstructor { name, .. } => name == expected_name,
                _ => false,
            },
        }
    }

    pub fn get_type(&self) -> SuperType {
        match self {
            // 🎯 Adiciona este caso aqui:
            SuperValue::Any => SuperType::Any,

            SuperValue::Int(_) => SuperType::Int,
            SuperValue::Float(_) => SuperType::Float,
            SuperValue::String(_) => SuperType::String,
            SuperValue::Bool(_) => SuperType::Bool,
            SuperValue::Object(_) => SuperType::Object,
            SuperValue::Array(_) => SuperType::Array,
            SuperValue::Void => SuperType::Void,
            SuperValue::Null => SuperType::Any,
            SuperValue::ReturnSignal(val) => val.get_type(),
            SuperValue::Function { .. } => SuperType::Any,
            SuperValue::NativeFunction(_) => SuperType::Any,
            SuperValue::Class { name, .. } => SuperType::Custom(name.clone()),
            SuperValue::DataclassConstructor { name, .. } => SuperType::Custom(name.clone()),
        }
    }
}

// No ficheiro types.rs

impl PartialOrd for SuperValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            // 1. Comparação Numérica (Int e Float)
            (SuperValue::Int(a), SuperValue::Int(b)) => a.partial_cmp(b),
            (SuperValue::Float(a), SuperValue::Float(b)) => a.partial_cmp(b),
            (SuperValue::Int(a), SuperValue::Float(b)) => (*a as f64).partial_cmp(b),
            (SuperValue::Float(a), SuperValue::Int(b)) => a.partial_cmp(&(*b as f64)),

            // 2. Comparação de Strings
            (SuperValue::String(a), SuperValue::String(b)) => a.partial_cmp(b),

            // 3. Comparação de Bools
            (SuperValue::Bool(a), SuperValue::Bool(b)) => a.partial_cmp(b),

            // 4. Comparação de Arrays (Recursiva)
            (SuperValue::Array(a), SuperValue::Array(b)) => a.partial_cmp(b),

            // 🎯 A SOLUÇÃO PARA O ERRO:
            // Este braço captura o SuperValue::Any, Object, Function, Class, etc.
            // Retornamos None porque esses tipos não possuem uma ordem natural.
            _ => None,
        }
    }
}
