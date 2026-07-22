use crate::DefStruct;

/// `CBossDef` — C++ `CBossDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct BossDef {
    #[def("PhaseHealthPercent")]
    pub phase_health_percent: Vec<i32>,
    #[def("Shots")]
    pub shots: Vec<i32>,
    #[def("TwinBladeMissesBeforeLunge")]
    pub twin_blade_misses_before_lunge: Vec<i32>,
    #[def("TwinBladeStuckLoops")]
    pub twin_blade_stuck_loops: i32,
    #[def("TwinBladeTimeBeforeLunge")]
    pub twin_blade_time_before_lunge: f32,
}
