use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `ARMOUR` — C++ `CArmourDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ArmourDef {
        "AugmentationResponse" => pub augmentation_response: BTreeMap<i32, f32>,
        "DamageTypeResponse" => pub damage_type_response: BTreeMap<i32, f32>,
        "ArmourThresholds" => pub armour_thresholds: BTreeMap<i32, f32>,
        "AllHitsKnockdown" => pub all_hits_knockdown: bool,
        "AllHitsNegated" => pub all_hits_negated: bool,
        "AllHitsCauseRecoil" => pub all_hits_cause_recoil: bool,
        "DamageMaterial" => pub damage_material: DefIndex,
        "BlockMaterial" => pub block_material: DefIndex,
    }
}
