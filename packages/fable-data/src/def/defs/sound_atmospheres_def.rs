use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CSoundAtmospheresDef` — C++ `CSoundAtmospheresDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SoundAtmospheresDef {
        "SoundAtmospheres" => pub sound_atmospheres: Vec<String>,
    }
}
