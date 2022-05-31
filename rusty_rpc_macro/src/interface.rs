//! Data structures representing an RPC interface.

use std::collections::HashMap;

/// Represents the entire RPC interface file.
/// Represented as maps from names to structs/services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcInterface {
    pub structs: HashMap<Identifier, Struct>,
    pub services: HashMap<Identifier, Service>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    /// Map from field names to field type.
    pub fields: HashMap<Identifier, Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    /// Map from method name to method type.
    pub methods: HashMap<Identifier, Method>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method {
    // TODO currently only &self for now. Add &mut self.
    pub non_self_params: Vec<(Identifier, Type)>,
    pub return_type: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    I32,
    Struct(Identifier),
    // TODO Add here:
    // Service(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);
