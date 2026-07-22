use crate::DefStruct;
use crate::def::prelude::*;

/// `COpinionOfHeroDef` — C++ `COpinionOfHeroDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct OpinionOfHeroDef {
    #[def("ThreatenGift")]
    pub threaten_gift: DefIndex,
    #[def("WillAttackCriminals")]
    pub will_attack_criminals: bool,
    #[def("Personality")]
    pub personality: i32,
    #[def("ReactionMask")]
    pub reaction_mask: i32,
    #[def("DeedMask")]
    pub deed_mask: i32,
}
