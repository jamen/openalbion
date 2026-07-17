use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SIM_VOICES` — C++ `CSimVoicesDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SimVoicesDef {
        "Entry" => pub entry: SimVoice,
    }
}
