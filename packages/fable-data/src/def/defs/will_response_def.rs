use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CWillResponseDef` — C++ `CWillResponseDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct WillResponseDef {
        "IsForcePushable" => pub is_force_pushable: bool,
        "IsLifeDrainable" => pub is_life_drainable: bool,
        "IsLightningable" => pub is_lightningable: bool,
        "IsNotBloodyWillable" => pub is_not_bloody_willable: bool,
        "IsAffectedBySlowTime" => pub is_affected_by_slow_time: bool,
        "IsAbleToBeLifted" => pub is_able_to_be_lifted: bool,
        "IsEpicPowerExplodable" => pub is_epic_power_explodable: bool,
        "IsEpicSpellable" => pub is_epic_spellable: bool,
    }
}
