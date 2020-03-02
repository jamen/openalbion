mod decode;
mod encode;

use crate::script::Call;

#[derive(Debug,PartialEq)]
pub struct Qst {
    pub body: Vec<Call>
}