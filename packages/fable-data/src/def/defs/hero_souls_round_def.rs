use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CHeroSoulsRoundDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroSoulsRoundDef {
        "NumWaves" => pub num_waves: i32,
        "Waves" => pub waves: Vec<HeroSoulsWaveDef>,
    }
}
