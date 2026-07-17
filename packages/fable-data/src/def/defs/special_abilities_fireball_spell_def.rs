use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_FIREBALL_SPELL_DEF` — C++ `CSpecialAbilitiesFireballSpellDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesFireballSpellDef {
        "Inclination" => pub inclination: f32,
        "ReleaseDelay" => pub release_delay: f32,
        "StaminaCost" => pub stamina_cost: Vec<f32>,
        "TargettingArcAngle" => pub targetting_arc_angle: f32,
    }
}
