use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct OpinionReactionMaskDef {
    #[def("ReactionEnabledDefault", default = true)]
    pub reaction_enabled_default: bool,
    #[def("ReactionEnabled")]
    pub reaction_enabled: BTreeMap<OpinionDeedType, bool>,
}
