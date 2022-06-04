//! Data structures representing an RPC interface.

use std::collections::BTreeMap;

/// Represents the entire RPC interface file.
/// Represented as maps from names to structs/services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcInterface {
    pub structs: BTreeMap<Identifier, Struct>,
    pub services: BTreeMap<Identifier, Service>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    /// Map from field names to field type.
    pub fields: BTreeMap<Identifier, DataType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    /// Map from method name to method type.
    pub methods: BTreeMap<Identifier, Method>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method {
    // Currently only &mut self. &self is not supported.
    pub non_self_params: Vec<(Identifier, DataType)>,
    pub return_type: ReturnType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnType {
    ServiceRefMut(Identifier),
    Data(DataType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    I32,
    Struct(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier(pub String);
