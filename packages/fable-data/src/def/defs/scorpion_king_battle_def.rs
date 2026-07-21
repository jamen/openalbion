use crate::def_struct;

def_struct! {
    /// `CScorpionKingBattleDef` — C++ `CScorpionKingBattleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ScorpionKingBattleDef {
        "NumChargesBeforeTailStrike" => pub num_charges_before_tail_strike: i32,
        "UnderGroundTime" => pub under_ground_time: f32,
        "TailPlungeTime" => pub tail_plunge_time: f32,
        "SpikeTime" => pub spike_time: f32,
        "SpikeDecisionTime" => pub spike_decision_time: f32,
        "TailStrikePrepareTime" => pub tail_strike_prepare_time: f32,
        "DamageToCauseTailStrike" => pub damage_to_cause_tail_strike: f32,
        "ChargeDamage" => pub charge_damage: f32 = 10.0,
        "ClawObstructionRadius" => pub claw_obstruction_radius: f32 = 2.0,
    }
}
