use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CRegionScriptDef` — C++ `CRegionScriptDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct RegionScriptDef {
        "RandomVillagerMax" => pub random_villager_max: DefIndex,
        "RandomGuardMax" => pub random_guard_max: DefIndex,
        "RandomBanditMax" => pub random_bandit_max: DefIndex,
        "RegionDangerLevel" => pub region_danger_level: DefIndex,
    }
}
