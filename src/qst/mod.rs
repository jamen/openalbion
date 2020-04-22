mod decode;
mod encode;

use crate::script::ScriptCall;

#[derive(Debug,PartialEq)]
pub struct Qst {
    pub body: Vec<ScriptCall>
}