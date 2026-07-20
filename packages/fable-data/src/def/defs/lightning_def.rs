use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `LIGHTNING` — C++ `CLightningDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct LightningDef {
        "Range" => pub range: f32,
        "LifeTime" => pub life_time: f32,
        "MaxNumberOfBranches" => pub max_number_of_branches: i32,
        "NeedLOSToBranch" => pub need_los_to_branch: bool,
        "DamagePeriod" => pub damage_period: f32,
        "DamagePerCycle" => pub damage_per_cycle: f32,
        "NodeDefIndex" => pub node_def_index: DefIndex,
    }
}
