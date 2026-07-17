use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `OPINION_DEED_EFFECTS` — C++ `COpinionDeedEffectsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct OpinionDeedEffectsDef {
        "KnowledgeLifeInSeconds" => pub knowledge_life_in_seconds: i32,
        "MinimumSecondsBetweenPostings" => pub minimum_seconds_between_postings: i32,
        "MinimumSecondsBetweenEffectsForIndividual" => pub minimum_seconds_between_effects_for_individual: i32,
        "PreReactionDelayIgnoreOnDelay" => pub pre_reaction_delay_ignore_on_delay: f32,
        "Effects" => pub effects: OpinionTransientOffsetList,
        "ReactionPriority" => pub reaction_priority: i32,
        "Reaction" => pub reaction: Vec<OpinionDeedReactionDef>,
        "ReactionDialogueLayer" => pub reaction_dialogue_layer: DialogueLayer,
        "WarningText" => pub warning_text: Vec<String>,
        "CrimeSeverity" => pub crime_severity: CrimeSeverity,
        "CrimeFineAmount" => pub crime_fine_amount: i32,
        "Tutorial" => pub tutorial: TutorialCategory,
        "CanBeHeard" => pub can_be_heard: bool,
        "ReactionRestrictToTarget" => pub reaction_restrict_to_target: bool,
        "AllowSimultaneousReactionDialogue" => pub allow_simultaneous_reaction_dialogue: bool,
        "CanApologise" => pub can_apologise: bool,
        "MustNotRunWhenWarned" => pub must_not_run_when_warned: bool,
        "OnlyGetFinedForOneInstance" => pub only_get_fined_for_one_instance: bool,
        "IsFlirting" => pub is_flirting: bool,
        "OnReactSetGreeted" => pub on_react_set_greeted: bool,
        "AllowSpeechOnNonPureAISpeaker" => pub allow_speech_on_non_pure_ai_speaker: bool,
        "AllowIndirectReactWhileCarrying" => pub allow_indirect_react_while_carrying: bool,
        "BlockWhileFollowingPlayer" => pub block_while_following_player: bool,
    }
}
