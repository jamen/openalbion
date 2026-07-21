use crate::def_struct;

def_struct! {
    /// `CWillResponseDef` — C++ `CWillResponseDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct WillResponseDef {
        "IsForcePushable" => pub is_force_pushable: bool = true,
        "IsLifeDrainable" => pub is_life_drainable: bool = true,
        "IsLightningable" => pub is_lightningable: bool = true,
        "IsNotBloodyWillable" => pub is_not_bloody_willable: bool,
        "IsAffectedBySlowTime" => pub is_affected_by_slow_time: bool = true,
        "IsAbleToBeLifted" => pub is_able_to_be_lifted: bool = true,
        "IsEpicPowerExplodable" => pub is_epic_power_explodable: bool = true,
        "IsEpicSpellable" => pub is_epic_spellable: bool = true,
    }
}
