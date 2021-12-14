use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug)]
pub(crate) enum Item {
    Field(Field),
    Call(Call),
}

#[derive(Debug)]
pub(crate) struct Field {
    pub key: Key,
    pub path: Vec<Accessor>,
    pub value: Option<Value>,
}

#[derive(Debug)]
pub(crate) enum Accessor {
    Dot(String),
    Box(Value),
}

#[derive(Debug)]
pub(crate) enum Value {
    Integer(i64),
    Uid(u64),
    Float(f32),
    Bool(bool),
    String(String),
    Ident(String),
    Call(Call),
    Null,
}

#[derive(Debug)]
pub(crate) struct Call {
    pub name: String,
    pub params: Vec<Value>,
}

#[derive(Debug)]
pub(crate) enum Key {
    NewThing,
    NewRegion,
    Other(String),
}
