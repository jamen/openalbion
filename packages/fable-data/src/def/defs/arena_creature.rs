use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ArenaCreatureDef {
    #[def("CreatureType")]
    pub creature_type: String,
    #[def("NumCreatures")]
    pub num_creatures: DefIndex,
    #[def("HUDType")]
    pub hud_type: String,
    #[def("DeathScore")]
    pub death_score: DefIndex,
}
