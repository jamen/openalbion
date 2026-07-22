use crate::DefStruct;
use crate::def::prelude::*;

/// C++ `CHeroSoulsWaveDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct HeroSoulsWaveDef {
    #[def("NumWaveCreatures")]
    pub num_wave_creatures: i32,
    #[def("Creatures")]
    pub creatures: Vec<HeroSoulsCreatureDef>,
}
