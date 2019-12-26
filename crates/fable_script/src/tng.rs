pub mod decode;
pub mod encode;

use crate::shared::Instr;

#[derive(Debug,PartialEq)]
pub struct Tng {
    pub version: Instr,
    pub sections: Vec<TngSection>,
}

#[derive(Debug,PartialEq)]
pub struct TngSection {
    pub section_start: Instr,
    pub things: Vec<TngThing>,
}

#[derive(Debug,PartialEq)]
pub struct TngThing {
    pub new_thing: Instr,
    pub instrs: Vec<Instr>
    // TODO: Parse instrs more thoroughly into fields.
}