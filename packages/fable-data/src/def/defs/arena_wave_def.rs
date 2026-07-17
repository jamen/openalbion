use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CArenaWaveDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct ArenaWaveDef {
        "NumWaveCreatures" => pub num_wave_creatures: DefIndex,
        "Creatures" => pub creatures: Vec<ArenaCreatureDef>,
        "ShortWave" => pub short_wave: bool,
    }
}
