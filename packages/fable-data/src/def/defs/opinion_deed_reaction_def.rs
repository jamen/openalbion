use crate::DefStruct;
use crate::def::prelude::*;

/// C++ `COpinionDeedReactionDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct OpinionDeedReactionDef {
    #[def("Attitudes")]
    pub attitudes: Vec<OpinionAttitudeType>,
    #[def("Animation")]
    pub animation: DefString,
    #[def("DialogueTag")]
    pub dialogue_tag: u32,
}
