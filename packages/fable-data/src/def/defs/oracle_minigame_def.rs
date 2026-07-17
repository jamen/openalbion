use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `COracleMinigameDef` — C++ `COracleMinigameDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct OracleMinigameDef {
        "WaitBetweenRoundsSeconds" => pub wait_between_rounds_seconds: f32,
        "PauseBetweenOutputSeconds" => pub pause_between_output_seconds: f32,
        "PauseBetweenOutputsLevelMultiplier" => pub pause_between_outputs_level_multiplier: f32,
        "MaxLevel" => pub max_level: i32,
        "TintRed" => pub tint_red: u32,
        "TintGreen" => pub tint_green: u32,
        "TintBlue" => pub tint_blue: u32,
        "TextWin" => pub text_win: u32,
        "TextLose" => pub text_lose: u32,
        "TextWatch" => pub text_watch: u32,
        "TextRepeat" => pub text_repeat: u32,
        "TextOfferGame" => pub text_offer_game: u32,
        "TextWonRound" => pub text_won_round: u32,
    }
}
