use crate::def_struct;

def_struct! {
    /// C++ `CHeroSoulsCreatureDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroSoulsCreatureDef {
        "CreatureType" => pub creature_type: String,
        "Location" => pub location: String,
        "NumCreatures" => pub num_creatures: i32,
    }
}
