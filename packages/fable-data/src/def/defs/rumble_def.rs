use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CRumbleDef` — C++ `CRumbleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct RumbleDef {
        "QuakeIntensities" => pub quake_intensities: VecMap<i32, i32>,
        "QuakeDurations" => pub quake_durations: VecMap<i32, i32>,
    }
}
