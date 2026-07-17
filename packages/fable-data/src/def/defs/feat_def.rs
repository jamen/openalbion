use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CFeatDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct FeatDef {
        "FeatName" => pub feat_name: DefString,
        "Verb" => pub verb: DefString,
        "TimeLimit" => pub time_limit: f32,
        "TargetNumber" => pub target_number: DefIndex,
        "GoldReward" => pub gold_reward: DefIndex,
        "XPReward" => pub xp_reward: DefIndex,
        "ItemReward" => pub item_reward: DefString,
        "NoBlocking" => pub no_blocking: bool,
        "KN_AttackType" => pub kn_attack_type: FeatAttackType,
        "KN_Perfect" => pub kn_perfect: bool,
        "KN_CreatureType" => pub kn_creature_type: DefIndex,
        "GF_FromRegion" => pub gf_from_region: DefString,
        "GF_ToRegion" => pub gf_to_region: DefString,
        "GF_NoTeleporting" => pub gf_no_teleporting: bool,
        "CO_ItemName" => pub co_item_name: DefString,
    }
}
