pub mod decode;
pub mod encode;

use crate::shared::InstrValue;

#[derive(Debug,PartialEq)]
pub struct Qst {
    pub body: Vec<InstrValue>
}