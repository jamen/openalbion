pub mod decode;
pub mod encode;

use crate::script::ScriptCall;

#[derive(Debug,PartialEq)]
pub struct Qst {
    pub body: Vec<ScriptCall>
}