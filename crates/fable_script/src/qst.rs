pub mod decode;
pub mod encode;

use crate::script::InstrValue;

#[derive(Debug,PartialEq)]
pub struct Qst {
    pub body: Vec<InstrValue>
}