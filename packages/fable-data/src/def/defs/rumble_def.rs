use crate::DefStruct;
use crate::def::prelude::*;

/// `CRumbleDef` — C++ `CRumbleDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct RumbleDef {
    // C++: `std::map<EQuakeStrength, float>` / `std::map<EQuakeLength, float>`
    // (tcd_rumble.hpp) — the values are floats, not ints.
    #[def("QuakeIntensities")]
    pub quake_intensities: VecMap<i32, f32>,
    #[def("QuakeDurations")]
    pub quake_durations: VecMap<i32, f32>,
}
