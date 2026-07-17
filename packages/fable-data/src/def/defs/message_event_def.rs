use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `MESSAGE_EVENT` — C++ `CMessageEventDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MessageEventDef {
        "MaxSeenRadius" => pub max_seen_radius: i32,
        "MaxHeardRadius" => pub max_heard_radius: i32,
        "Lifespan" => pub lifespan: i32,
        "Type" => pub type_: MessageEventType,
        "ReactionSpeechType" => pub reaction_speech_type: ReactionSpeechType,
    }
}
