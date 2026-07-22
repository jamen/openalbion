use crate::DefStruct;

/// `CSoundAtmospheresDef` — C++ `CSoundAtmospheresDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SoundAtmospheresDef {
    #[def("SoundAtmospheres")]
    pub sound_atmospheres: Vec<String>,
}
