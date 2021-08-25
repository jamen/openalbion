pub use crate::tng_parser::TngParser;

#[derive(Debug)]
pub struct Tng {
    pub version: i64,
    pub sections: Vec<TngSection>,
}

#[derive(Debug)]
pub struct TngSection {
    pub name: String,
    pub things: Vec<TngThing>,
}

#[derive(Debug)]
pub enum TngThing {
    Unknown {
        kind: String,
        fields: Vec<(TngKey, TngValue)>,
    },
}

#[derive(Debug)]
pub struct TngKey {
    pub name: String,
    pub accessors: Vec<TngAccessor>,
}

#[derive(Debug)]
pub enum TngAccessor {
    Array(i64),
    Object(String),
}

#[derive(Debug)]
pub enum TngValue {
    Integer(i64),
    Uid(u64),
    Float(f32),
    Bool(bool),
    String(String),
    Ident(String),
    Struct(String, Vec<TngValue>),
    Null,
    Empty,
}
