use crate::def_struct;

def_struct! {
    /// `CBossDef` — C++ `CBossDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BossDef {
        "PhaseHealthPercent" => pub phase_health_percent: Vec<i32>,
        "Shots" => pub shots: Vec<i32>,
        "TwinBladeMissesBeforeLunge" => pub twin_blade_misses_before_lunge: Vec<i32>,
        "TwinBladeStuckLoops" => pub twin_blade_stuck_loops: i32,
        "TwinBladeTimeBeforeLunge" => pub twin_blade_time_before_lunge: f32,
    }
}
