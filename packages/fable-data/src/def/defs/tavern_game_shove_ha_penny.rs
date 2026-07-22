use crate::DefStruct;
use crate::def::{
    defs::PrizeScoreDef,
    defs::ShoveHaPennyRegionDef,
    wire::DefIndex,
    wire::DefString,
};


#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct TavernGameShoveHaPennyDef {
    #[def("Banter")]
    pub banter: DefIndex,
    #[def("Greeting")]
    pub greeting: DefIndex,
    #[def("OptionsInitial")]
    pub options_initial: DefIndex,
    #[def("OptionsSubsequent")]
    pub options_subsequent: DefIndex,
    #[def("Instructions")]
    pub instructions: DefIndex,
    #[def("InstructionsPC")]
    pub instructions_pc: DefIndex,
    #[def("Betting")]
    pub betting: DefIndex,
    #[def("Play")]
    pub play: DefIndex,
    #[def("ReactionWin")]
    pub reaction_win: DefIndex,
    #[def("ReactionLose")]
    pub reaction_lose: DefIndex,
    #[def("ReactionDraw")]
    pub reaction_draw: DefIndex,
    #[def("ReactionWinNewBestScore")]
    pub reaction_win_new_best_score: DefIndex,
    #[def("FarewellInitial")]
    pub farewell_initial: DefIndex,
    #[def("FarewellSubsequent")]
    pub farewell_subsequent: DefIndex,
    #[def("WinRoundPhrase")]
    pub win_round_phrase: DefIndex,
    #[def("OutOfTimePhrase")]
    pub out_of_time_phrase: DefIndex,
    #[def("NoMoney")]
    pub no_money: DefIndex,
    #[def("GreetingForward")]
    pub greeting_forward: DefIndex,
    #[def("OptionsForward")]
    pub options_forward: DefIndex,
    #[def("OptionsBack")]
    pub options_back: DefIndex,
    #[def("OptionsAlternative")]
    pub options_alternative: DefIndex,
    #[def("InstructionsBack")]
    pub instructions_back: DefIndex,
    #[def("BettingForward")]
    pub betting_forward: DefIndex,
    #[def("BettingBack")]
    pub betting_back: DefIndex,
    #[def("ReactionForward")]
    pub reaction_forward: DefIndex,
    #[def("FarewellForward")]
    pub farewell_forward: DefIndex,
    #[def("CameraName")]
    pub camera_name: DefString,
    #[def("BoxGraphicL")]
    pub box_graphic_l: DefIndex,
    #[def("BoxGraphicC")]
    pub box_graphic_c: DefIndex,
    #[def("BoxGraphicR")]
    pub box_graphic_r: DefIndex,
    #[def("ClickToContinue")]
    pub click_to_continue: DefIndex,
    #[def("WinPhrase")]
    pub win_phrase: DefIndex,
    #[def("LosePhrase")]
    pub lose_phrase: DefIndex,
    #[def("DrawPhrase")]
    pub draw_phrase: DefIndex,
    #[def("NewGame")]
    pub new_game: DefIndex,
    #[def("BestScore")]
    pub best_score: DefIndex,
    #[def("CurrentScore")]
    pub current_score: DefIndex,
    #[def("RequiredScore")]
    pub required_score: DefIndex,
    #[def("AdditionalInfo")]
    pub additional_info: DefIndex,
    #[def("BlackjackBusted")]
    pub blackjack_busted: DefIndex,
    #[def("BlackjackSplit")]
    pub blackjack_split: DefIndex,
    #[def("BlackjackDouble")]
    pub blackjack_double: DefIndex,
    #[def("BlackjackHit")]
    pub blackjack_hit: DefIndex,
    #[def("BlackjackStand")]
    pub blackjack_stand: DefIndex,
    #[def("BlackjackDealerTakesCard")]
    pub blackjack_dealer_takes_card: DefIndex,
    #[def("BlackjackSplitGUI")]
    pub blackjack_split_gui: DefIndex,
    #[def("BlackjackDoubleGUI")]
    pub blackjack_double_gui: DefIndex,
    #[def("BlackjackHitGUI")]
    pub blackjack_hit_gui: DefIndex,
    #[def("BlackjackStandGUI")]
    pub blackjack_stand_gui: DefIndex,
    #[def("Bet")]
    pub bet: DefIndex,
    #[def("PlayersMoney")]
    pub players_money: DefIndex,
    #[def("TotalWinnings")]
    pub total_winnings: DefIndex,
    #[def("Continue")]
    pub continue_: DefIndex,
    #[def("Quit")]
    pub quit: DefIndex,
    #[def("Yes")]
    pub yes: DefIndex,
    #[def("No")]
    pub no: DefIndex,
    #[def("PrizeGiven")]
    pub prize_given: DefIndex,
    #[def("MoneyBagGraphic")]
    pub money_bag_graphic: DefIndex,
    #[def("MinBet")]
    pub min_bet: DefIndex,
    #[def("MaxBet")]
    pub max_bet: DefIndex,
    #[def("BetIncrement")]
    pub bet_increment: DefIndex,
    #[def("ScoreFont")]
    pub score_font: DefString,
    #[def("TargetFont")]
    pub target_font: DefString,
    #[def("StatsFont")]
    pub stats_font: DefString,
    #[def("ScoreX")]
    pub score_x: f32,
    #[def("ScoreY")]
    pub score_y: f32,
    #[def("TargetX")]
    pub target_x: f32,
    #[def("TargetY")]
    pub target_y: f32,
    #[def("BestX")]
    pub best_x: f32,
    #[def("BestY")]
    pub best_y: f32,
    #[def("AdditionalX")]
    pub additional_x: f32,
    #[def("AdditionalY")]
    pub additional_y: f32,
    #[def("BetX")]
    pub bet_x: f32,
    #[def("BetY")]
    pub bet_y: f32,
    #[def("MoneyX")]
    pub money_x: f32,
    #[def("MoneyY")]
    pub money_y: f32,
    #[def("WinningsX")]
    pub winnings_x: f32,
    #[def("WinningsY")]
    pub winnings_y: f32,
    #[def("MainBetX")]
    pub main_bet_x: f32,
    #[def("MainBetY")]
    pub main_bet_y: f32,
    #[def("MainMoneyX")]
    pub main_money_x: f32,
    #[def("MainMoneyY")]
    pub main_money_y: f32,
    #[def("BestScoreHigh")]
    pub best_score_high: bool,
    #[def("PrizeScores")]
    pub prize_scores: Vec<PrizeScoreDef>,
    #[def("Prize")]
    pub prize: DefIndex,
    #[def("PrizeRenown")]
    pub prize_renown: DefIndex,
    #[def("MainGameScoreBoxX")]
    pub main_game_score_box_x: f32,
    #[def("MainGameScoreBoxY")]
    pub main_game_score_box_y: f32,
    #[def("MainGameScoreBoxWidthXbox")]
    pub main_game_score_box_width_xbox: f32,
    #[def("MainGameScoreBoxWidthPC")]
    pub main_game_score_box_width_pc: f32,
    #[def("MainGameScoreBoxHeight")]
    pub main_game_score_box_height: f32,
    #[def("DisplayErrata")]
    pub display_errata: bool,
    #[def("PointerPhaseSpeed")]
    pub pointer_phase_speed: f32,
    #[def("Regions")]
    pub regions: Vec<ShoveHaPennyRegionDef>,
    #[def("BoardDefIndex")]
    pub board_def_index: DefIndex,
    #[def("ArrowDefIndex")]
    pub arrow_def_index: DefIndex,
    #[def("ImpulseScale")]
    pub impulse_scale: f32,
    #[def("TargetDistanceScale")]
    pub target_distance_scale: f32,
    #[def("JoystickScaleXboxX")]
    pub joystick_scale_xbox_x: f32,
    #[def("JoystickScaleXboxY")]
    pub joystick_scale_xbox_y: f32,
    #[def("JoystickScalePCX")]
    pub joystick_scale_pcx: f32,
    #[def("JoystickScalePCY")]
    pub joystick_scale_pcy: f32,
    #[def("JoystickTranslateY")]
    pub joystick_translate_y: f32,
    #[def("JoystickMagBelow")]
    pub joystick_mag_below: f32,
    #[def("AverageJoystickMagAbove")]
    pub average_joystick_mag_above: f32,
    #[def("MaxPower")]
    pub max_power: f32,
    #[def("AverageDrift")]
    pub average_drift: f32,
    #[def("PositionCoinSpeedXbox")]
    pub position_coin_speed_xbox: f32,
    #[def("PositionCoinSpeedPC")]
    pub position_coin_speed_pc: f32,
    #[def("PositionCoinMinimum")]
    pub position_coin_minimum: f32,
    #[def("PositionCoinMaximum")]
    pub position_coin_maximum: f32,
    #[def("PositionCoinDepth")]
    pub position_coin_depth: f32,
    #[def("RestSpeed")]
    pub rest_speed: f32,
    #[def("Rounds")]
    pub rounds: DefIndex,
}
