pub mod decode;
pub mod encode;

pub type Instr = (InstrKey, InstrValue);

#[derive(Debug,PartialEq)]
pub enum InstrKey {
    Name(String),
    Index(u32),
    Property(Vec<InstrKey>),
}

#[derive(Debug,PartialEq)]
pub enum InstrValue {
    None,
    Bool(bool),
    Number(i32),
    BigNumber(u64),
    Float(f32),
    String(String),
    Call((String, Vec<InstrValue>)),
    Name(String),
}