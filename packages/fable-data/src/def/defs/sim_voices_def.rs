use crate::DefStruct;
use crate::def::prelude::*;

/// `SIM_VOICES` — C++ `CSimVoicesDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SimVoicesDef {
    #[def("Entry")]
    pub entry: SimVoice,
}
