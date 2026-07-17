use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `COpinionDeedReactionDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct OpinionDeedReactionDef {
        "Attitudes" => pub attitudes: Vec<OpinionAttitudeType>,
        "Animation" => pub animation: DefString,
        "DialogueTag" => pub dialogue_tag: u32,
    }
}
