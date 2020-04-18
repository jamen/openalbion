mod decode;
mod encode;

use crate::script::ScriptField;

#[derive(Debug,PartialEq)]
pub struct Tng {
    pub version: ScriptField,
    pub sections: Vec<TngSection>,
}

#[derive(Debug,PartialEq)]
pub struct TngSection {
    pub section_start: ScriptField,
    pub things: Vec<TngThing>,
}

#[derive(Debug,PartialEq)]
pub struct TngThing {
    pub new_thing: ScriptField,
    pub fields: Vec<ScriptField>,
}