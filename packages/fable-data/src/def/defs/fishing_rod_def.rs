use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CFishingRodDef` — C++ `CFishingRodDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FishingRodDef {
        "StrainReductionPerSecond" => pub strain_reduction_per_second: f32,
        "DistanceDecreasePerPress" => pub distance_decrease_per_press: f32,
        "PullingDistanceDecreasePerPress" => pub pulling_distance_decrease_per_press: f32,
        "MaxStrain" => pub max_strain: f32,
        "MaxLengthMetres" => pub max_length_metres: f32,
    }
}
