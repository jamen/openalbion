use crate::DefStruct;

/// `SPECIAL_ABILITIES_HEAL_LIFE_DEF` — C++ `CSpecialAbilitiesHealLifeDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SpecialAbilitiesHealLifeDef {
    #[def("HealthIncreasePerFrame")]
    pub health_increase_per_frame: Vec<f32>,
    #[def("TotalHealthIncrease")]
    pub total_health_increase: Vec<f32>,
    #[def("RadiusOfEffect")]
    pub radius_of_effect: Vec<f32>,
}
