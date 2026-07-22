use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ArenaWaveDef {
    #[def("NumWaveCreatures")]
    pub num_wave_creatures: DefIndex,
    #[def("Creatures")]
    pub creatures: Vec<ArenaCreatureDef>,
    #[def("ShortWave")]
    pub short_wave: bool,
}
