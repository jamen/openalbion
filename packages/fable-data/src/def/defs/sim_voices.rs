use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SimVoicesDef {
    #[def("Entry")]
    pub entry: SimVoice,
}
