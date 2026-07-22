use crate::DefStruct;
use crate::def::prelude::*;

/// C++ `CArenaRoundDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ArenaRoundDef {
    #[def("NumWaves")]
    pub num_waves: i32,
    #[def("Waves")]
    pub waves: Vec<ArenaWaveDef>,
}
