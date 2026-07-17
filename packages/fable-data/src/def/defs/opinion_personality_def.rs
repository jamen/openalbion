use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `OPINION_PERSONALITY` — C++ `COpinionPersonalityDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct OpinionPersonalityDef {
        "PersonalityTraits" => pub personality_traits: OpinionPersonalityTraitsPtr,
        "AttitudeEnabledDefaultForNotNone" => pub attitude_enabled_default_for_not_none: bool,
        "AttitudeEnabled" => pub attitude_enabled: VecMap<OpinionDeedType, bool>,
        "ToleranceToBeingHit" => pub tolerance_to_being_hit: f32,
    }
}
