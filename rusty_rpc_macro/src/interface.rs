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
    pub fields: HashMap<Identifier, DataType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    /// Map from method name to method type.
    pub methods: HashMap<Identifier, Method>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method {
    // Currently only &mut self. &self is not supported.
    pub non_self_params: Vec<(Identifier, DataType)>,
    pub return_type: ReturnType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReturnType {
    ServiceRefMut(Identifier),
    Data(DataType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataType {
    I32,
    Struct(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(pub String);
