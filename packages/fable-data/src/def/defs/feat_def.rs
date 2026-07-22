use crate::DefStruct;
use crate::def::prelude::*;

/// C++ `CFeatDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct FeatDef {
    #[def("FeatName")]
    pub feat_name: DefString,
    #[def("Verb")]
    pub verb: DefString,
    #[def("TimeLimit")]
    pub time_limit: f32,
    #[def("TargetNumber")]
    pub target_number: DefIndex,
    #[def("GoldReward")]
    pub gold_reward: DefIndex,
    #[def("XPReward")]
    pub xp_reward: DefIndex,
    #[def("ItemReward")]
    pub item_reward: DefString,
    #[def("NoBlocking")]
    pub no_blocking: bool,
    #[def("KN_AttackType")]
    pub kn_attack_type: FeatAttackType,
    #[def("KN_Perfect")]
    pub kn_perfect: bool,
    #[def("KN_CreatureType")]
    pub kn_creature_type: DefIndex,
    #[def("GF_FromRegion")]
    pub gf_from_region: DefString,
    #[def("GF_ToRegion")]
    pub gf_to_region: DefString,
    #[def("GF_NoTeleporting")]
    pub gf_no_teleporting: bool,
    #[def("CO_ItemName")]
    pub co_item_name: DefString,
}
