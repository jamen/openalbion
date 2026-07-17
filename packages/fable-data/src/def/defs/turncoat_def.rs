use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTurncoatDef` — C++ `CTurncoatDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TurncoatDef {
        "Turncoatable" => pub turncoatable: bool,
        "TurncoatedEffect" => pub turncoated_effect: DefIndex,
        "TurncoatActivateEffect" => pub turncoat_activate_effect: DefIndex,
        "TurncoatDeactivateEffect" => pub turncoat_deactivate_effect: DefIndex,
        "MinimumResistance" => pub minimum_resistance: f32,
        "TimeDelayBeforeResistanceRecoverySecs" => pub time_delay_before_resistance_recovery_secs: f32,
        "ResistanceRecoveryPerSec" => pub resistance_recovery_per_sec: f32,
        "TurncoatStageEffects" => pub turncoat_stage_effects: Vec<i32>,
        "PercentageOfHealthAsResistance" => pub percentage_of_health_as_resistance: f32,
        "ExtraResistance" => pub extra_resistance: f32,
    }
}
