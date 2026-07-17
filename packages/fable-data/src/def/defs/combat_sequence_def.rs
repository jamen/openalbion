use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `COMBAT_SEQUENCE` — C++ `CCombatSequenceDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CombatSequenceDef {
        "ValidCombatZones" => pub valid_combat_zones: i32,
        "Type" => pub type_: CombatSequenceType,
        "InterruptionType" => pub interruption_type: CombatSequenceInterruptionType,
        "IsValidType" => pub is_valid_type: CombatSequenceIsValidType,
        "OnStartType" => pub on_start_type: CombatSequenceOnStartModuleType,
        "OnStopType" => pub on_stop_type: CombatSequenceOnStopModuleType,
        "ActionList" => pub action_list: Vec<ActionRegisteredType>,
        "SecondsToPredictTargetPosition" => pub seconds_to_predict_target_position: f32,
        "PreferIfValid" => pub prefer_if_valid: bool,
    }
}
