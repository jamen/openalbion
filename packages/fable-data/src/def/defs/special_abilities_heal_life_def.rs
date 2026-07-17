use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_HEAL_LIFE_DEF` — C++ `CSpecialAbilitiesHealLifeDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesHealLifeDef {
        "HealthIncreasePerFrame" => pub health_increase_per_frame: Vec<f32>,
        "TotalHealthIncrease" => pub total_health_increase: Vec<f32>,
        "RadiusOfEffect" => pub radius_of_effect: Vec<f32>,
    }
}
