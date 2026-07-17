use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCreatureModeDef` — C++ `CCreatureModeDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CreatureModeDef {
        "DefaultCreatureMode" => pub default_creature_mode: i32,
        "InitialCreatureMode" => pub initial_creature_mode: i32,
        "DefaultWeaponCreatureMode" => pub default_weapon_creature_mode: i32,
        "AvailableIdleModes" => pub available_idle_modes: Vec<i32>,
    }
}
