use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CHeroSoulsWaveDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroSoulsWaveDef {
        "NumWaveCreatures" => pub num_wave_creatures: i32,
        "Creatures" => pub creatures: Vec<HeroSoulsCreatureDef>,
    }
}
