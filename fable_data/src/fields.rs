pub use crate::fields_parser::FieldsParser;

use alloc::string::String;
use alloc::vec::Vec;

pub enum Value {
    Integer(i64),
    Uid(u64),
    Float(f32),
    Bool(bool),
    String(String),
    Ident(String),
    Constructor(String, Vec<Value>),
    Null,
}

pub struct Key {
    pub name: String,
    pub accessors: Vec<Accessor>,
}

pub enum Accessor {
    Dot(String),
    Box(i64),
}

pub struct Field {
    pub key: Key,
    pub value: Option<Value>,
}

impl Field {
    // pub fn parse(data: &mut &[u8]) -> Vec<Self> {}
}
