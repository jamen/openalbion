use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CWhisperBattleDef` — C++ `CWhisperBattleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct WhisperBattleDef {
        "NumGetHitsBeforeSommersault" => pub num_get_hits_before_sommersault: i32,
        "NumPhase2Potions" => pub num_phase2_potions: i32,
    }
}
