use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `COpinionOfHeroDef` — C++ `COpinionOfHeroDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct OpinionOfHeroDef {
        "ThreatenGift" => pub threaten_gift: DefIndex,
        "WillAttackCriminals" => pub will_attack_criminals: bool,
        "Personality" => pub personality: i32,
        "ReactionMask" => pub reaction_mask: i32,
        "DeedMask" => pub deed_mask: i32,
    }
}
