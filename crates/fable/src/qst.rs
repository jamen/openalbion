pub mod decode;
pub mod encode;

use crate::shared::script::InstrValue;

#[derive(Debug,PartialEq)]
pub struct Qst {
    pub body: Vec<InstrValue>
}