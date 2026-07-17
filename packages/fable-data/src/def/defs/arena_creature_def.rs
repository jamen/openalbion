use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CArenaCreatureDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct ArenaCreatureDef {
        "CreatureType" => pub creature_type: String,
        "NumCreatures" => pub num_creatures: DefIndex,
        "HUDType" => pub hud_type: String,
        "DeathScore" => pub death_score: DefIndex,
    }
}
