use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_TURNCOAT_SPELL_DEF` — C++ `CSpecialAbilitiesTurncoatSpellDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesTurncoatSpellDef {
        "StaminaCostPerSec" => pub stamina_cost_per_sec: Vec<f32>,
        "ResistanceDamagePerSec" => pub resistance_damage_per_sec: Vec<f32>,
        "TurncoatTime" => pub turncoat_time: Vec<f32>,
        "TurncoatRange" => pub turncoat_range: Vec<f32>,
        "TurncoatXPUpEveryNSecs" => pub turncoat_xp_up_every_n_secs: Vec<f32>,
    }
}
