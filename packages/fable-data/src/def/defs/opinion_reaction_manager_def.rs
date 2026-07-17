use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `OPINION_REACTION_MANAGER` — C++ `COpinionReactionManagerDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct OpinionReactionManagerDef {
        "AttitudeBlur" => pub attitude_blur: Vec<BlurredAttitudesDef>,
        "AttitudeCondition" => pub attitude_condition: VecMap<OpinionAttitudeType, OpinionReactionType>,
        "TargetingCondition" => pub targeting_condition: VecMap<OpinionTargetingConditionType, OpinionReactionType>,
        "PreReactionDelay" => pub pre_reaction_delay: VecMap<f32, OpinionReactionType>,
        "ToleranceToBeingHit" => pub tolerance_to_being_hit: VecMap<f32, OpinionAttitudeType>,
        "BlockFurtherReactions" => pub block_further_reactions: VecMap<OpinionReactionType, bool>,
        "AllowSpeechOnNonPureAISpeaker" => pub allow_speech_on_non_pure_ai_speaker: VecMap<OpinionReactionType, bool>,
        "AllowWhileCarrying" => pub allow_while_carrying: VecMap<OpinionReactionType, bool>,
        "AllowWhileFollowingPlayer" => pub allow_while_following_player: VecMap<OpinionReactionType, bool>,
        "Matches" => pub matches: ReactionMatchList,
        "Frequency" => pub frequency: ReactionFrequencyTraitsArray,
    }
}
