use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CRumbleDef` — C++ `CRumbleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct RumbleDef {
        // C++: `std::map<EQuakeStrength, float>` / `std::map<EQuakeLength, float>`
        // (tcd_rumble.hpp) — the values are floats, not ints.
        "QuakeIntensities" => pub quake_intensities: VecMap<i32, f32>,
        "QuakeDurations" => pub quake_durations: VecMap<i32, f32>,
    }
}
