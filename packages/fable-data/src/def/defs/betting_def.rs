use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CBettingDef` — C++ `CBettingDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BettingDef {
        "BetX" => pub bet_x: f32,
        "BetY" => pub bet_y: f32,
        "MoneyX" => pub money_x: f32,
        "MoneyY" => pub money_y: f32,
        "MinBet" => pub min_bet: i32,
        "MaxBet" => pub max_bet: i32,
        "BetIncrement" => pub bet_increment: i32,
        "Bet" => pub bet: u32,
        "PlayersMoney" => pub players_money: u32,
        "Question" => pub question: u32,
        "Continue" => pub continue_: u32,
        "Cancel" => pub cancel: u32,
    }
}
