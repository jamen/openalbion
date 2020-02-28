pub mod decode;
pub mod encode;

use crate::script::Call;

#[derive(Debug,PartialEq)]
pub struct Qst {
    pub body: Vec<Call>
}