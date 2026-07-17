use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_DRAIN_LIFE_DEF` — C++ `CSpecialAbilitiesDrainLifeDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesDrainLifeDef {
        "AmountOfHealthSucked" => pub amount_of_health_sucked: Vec<f32>,
        "NumEffects" => pub num_effects: Vec<i32>,
        "SpellRange" => pub spell_range: Vec<f32>,
        "ShotName" => pub shot_name: Vec<String>,
        "SpellDelaySecs" => pub spell_delay_secs: f32,
    }
}
