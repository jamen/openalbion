use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `OPINION_REACTION_MASK` — C++ `COpinionReactionMaskDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct OpinionReactionMaskDef {
        "ReactionEnabledDefault" => pub reaction_enabled_default: bool,
        "ReactionEnabled" => pub reaction_enabled: VecMap<OpinionDeedType, bool>,
    }
}
