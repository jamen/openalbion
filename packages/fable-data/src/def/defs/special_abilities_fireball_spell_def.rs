use crate::DefStruct;

/// `SPECIAL_ABILITIES_FIREBALL_SPELL_DEF` — C++ `CSpecialAbilitiesFireballSpellDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SpecialAbilitiesFireballSpellDef {
    #[def("Inclination")]
    pub inclination: f32,
    #[def("ReleaseDelay")]
    pub release_delay: f32,
    #[def("StaminaCost")]
    pub stamina_cost: Vec<f32>,
    #[def("TargettingArcAngle", default = 100.0)]
    pub targetting_arc_angle: f32,
}
