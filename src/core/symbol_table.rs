use dashmap::DashMap;
use std::sync::Arc;
use crate::core::types::{SuperType, SuperValue};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SuperType,
    pub value: SuperValue,
    pub is_mutable: bool,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    // "Tesouro Real" - Concurrent high performance hash map
    pub symbols: Arc<DashMap<String, Symbol>>,
    pub parent: Option<Box<SymbolTable>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: Arc::new(DashMap::new()),
            parent: None,
        }
    }

    pub fn spawn_child(self) -> Self {
        SymbolTable {
            symbols: Arc::new(DashMap::new()),
            parent: Some(Box::new(self)),
        }
    }

    pub fn kill_child(self) -> Result<Self, String> {
        match self.parent {
            Some(parent) => Ok(*parent),
            None => Err("Cannot kill global scope".to_string()),
        }
    }

    pub fn define(
        &mut self,
        name: String,
        symbol_type: Option<SuperType>,
        value: SuperValue,
        is_mutable: bool
    ) -> Result<(), String> {
        // 1. Verificar se já existe no escopo atual
        if self.symbols.contains_key(&name) {
            return Err(format!("Symbol '{}' already defined in current scope", name));
        }

        // 2. INFERÊNCIA DE TIPO:
        // Se o usuário não deu o tipo, usamos o tipo do valor.
        // Se deu, usamos o tipo fornecido.
        let final_type = match symbol_type {
            Some(t) => {
                // Se o tipo foi fornecido, validamos se o valor bate
                if !value.matches(&t) {
                    return Err(format!(
                        "Type error: Cannot assign {:?} to symbol of type {:?}",
                        value.get_type(), t
                    ));
                }
                t
            },
            None => value.get_type(), // 💡 Inferência mágica acontece aqui!
        };

        // 3. Inserir na tabela usando o tipo resolvido
        self.symbols.insert(name.clone(), Symbol {
            name,
            symbol_type: final_type,
            value,
            is_mutable,
        });

        Ok(())
    }

    pub fn assign(&mut self, name: &str, value: SuperValue) -> Result<(), String> {
        if let Some(mut symbol) = self.symbols.get_mut(name) {
            if !symbol.is_mutable {
                return Err(format!("Cannot reassign immutable symbol '{}'", name));
            }
            if !value.matches(&symbol.symbol_type) {
                return Err(format!("Type error: Cannot assign value of type {:?} to symbol '{}' of type {:?}", value.get_type(), name, symbol.symbol_type));
            }
            symbol.value = value;
            return Ok(());
        }

        if let Some(parent) = &mut self.parent {
            return parent.assign(name, value);
        }

        Err(format!("Symbol '{}' not found", name))
    }

    pub fn lookup(&self, name: &str) -> Option<Symbol> {
        if let Some(symbol) = self.symbols.get(name) {
            return Some(symbol.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.lookup(name);
        }

        None
    }
}
