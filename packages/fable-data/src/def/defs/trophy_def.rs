use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTrophyDef` — C++ `CTrophyDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TrophyDef {
        "MinigameInitialTimerSecs" => pub minigame_initial_timer_secs: f32,
        "TimeBonusPerWitness" => pub time_bonus_per_witness: f32,
        "RenownGainedPerWitnessAhead" => pub renown_gained_per_witness_ahead: DefIndex,
        "EvilTrophy" => pub evil_trophy: bool,
        "FakeTrophy" => pub fake_trophy: bool,
        "EmoteIconForAvailableWitness" => pub emote_icon_for_available_witness: DefIndex,
        "ShowTrophyAnimName" => pub show_trophy_anim_name: DefString,
        "GameDebugTextScale" => pub game_debug_text_scale: f32,
        "SecsDelayPerGainedWitnessSound" => pub secs_delay_per_gained_witness_sound: f32,
        "SoundStartGame" => pub sound_start_game: String,
        "SoundShowTrophy" => pub sound_show_trophy: String,
        "SoundGainedWitness" => pub sound_gained_witness: String,
        "SoundOutOfTime" => pub sound_out_of_time: String,
        "SoundGotAllWitnesses" => pub sound_got_all_witnesses: String,
        "SoundEndNewHighScore" => pub sound_end_new_high_score: String,
        "SoundEndNoNewScore" => pub sound_end_no_new_score: String,
    }
}
