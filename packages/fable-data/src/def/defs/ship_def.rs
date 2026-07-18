use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CShipDef` — C++ `CShipDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ShipDef {
        "SightingFrequency" => pub sighting_frequency: i32 = 1,
        "SightingOffset" => pub sighting_offset: i32,
        "DaysVisible" => pub days_visible: i32 = 1,
        "Swell" => pub swell: f32,
        "Pitch" => pub pitch: f32,
        "Roll" => pub roll: f32,
        "WaveLength" => pub wave_length: f32 = 1.0,
    }
}
