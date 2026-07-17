use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CHeroMarriageDef` — C++ `CHeroMarriageDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroMarriageDef {
        "WeddingTimeForScreenToFadeOut" => pub wedding_time_for_screen_to_fade_out: f32,
        "WeddingTimeForScreenToFadeIn" => pub wedding_time_for_screen_to_fade_in: f32,
        "WeddingAfterFrescoTimeOfDayFastForwardTo" => pub wedding_after_fresco_time_of_day_fast_forward_to: f32,
        "SecondsJustMarriedDuration" => pub seconds_just_married_duration: f32,
        "HeroStatRenownIncreasePerMarriage" => pub hero_stat_renown_increase_per_marriage: f32,
        "SecondsBetweenCheckingPlayersAppearance" => pub seconds_between_checking_players_appearance: DefIndex,
        "SecondsMaxTimeWithinNoticingCanReact" => pub seconds_max_time_within_noticing_can_react: DefIndex,
        "FatnessChangeThresholdForComment" => pub fatness_change_threshold_for_comment: f32,
        "SecondsBetweenReceivingGiftOpinionReactionCanHappen" => pub seconds_between_receiving_gift_opinion_reaction_can_happen: DefIndex,
        "SecondsThatGiftsReceivedWillBeRemembered" => pub seconds_that_gifts_received_will_be_remembered: DefIndex,
        "FramesBetweenCullingOldGifts" => pub frames_between_culling_old_gifts: DefIndex,
        "SecondsBetweenEvaluatingGiftOpinion" => pub seconds_between_evaluating_gift_opinion: DefIndex,
        "SecondsBetweenGiftGivingOpportunities" => pub seconds_between_gift_giving_opportunities: DefIndex,
        "ChanceThatGiftWillBeGiven" => pub chance_that_gift_will_be_given: f32,
        "AmountOfGoldPerHourOfMarriageToIncreaseGiftGivingPriceValuePerHourOfPlay" => pub amount_of_gold_per_hour_of_marriage_to_increase_gift_giving_price_value_per_hour_of_play: f32,
        "MaxNumHoursOfMarriageToIncreaseGiftGivingPriceValue" => pub max_num_hours_of_marriage_to_increase_gift_giving_price_value: f32,
        "MultiplierForGiftGivingValueEveryOpportunity" => pub multiplier_for_gift_giving_value_every_opportunity: f32,
        "MaxDistanceFromMaxStatsThatGiftsCanBeGiven" => pub max_distance_from_max_stats_that_gifts_can_be_given: f32,
        "SecondsBetweenEvaluatingLoveAttitude" => pub seconds_between_evaluating_love_attitude: DefIndex,
        "SecondsOutOfTheRegionForLoveAttitudeToDecayOneToZero" => pub seconds_out_of_the_region_for_love_attitude_to_decay_one_to_zero: f32,
        "ThresholdForLoveAttitudeOverride" => pub threshold_for_love_attitude_override: f32,
        "FallenInLoveSoundCriteria" => pub fallen_in_love_sound_criteria: String,
        "FallenInLoveSoundGain" => pub fallen_in_love_sound_gain: f32,
        "FianceeOpinionPersonalityDef" => pub fiancee_opinion_personality_def: DefIndex,
        "ProbabilityOfCourtingAdviceOnGreet" => pub probability_of_courting_advice_on_greet: f32,
        "SecondsTimeSinceSeeingYouMinThreshold" => pub seconds_time_since_seeing_you_min_threshold: f32,
        "SecondsRunOutWifeOnViolenceHateSpike" => pub seconds_run_out_wife_on_violence_hate_spike: f32,
        "PeakEffectWifeOnViolenceHateSpike" => pub peak_effect_wife_on_violence_hate_spike: f32,
        "PersistEffectWifeOnViolenceHateSpike" => pub persist_effect_wife_on_violence_hate_spike: f32,
        "SecsInHateBeforeDivorceOccurs" => pub secs_in_hate_before_divorce_occurs: f32,
        "SecsInHateBetweenDivorceWarnings" => pub secs_in_hate_between_divorce_warnings: f32,
        "TextGuiWifeDivorceOccurred" => pub text_gui_wife_divorce_occurred: String,
        "TextGuiWifeDivorceRenting" => pub text_gui_wife_divorce_renting: String,
        "HeroStatMoralityInOpinionFormChangePerDivorce" => pub hero_stat_morality_in_opinion_form_change_per_divorce: f32,
        "SecsInLoveAndHusbandPresentBeforeSexOffered" => pub secs_in_love_and_husband_present_before_sex_offered: f32,
        "SecsInLoveAndHusbandPresentBetweenSexOffers" => pub secs_in_love_and_husband_present_between_sex_offers: f32,
        "TextGuiWifeSexOffered" => pub text_gui_wife_sex_offered: String,
        "SecsMinSoundDurationInSexCutscene" => pub secs_min_sound_duration_in_sex_cutscene: f32,
        "SecsWaitAfterSoundInSexCutscene" => pub secs_wait_after_sound_in_sex_cutscene: f32,
        "DialogueGroupForSexCutscene" => pub dialogue_group_for_sex_cutscene: String,
    }
}
