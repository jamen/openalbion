mod decode;
mod encode;

use crate::script::Field;

#[derive(Debug,PartialEq)]
pub struct Tng {
    pub version: Field,
    pub sections: Vec<TngSection>,
}

#[derive(Debug,PartialEq)]
pub struct TngSection {
    pub section_start: Field,
    pub things: Vec<TngThing>,
}

#[derive(Debug,PartialEq)]
pub struct TngThing {
    pub new_thing: Field,
    pub fields: Vec<Field>,
}