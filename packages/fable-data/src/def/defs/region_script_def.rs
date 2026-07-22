use crate::DefStruct;
use crate::def::prelude::*;

/// `CRegionScriptDef` — C++ `CRegionScriptDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct RegionScriptDef {
    #[def("RandomVillagerMax")]
    pub random_villager_max: DefIndex,
    #[def("RandomGuardMax")]
    pub random_guard_max: DefIndex,
    #[def("RandomBanditMax")]
    pub random_bandit_max: DefIndex,
    #[def("RegionDangerLevel")]
    pub region_danger_level: DefIndex,
}
