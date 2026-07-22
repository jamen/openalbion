use crate::DefStruct;

/// C++ `CHeroSoulsCreatureDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct HeroSoulsCreatureDef {
    #[def("CreatureType")]
    pub creature_type: String,
    #[def("Location")]
    pub location: String,
    #[def("NumCreatures")]
    pub num_creatures: i32,
}
