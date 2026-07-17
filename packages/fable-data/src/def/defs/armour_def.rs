use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `ARMOUR` — C++ `CArmourDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ArmourDef {
        "AugmentationResponse" => pub augmentation_response: VecMap<f32, Opinion>,
        "DamageTypeResponse" => pub damage_type_response: VecMap<f32, Opinion>,
        "ArmourThresholds" => pub armour_thresholds: VecMap<f32, Opinion>,
        "AllHitsKnockdown" => pub all_hits_knockdown: bool,
        "AllHitsNegated" => pub all_hits_negated: bool,
        "AllHitsCauseRecoil" => pub all_hits_cause_recoil: bool,
        "DamageMaterial" => pub damage_material: i32,
        "BlockMaterial" => pub block_material: i32,
    }
}
