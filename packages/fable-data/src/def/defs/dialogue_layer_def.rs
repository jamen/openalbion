use crate::def_struct;

def_struct! {
    /// C++ `NSpeechGainManager::CDialogueLayerDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct DialogueLayerDef {
        "ResponseTimeSecsAttack" => pub response_time_secs_attack: f32,
        "ResponseTimeSecsRelease" => pub response_time_secs_release: f32,
        "GainLevelHigh" => pub gain_level_high: f32,
        "GainLevelMid" => pub gain_level_mid: f32,
        "GainLevelLow" => pub gain_level_low: f32,
        "FocusShiftDistanceThreshold" => pub focus_shift_distance_threshold: f32,
    }
}
