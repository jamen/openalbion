use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CArenaRoundDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct ArenaRoundDef {
        "NumWaves" => pub num_waves: i32,
        "Waves" => pub waves: Vec<ArenaWaveDef>,
    }
}
